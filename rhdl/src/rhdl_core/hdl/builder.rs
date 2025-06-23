use crate::rhdl_core::{
    bitx::BitX,
    compiler::mir::error::{RHDLCompileError, ICE},
    error::rhdl_error,
    hdl::ast::{
        self, assign, concatenate, constant, declaration, id, index, index_bit, input_reg, literal,
        repeat, unary, CaseItem, Function, HDLKind,
    },
    rtl::{
        self,
        object::LocatedOpCode,
        spec::{AluUnary, CaseArgument, CastKind, Operand},
        SourceOpCode,
    },
    RHDLError,
};

use rtl::spec as tl;

type Result<T> = std::result::Result<T, RHDLError>;

struct TranslationContext<'a> {
    func: Function,
    rtl: &'a rtl::Object,
}

impl TranslationContext<'_> {
    fn raise_ice(&self, cause: ICE, id: SourceOpCode) -> RHDLError {
        rhdl_error(RHDLCompileError {
            cause,
            src: self.rtl.symbols.source(),
            err_span: self.rtl.symbols.span(id).into(),
        })
    }
    /// Cast the argument ot the desired width, considering the result a signed value.
    /// The cast length must be less than or equal to the argument length, or an ICE is raised.
    fn translate_as_signed(&mut self, cast: &tl::Cast, id: SourceOpCode) -> Result<()> {
        if cast.len > self.rtl.kind(cast.arg).len() {
            return Err(self.raise_ice(
                ICE::InvalidSignedCast {
                    lhs: cast.lhs,
                    arg: cast.arg,
                    len: cast.len,
                },
                id,
            ));
        }
        let lhs = self.rtl.op_name(cast.lhs);
        let arg = self.rtl.op_name(cast.arg);
        let cast = index(&arg, 0..cast.len);
        let signed = unary(AluUnary::Signed, cast);
        self.func.block.push(assign(&lhs, signed));
        Ok(())
    }
    /// Cast the argument to the desired width, with no error and no sign extension
    fn translate_resize_unsigned(&mut self, cast: &tl::Cast) {
        let arg_kind = self.rtl.kind(cast.arg);
        let lhs = self.rtl.op_name(cast.lhs);
        let arg = self.rtl.op_name(cast.arg);
        // Truncation case
        if cast.len <= arg_kind.len() {
            self.func.block.push(assign(&lhs, index(&arg, 0..cast.len)));
        } else {
            // zero extend
            let num_z = cast.len - arg_kind.len();
            let prefix = repeat(constant(BitX::Zero), num_z);
            self.func
                .block
                .push(assign(&lhs, concatenate(vec![prefix, id(&arg)])));
        }
    }
    /// Cast the argument to the desired width, with sign extension if needed.
    fn translate_resize_signed(&mut self, cast: &tl::Cast) {
        let arg_kind = self.rtl.kind(cast.arg);
        let lhs = self.rtl.op_name(cast.lhs);
        let arg = self.rtl.op_name(cast.arg);
        // Truncation case
        if cast.len <= arg_kind.len() {
            self.func.block.push(assign(
                &lhs,
                unary(AluUnary::Signed, index(&arg, 0..cast.len)),
            ));
        } else {
            // sign extend
            let num_s = cast.len - arg_kind.len();
            let sign_bit = index_bit(&arg, arg_kind.len().saturating_sub(1));
            let prefix = repeat(sign_bit, num_s);
            self.func.block.push(assign(
                &lhs,
                unary(AluUnary::Signed, concatenate(vec![prefix, id(&arg)])),
            ));
        }
    }
    fn translate_resize(&mut self, cast: &tl::Cast, id: SourceOpCode) -> Result<()> {
        if cast.len == 0 {
            return Err(self.raise_ice(
                ICE::InvalidResize {
                    lhs: cast.lhs,
                    arg: cast.arg,
                    len: cast.len,
                },
                id,
            ));
        }
        if self.rtl.kind(cast.arg).is_signed() {
            self.translate_resize_signed(cast);
        } else {
            self.translate_resize_unsigned(cast);
        }
        Ok(())
    }
    fn translate_cast(&mut self, cast: &tl::Cast, id: SourceOpCode) -> Result<()> {
        match cast.kind {
            CastKind::Signed => self.translate_as_signed(cast, id),
            CastKind::Unsigned => {
                self.translate_resize_unsigned(cast);
                Ok(())
            }
            CastKind::Resize => self.translate_resize(cast, id),
        }
    }
    fn translate_assign(&mut self, assign: &tl::Assign) -> Result<()> {
        self.func.block.push(ast::assign(
            &self.rtl.op_name(assign.lhs),
            id(&self.rtl.op_name(assign.rhs)),
        ));
        Ok(())
    }
    fn translate_binary(&mut self, binary: &tl::Binary) -> Result<()> {
        self.func.block.push(ast::assign(
            &self.rtl.op_name(binary.lhs),
            ast::binary(
                binary.op,
                id(&self.rtl.op_name(binary.arg1)),
                id(&self.rtl.op_name(binary.arg2)),
            ),
        ));
        Ok(())
    }
    fn translate_case(&mut self, case: &tl::Case) -> Result<()> {
        let discriminant = id(&self.rtl.op_name(case.discriminant));
        let lhs = self.rtl.op_name(case.lhs);
        let table = case
            .table
            .iter()
            .map(|(arg, value)| {
                let item = match arg {
                    CaseArgument::Literal(lit) => {
                        let lit = self.rtl.literals[lit].clone();
                        CaseItem::Literal(lit)
                    }
                    CaseArgument::Wild => CaseItem::Wild,
                };
                let assign = assign(&lhs, id(&self.rtl.op_name(*value)));
                (item, assign)
            })
            .collect();
        self.func.block.push(ast::case(discriminant, table));
        Ok(())
    }
    fn translate_concat(&mut self, concat: &tl::Concat) -> Result<()> {
        let args = concat
            .args
            .iter()
            .rev()
            .map(|arg| id(&self.rtl.op_name(*arg)))
            .collect::<Vec<_>>();
        let lhs = self.rtl.op_name(concat.lhs);
        self.func.block.push(assign(&lhs, concatenate(args)));
        Ok(())
    }
    fn translate_index(&mut self, index: &tl::Index) -> Result<()> {
        let lhs = self.rtl.op_name(index.lhs);
        let arg = self.rtl.op_name(index.arg);
        self.func
            .block
            .push(assign(&lhs, ast::index(&arg, index.bit_range.clone())));
        Ok(())
    }
    fn translate_select(&mut self, select: &tl::Select) -> Result<()> {
        let lhs = self.rtl.op_name(select.lhs);
        let cond = self.rtl.op_name(select.cond);
        let true_value = self.rtl.op_name(select.true_value);
        let false_value = self.rtl.op_name(select.false_value);
        self.func.block.push(assign(
            &lhs,
            ast::select(id(&cond), id(&true_value), id(&false_value)),
        ));
        Ok(())
    }
    fn translate_splice(&mut self, splice: &tl::Splice) -> Result<()> {
        let lhs = self.rtl.op_name(splice.lhs);
        let orig = self.rtl.op_name(splice.orig);
        let value = self.rtl.op_name(splice.value);
        self.func.block.push(ast::splice(
            &lhs,
            id(&orig),
            splice.bit_range.clone(),
            id(&value),
        ));
        Ok(())
    }
    fn translate_unary(&mut self, unary: &tl::Unary) -> Result<()> {
        let lhs = self.rtl.op_name(unary.lhs);
        let arg1 = self.rtl.op_name(unary.arg1);
        self.func
            .block
            .push(assign(&lhs, ast::unary(unary.op, id(&arg1))));
        Ok(())
    }
    fn translate_comment(&mut self, comment: &str) -> Result<()> {
        self.func.block.push(ast::comment(comment));
        Ok(())
    }
    fn translate_op(&mut self, lop: &LocatedOpCode) -> Result<()> {
        let op = &lop.op;
        match op {
            tl::OpCode::Noop => Ok(()),
            tl::OpCode::Assign(assign) => self.translate_assign(assign),
            tl::OpCode::Binary(binary) => self.translate_binary(binary),
            tl::OpCode::Case(case) => self.translate_case(case),
            tl::OpCode::Cast(cast) => self.translate_cast(cast, lop.loc),
            tl::OpCode::Comment(comment) => self.translate_comment(comment),
            tl::OpCode::Concat(concat) => self.translate_concat(concat),
            tl::OpCode::Index(index) => self.translate_index(index),
            tl::OpCode::Select(select) => self.translate_select(select),
            tl::OpCode::Splice(splice) => self.translate_splice(splice),
            tl::OpCode::Unary(unary) => self.translate_unary(unary),
        }
    }
    fn translate_block(&mut self, block: &[LocatedOpCode]) -> Result<()> {
        block.iter().try_for_each(|lop| self.translate_op(lop))
    }
    fn translate_kernel_for_object(mut self) -> Result<Function> {
        let arg_decls = self
            .rtl
            .arguments
            .iter()
            .enumerate()
            .flat_map(|(ndx, x)| x.map(|r| (ndx, r)))
            .map(|(ndx, reg)| input_reg(&format!("arg_{ndx}"), self.rtl.register_kind[&reg].into()))
            .collect::<Vec<_>>();
        self.func.arguments = arg_decls;
        let reg_decls = self
            .rtl
            .register_kind
            .keys()
            .map(|reg| {
                let alias = self.rtl.op_alias(Operand::Register(*reg));
                declaration(
                    HDLKind::Reg,
                    &self.rtl.op_name(Operand::Register(*reg)),
                    self.rtl.register_kind[reg].into(),
                    alias,
                )
            })
            .collect::<Vec<_>>();
        self.func.registers = reg_decls;
        let literals = self
            .rtl
            .literals
            .iter()
            .map(|(lit, bs)| literal(&self.rtl.op_name(Operand::Literal(*lit)), bs))
            .collect::<Vec<_>>();
        self.func.literals = literals;
        // Bind the argument registers
        for (ndx, reg) in self.rtl.arguments.iter().enumerate() {
            if let Some(reg) = reg {
                let reg_name = self.rtl.op_name(Operand::Register(*reg));
                let arg_name = format!("arg_{}", ndx);
                self.func.block.push(assign(&reg_name, id(&arg_name)));
            }
        }
        self.translate_block(&self.rtl.ops)?;
        self.func.block.push(assign(
            &self.func.name,
            id(&self.rtl.op_name(self.rtl.return_register)),
        ));
        Ok(self.func)
    }
}

fn translate(object: &crate::rhdl_core::rtl::Object) -> Result<Function> {
    let context = TranslationContext {
        func: Function {
            name: format!("kernel_{}", object.name),
            width: object.kind(object.return_register).into(),
            arguments: vec![],
            registers: vec![],
            literals: vec![],
            block: vec![],
        },
        rtl: object,
    };
    context.translate_kernel_for_object()
}

pub fn generate_verilog(object: &crate::rhdl_core::rtl::Object) -> Result<Function> {
    translate(object)
}
