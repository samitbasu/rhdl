use std::collections::BTreeSet;

use crate::{
    ast::source_location::SourceLocation,
    compiler::mir::error::{RHDLCompileError, ICE},
    error::rhdl_error,
    rhif::spec::AluUnary,
    rtl::{
        self,
        object::{LocatedOpCode, RegisterKind},
        spec::{CastKind, Operand},
    },
    test_module::VerilogDescriptor,
    types::bit_string::BitString,
    util::binary_string,
    verilog::ast::{
        self, assign, concatenate, constant, declaration, id, index, input_reg, literal, repeat,
        unary, Function, Kind,
    },
    RHDLError,
};

use rtl::spec as tl;

type Result<T> = std::result::Result<T, RHDLError>;

struct TranslationContext<'a> {
    func: Function,
    rtl: &'a rtl::Object,
}

impl<'a> TranslationContext<'a> {
    fn raise_ice(&self, cause: ICE, id: SourceLocation) -> RHDLError {
        rhdl_error(RHDLCompileError {
            cause,
            src: self.rtl.symbols.source(),
            err_span: self.rtl.symbols.span(id).into(),
        })
    }
    fn translate_as_bits(&mut self, cast: &tl::Cast) -> Result<()> {
        // Check for the extension case
        let arg_kind = self.rtl.kind(cast.arg);
        let lhs = self.rtl.op_name(cast.lhs);
        let arg = self.rtl.op_name(cast.arg);
        if cast.len <= arg_kind.len() {
            self.func
                .block
                .push(assign(&lhs, index(id(&arg), 0..cast.len)));
        } else {
            // zero extend
            let num_z = cast.len - arg_kind.len();
            let prefix = repeat(constant(false), num_z);
            self.func
                .block
                .push(assign(&lhs, concatenate(vec![prefix, id(&arg)])));
        }
        Ok(())
    }
    fn translate_as_signed(&mut self, cast: &tl::Cast, id: SourceLocation) -> Result<()> {
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
        let cast = index(ast::id(&arg), 0..cast.len);
        let signed = unary(AluUnary::Signed, cast);
        self.func.block.push(assign(&lhs, signed));
        Ok(())
    }
    fn translate_resize_unsigned(&mut self, cast: &tl::Cast) {
        let arg_kind = self.rtl.kind(cast.arg);
        let lhs = self.rtl.op_name(cast.lhs);
        let arg = self.rtl.op_name(cast.arg);
        // Truncation case
        if cast.len <= arg_kind.len() {
            self.func.push_str(&format!(
                "    {lhs} = {arg}[{len}:0];\n",
                lhs = self.rtl.op_name(cast.lhs),
                arg = self.rtl.op_name(cast.arg),
                len = cast.len - 1
            ));
        } else {
            // zero extend
            let num_z = cast.len - arg_kind.len();
            self.func.push_str(&format!(
                "    {lhs} = {{ {num_z}'b0, {arg} }};\n",
                lhs = self.rtl.op_name(cast.lhs),
                arg = self.rtl.op_name(cast.arg),
                num_z = num_z
            ));
        }
    }
    fn translate_resize_signed(&mut self, cast: &tl::Cast) {
        let arg_kind = self.rtl.kind(cast.arg);
        // Truncation case
        if cast.len <= arg_kind.len() {
            self.func.push_str(&format!(
                "    {lhs} = $signed({arg}[{len}:0]);\n",
                lhs = self.rtl.op_name(cast.lhs),
                arg = self.rtl.op_name(cast.arg),
                len = cast.len - 1
            ));
        } else {
            // sign extend
            let num_s = cast.len - arg_kind.len();
            self.func.push_str(&format!(
                "    {lhs} = $signed({{ {{ {num_s}{{ {arg}[{arg_len}]}} }}, {arg} }});\n",
                lhs = self.rtl.op_name(cast.lhs),
                arg = self.rtl.op_name(cast.arg),
                num_s = num_s,
                arg_len = arg_kind.len() - 1
            ));
        }
    }
    fn translate_resize(&mut self, cast: &tl::Cast, id: SourceLocation) -> Result<()> {
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
    fn translate_cast(&mut self, cast: &tl::Cast, id: SourceLocation) -> Result<()> {
        match cast.kind {
            CastKind::Signed => self.translate_as_signed(cast, id),
            CastKind::Unsigned => self.translate_as_bits(cast),
            CastKind::Resize => self.translate_resize(cast, id),
        }
    }
    fn translate_assign(&mut self, assign: &tl::Assign) -> Result<()> {
        self.func.push_str(&format!(
            "    {lhs} = {rhs};\n",
            lhs = self.rtl.op_name(assign.lhs),
            rhs = self.rtl.op_name(assign.rhs)
        ));
        Ok(())
    }
    fn translate_binary(&mut self, binary: &tl::Binary) -> Result<()> {
        self.func.push_str(&format!(
            "    {lhs} = {arg1} {op} {arg2};\n",
            op = binary.op.verilog_binop(),
            lhs = self.rtl.op_name(binary.lhs),
            arg1 = self.rtl.op_name(binary.arg1),
            arg2 = self.rtl.op_name(binary.arg2)
        ));
        Ok(())
    }
    fn translate_case(&mut self, case: &tl::Case) -> Result<()> {
        self.func.push_str(&format!(
            "    case ({sel})\n",
            sel = self.rtl.op_name(case.discriminant)
        ));
        for (val, block) in &case.table {
            match val {
                tl::CaseArgument::Literal(lit) => {
                    self.func.push_str(&format!(
                        "      {}: ",
                        verilog_literal(&self.rtl.literals[lit])
                    ));
                    self.func.push_str(&format!(
                        "{} = {};\n",
                        self.rtl.op_name(case.lhs),
                        self.rtl.op_name(*block),
                    ));
                }
                tl::CaseArgument::Wild => {
                    self.func.push_str("      default: ");
                    self.func.push_str(&format!(
                        "{} = {};\n",
                        self.rtl.op_name(case.lhs),
                        self.rtl.op_name(*block),
                    ));
                }
            }
        }
        self.func.push_str("    endcase\n");
        Ok(())
    }
    fn translate_concat(&mut self, concat: &tl::Concat) -> Result<()> {
        let args = concat
            .args
            .iter()
            .rev()
            .map(|arg| self.rtl.op_name(*arg))
            .collect::<Vec<_>>()
            .join(", ");
        let lhs = self.rtl.op_name(concat.lhs);
        self.func.push_str(&format!("    {lhs} = {{{args}}};\n",));
        Ok(())
    }
    fn translate_dynamic_index(&mut self, index: &tl::DynamicIndex) -> Result<()> {
        let lhs = self.rtl.op_name(index.lhs);
        let arg = self.rtl.op_name(index.arg);
        let offset = self.rtl.op_name(index.offset);
        let len = index.len;
        self.func
            .push_str(&format!("    {lhs} = {arg}[{offset} +: {len}];\n",));
        Ok(())
    }
    fn translate_dynamic_splice(&mut self, splice: &tl::DynamicSplice) -> Result<()> {
        let lhs = self.rtl.op_name(splice.lhs);
        let arg = self.rtl.op_name(splice.arg);
        let offset = self.rtl.op_name(splice.offset);
        let value = self.rtl.op_name(splice.value);
        let len = splice.len;
        self.func.push_str(&format!(
            "    {lhs} = {arg}; {lhs}[{offset} +: {len}] = {value};\n",
        ));
        Ok(())
    }
    fn translate_index(&mut self, index: &tl::Index) -> Result<()> {
        let lhs = self.rtl.op_name(index.lhs);
        let arg = self.rtl.op_name(index.arg);
        let start = index.bit_range.start;
        let end = index.bit_range.end - 1;
        self.func
            .push_str(&format!("    {lhs} = {arg}[{end}:{start}];\n",));
        Ok(())
    }
    fn translate_select(&mut self, select: &tl::Select) -> Result<()> {
        let lhs = self.rtl.op_name(select.lhs);
        let cond = self.rtl.op_name(select.cond);
        let true_value = self.rtl.op_name(select.true_value);
        let false_value = self.rtl.op_name(select.false_value);
        self.func.push_str(&format!(
            "    {lhs} = {cond} ? {true_value} : {false_value};\n",
        ));
        Ok(())
    }
    fn translate_splice(&mut self, splice: &tl::Splice) -> Result<()> {
        let lhs = self.rtl.op_name(splice.lhs);
        let orig = self.rtl.op_name(splice.orig);
        let start = splice.bit_range.start;
        let end = splice.bit_range.end - 1;
        let value = self.rtl.op_name(splice.value);
        self.func.push_str(&format!(
            "    {lhs} = {orig}; {lhs}[{end}:{start}] = {value};\n",
        ));
        Ok(())
    }
    fn translate_unary(&mut self, unary: &tl::Unary) -> Result<()> {
        let lhs = self.rtl.op_name(unary.lhs);
        let arg1 = self.rtl.op_name(unary.arg1);
        match &unary.op {
            AluUnary::Signed => {
                self.func
                    .push_str(&format!("    {lhs} = $signed({arg1});\n",));
            }
            AluUnary::Unsigned => {
                self.func
                    .push_str(&format!("    {lhs} = $unsigned({arg1});\n",));
            }
            _ => {
                let op = unary.op.verilog_unop();
                self.func.push_str(&format!("    {lhs} = {op}{arg1};\n",));
            }
        }
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
            tl::OpCode::Comment(_) => Ok(()),
            tl::OpCode::Concat(concat) => self.translate_concat(concat),
            tl::OpCode::DynamicIndex(index) => self.translate_dynamic_index(index),
            tl::OpCode::DynamicSplice(splice) => self.translate_dynamic_splice(splice),
            tl::OpCode::Index(index) => self.translate_index(index),
            tl::OpCode::Select(select) => self.translate_select(select),
            tl::OpCode::Splice(splice) => self.translate_splice(splice),
            tl::OpCode::Unary(unary) => self.translate_unary(unary),
        }
    }
    fn translate_block(&mut self, block: &[LocatedOpCode]) -> Result<()> {
        block.iter().try_for_each(|lop| self.translate_op(lop))
    }
    fn translate_kernel_for_object(mut self) -> Result<VerilogModule> {
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
                    Kind::Reg,
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
        self.func.push_str("    // Body\n");
        self.func.push_str("begin\n");
        // Bind the argument registers
        for (ndx, reg) in self.rtl.arguments.iter().enumerate() {
            if let Some(reg) = reg {
                self.func.push_str(&format!(
                    "    {} = arg_{};\n",
                    self.rtl.op_name(Operand::Register(*reg)),
                    ndx
                ));
            }
        }
        self.translate_block(&self.rtl.ops)?;
        self.func.push_str(&format!(
            "    {} = {};\n",
            func_name,
            self.rtl.op_name(self.rtl.return_register)
        ));
        self.func.push_str("end\n");
        self.func.push_str("endfunction\n");
        let mut module = VerilogModule::default();
        for kernel in self.kernels {
            module.functions.extend(kernel.functions);
        }
        module.functions.push(self.func.to_string());
        Ok(module)
    }
}

fn translate(object: &crate::rtl::Object) -> Result<Function> {
    let context = TranslationContext {
        func: Function {
            name: object.name.clone(),
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

pub fn generate_verilog(object: &crate::rtl::Object) -> Result<Function> {
    translate(object)
}
