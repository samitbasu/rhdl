use crate::{
    RHDLError, TypedBits,
    ast::source::source_location::SourceLocation,
    bitx::BitX,
    compiler::mir::error::{ICE, RHDLCompileError},
    error::rhdl_error,
    rtl::{
        self,
        object::LocatedOpCode,
        spec::{AluUnary, CaseArgument, CastKind, Operand},
    },
    types::bit_string::BitString,
};
use quote::{format_ident, quote};
use rhdl_vlog::{self as vlog, vlog_expr_quote, vlog_item_quote};
use rtl::spec as tl;

type Result<T> = std::result::Result<T, RHDLError>;

struct TranslationContext<'a> {
    func: vlog::FunctionDef,
    rtl: &'a rtl::Object,
}

impl From<&TypedBits> for vlog::LitVerilog {
    fn from(tb: &TypedBits) -> Self {
        let bits = "'b"
            .chars()
            .chain(tb.bits.iter().map(|b| match b {
                BitX::Zero => '0',
                BitX::One => '1',
                BitX::X => 'X',
            }))
            .collect::<String>();
        vlog::lit_verilog(tb.bits.len() as u32, &bits)
    }
}

impl TranslationContext<'_> {
    fn raise_ice(&self, cause: ICE, id: SourceLocation) -> RHDLError {
        rhdl_error(RHDLCompileError {
            cause,
            src: self.rtl.symbols.source(),
            err_span: self.rtl.symbols.span(id).into(),
        })
    }
    fn add_item(&mut self, item: vlog::Item) {
        self.func.items.push(item);
    }
    fn op_ident(&self, op: Operand) -> syn::Ident {
        format_ident!("{}", self.rtl.op_name(op))
    }
    /// Cast the argument ot the desired width, considering the result a signed value.
    /// The cast length must be less than or equal to the argument length, or an ICE is raised.
    fn translate_as_signed(&mut self, cast: &tl::Cast, id: SourceLocation) -> Result<()> {
        if cast.len > self.rtl.kind(cast.arg).bits() {
            return Err(self.raise_ice(
                ICE::InvalidSignedCast {
                    lhs: cast.lhs,
                    arg: cast.arg,
                    len: cast.len,
                },
                id,
            ));
        }
        let lhs = self.op_ident(cast.lhs);
        let arg = self.op_ident(cast.arg);
        let msb = syn::Index::from(cast.len - 1);
        self.add_item(vlog_item_quote! { #lhs[#msb:0] = $signed(#arg) });
        Ok(())
    }
    /// Cast the argument to the desired width, with no error and no sign extension
    fn translate_resize_unsigned(&mut self, cast: &tl::Cast) {
        let arg_kind = self.rtl.kind(cast.arg);
        let lhs = self.op_ident(cast.lhs);
        let arg = self.op_ident(cast.arg);
        let msb = syn::Index::from(cast.len - 1);
        // Truncation case
        if cast.len <= arg_kind.bits() {
            self.add_item(vlog_item_quote! { #lhs = #arg[#msb:0] });
        } else {
            // zero extend
            let num_z = syn::Index::from(cast.len - arg_kind.bits());
            let prefix = vlog_expr_quote!( { #num_z { 1'b0 } }  );
            self.add_item(vlog_item_quote! { #lhs = { #prefix, #arg } });
        }
    }
    /// Cast the argument to the desired width, with sign extension if needed.
    fn translate_resize_signed(&mut self, cast: &tl::Cast) {
        let arg_kind = self.rtl.kind(cast.arg);
        let lhs = self.op_ident(cast.lhs);
        let arg = self.op_ident(cast.arg);
        // Truncation case
        if cast.len <= arg_kind.bits() {
            // lhs = $signed(arg[cast.len:0])
            let msb = syn::Index::from(cast.len - 1);
            self.add_item(vlog_item_quote! { #lhs = $signed(#arg[#msb:0]) });
        } else {
            // sign extend
            let num_s = syn::Index::from(cast.len - arg_kind.bits());
            let sign_bit = syn::Index::from(arg_kind.bits().saturating_sub(1));
            // #lhs = $signed({ {#num_s{arg[#sign_bit]}}, #arg })
            self.add_item(vlog_item_quote! { #lhs = $signed({ {#num_s{#arg[#sign_bit]}}, #arg }) });
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
            CastKind::Unsigned => {
                self.translate_resize_unsigned(cast);
                Ok(())
            }
            CastKind::Resize => self.translate_resize(cast, id),
        }
    }
    fn translate_assign(&mut self, assign: &tl::Assign) -> Result<()> {
        // #lhs = #rhs
        let lhs = self.op_ident(assign.lhs);
        let rhs = self.op_ident(assign.rhs);
        self.add_item(vlog_item_quote! { #lhs = #rhs });
        Ok(())
    }
    fn translate_binary(&mut self, binary: &tl::Binary) -> Result<()> {
        let lhs = self.op_ident(binary.lhs);
        let arg1 = self.op_ident(binary.arg1);
        let arg2 = self.op_ident(binary.arg2);
        let op = match binary.op {
            tl::AluBinary::Add => vlog::kw_ops::BinaryOp::Plus,
            tl::AluBinary::Sub => vlog::kw_ops::BinaryOp::Minus,
            tl::AluBinary::Mul => vlog::kw_ops::BinaryOp::Mul,
            tl::AluBinary::BitXor => vlog::kw_ops::BinaryOp::Xor,
            tl::AluBinary::BitAnd => vlog::kw_ops::BinaryOp::And,
            tl::AluBinary::BitOr => vlog::kw_ops::BinaryOp::Or,
            tl::AluBinary::Shl => vlog::kw_ops::BinaryOp::Shl,
            tl::AluBinary::Shr => vlog::kw_ops::BinaryOp::SignedRightShift,
            tl::AluBinary::Eq => vlog::kw_ops::BinaryOp::Eq,
            tl::AluBinary::Lt => vlog::kw_ops::BinaryOp::Lt,
            tl::AluBinary::Le => vlog::kw_ops::BinaryOp::Le,
            tl::AluBinary::Ne => vlog::kw_ops::BinaryOp::Ne,
            tl::AluBinary::Ge => vlog::kw_ops::BinaryOp::Ge,
            tl::AluBinary::Gt => vlog::kw_ops::BinaryOp::Gt,
        };
        self.add_item(vlog_item_quote! { #lhs = #arg1 #op #arg2 });
        Ok(())
    }
    fn translate_case(&mut self, case: &tl::Case) -> Result<()> {
        let discriminant = self.op_ident(case.discriminant);
        let lhs = self.op_ident(case.lhs);
        let table = case.table.iter().map(|(arg, value)| {
            let value = self.op_ident(*value);
            match arg {
                CaseArgument::Literal(lit) => {
                    let lit = self.rtl.symtab[lit].into();
                    quote! { #lit : #lhs = #value }
                }
                CaseArgument::Wild => quote! { default : #lhs = #value },
            }
        });
        self.add_item(vlog_item_quote! {
            case (#discriminant)
                #(#table;)*
            endcase
        });
        Ok(())
    }
    fn translate_concat(&mut self, concat: &tl::Concat) -> Result<()> {
        let args = concat.args.iter().rev().map(|arg| self.op_ident(*arg));
        let lhs = self.op_ident(concat.lhs);
        self.add_item(vlog_item_quote! { #lhs = { #(#args),* } });
        Ok(())
    }
    fn translate_index(&mut self, index: &tl::Index) -> Result<()> {
        let lhs = self.op_ident(index.lhs);
        let arg = self.op_ident(index.arg);
        let range: vlog::BitRange = (&index.bit_range).into();
        self.add_item(vlog_item_quote! { #lhs = #arg[#range] });
        Ok(())
    }
    fn translate_select(&mut self, select: &tl::Select) -> Result<()> {
        let lhs = self.op_ident(select.lhs);
        let cond = self.op_ident(select.cond);
        let true_value = self.op_ident(select.true_value);
        let false_value = self.op_ident(select.false_value);
        self.add_item(vlog_item_quote! {
            #lhs = #cond ? #true_value : #false_value
        });
        Ok(())
    }
    fn translate_splice(&mut self, splice: &tl::Splice) -> Result<()> {
        let lhs = self.op_ident(splice.lhs);
        let orig = self.op_ident(splice.orig);
        let value = self.op_ident(splice.value);
        let range: vlog::BitRange = (&splice.bit_range).into();
        self.add_item(vlog_item_quote! { #lhs = #orig });
        self.add_item(vlog_item_quote! { #lhs[#range] = #value });
        Ok(())
    }
    fn translate_unary(&mut self, unary: &tl::Unary) -> Result<()> {
        let lhs = self.op_ident(unary.lhs);
        let arg1 = self.op_ident(unary.arg1);
        self.add_item(match unary.op {
            AluUnary::Neg => vlog_item_quote! {#lhs = -#arg1},
            AluUnary::Not => vlog_item_quote! {#lhs = ~#arg1},
            AluUnary::All => vlog_item_quote! {#lhs = &#arg1},
            AluUnary::Any => vlog_item_quote! {#lhs = |#arg1|},
            AluUnary::Xor => vlog_item_quote! {#lhs = ^#arg1},
            AluUnary::Signed => vlog_item_quote! {#lhs = $signed(#arg1)},
            AluUnary::Unsigned => vlog_item_quote! {#lhs = $unsigned(#arg1)},
            AluUnary::Val => vlog_item_quote! {#lhs = #arg1},
        });
        Ok(())
    }
    fn translate_comment(&mut self, comment: &str) -> Result<()> {
        // TODO - FIXME
        //self.func.block.push(ast::comment(comment));
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
            .map(|(ndx, reg)| input_reg(&format!("arg_{ndx}"), self.rtl.symtab[&reg].into()))
            .collect::<Vec<_>>();
        self.func.arguments = arg_decls;
        let reg_decls = self
            .rtl
            .symtab
            .iter_reg()
            .map(|(rid, (kind, _))| {
                let alias = self.rtl.op_alias(Operand::Register(rid));
                declaration(
                    HDLKind::Reg,
                    &self.op_ident(Operand::Register(rid)),
                    kind.into(),
                    alias,
                )
            })
            .collect::<Vec<_>>();
        self.func.registers = reg_decls;
        let literals = self
            .rtl
            .symtab
            .iter_lit()
            .map(|(lit, (tb, _))| {
                let bs: BitString = tb.into();
                literal(&self.op_ident(Operand::Literal(lit)), &bs)
            })
            .collect::<Vec<_>>();
        self.func.literals = literals;
        // Bind the argument registers
        for (ndx, reg) in self.rtl.arguments.iter().enumerate() {
            if let Some(reg) = reg {
                let reg_name = self.op_ident(Operand::Register(*reg));
                let arg_name = format!("arg_{ndx}");
                self.func.block.push(assign(&reg_name, id(&arg_name)));
            }
        }
        self.translate_block(&self.rtl.ops)?;
        self.func.block.push(assign(
            &self.func.name,
            id(&self.op_ident(self.rtl.return_register)),
        ));
        Ok(self.func)
    }
}

fn translate(object: &crate::rtl::Object) -> Result<Function> {
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

pub fn generate_verilog(object: &crate::rtl::Object) -> Result<Function> {
    translate(object)
}
