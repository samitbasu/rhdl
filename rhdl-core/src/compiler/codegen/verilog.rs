use std::collections::BTreeSet;

use crate::{
    ast::ast_impl::{FunctionId, NodeId},
    compiler::mir::error::{RHDLCompileError, ICE},
    error::rhdl_error,
    rhif::spec::{AluBinary, AluUnary},
    rtl::{
        self,
        object::{LocatedOpCode, RegisterKind},
        spec::{CastKind, Operand},
    },
    test_module::VerilogDescriptor,
    types::bit_string::BitString,
    util::binary_string,
    RHDLError,
};

use rtl::spec as tl;

type Result<T> = std::result::Result<T, RHDLError>;

#[derive(Default, Clone, Debug)]
pub struct VerilogModule {
    pub functions: Vec<String>,
}
impl VerilogModule {
    fn deduplicate(self) -> Result<VerilogModule> {
        let functions: BTreeSet<String> = self.functions.into_iter().collect();
        let functions: Vec<String> = functions.into_iter().collect();
        Ok(VerilogModule { functions })
    }
}

struct TranslationContext<'a> {
    body: String,
    kernels: Vec<VerilogModule>,
    rtl: &'a rtl::Object,
    id: FunctionId,
}

// TODO - add check that len > 0 or redefine it as numbits - 1
fn reg_decl(name: &str, kind: RegisterKind) -> String {
    match kind {
        RegisterKind::Signed(len) => format!("reg signed [{}:0] {}", len - 1, name),
        RegisterKind::Unsigned(len) => format!("reg [{}:0] {}", len - 1, name),
    }
}

fn verilog_literal(bs: &BitString) -> String {
    let signed = if bs.is_signed() { "s" } else { "" };
    let width = bs.len();
    let bs = binary_string(bs.bits());
    format!("{width}'{signed}b{bs}")
}

fn verilog_binop(op: &AluBinary) -> &'static str {
    match op {
        AluBinary::Add => "+",
        AluBinary::Sub => "-",
        AluBinary::Mul => "*",
        AluBinary::BitAnd => "&",
        AluBinary::BitOr => "|",
        AluBinary::BitXor => "^",
        AluBinary::Shl => "<<",
        AluBinary::Shr => ">>>",
        AluBinary::Eq => "==",
        AluBinary::Ne => "!=",
        AluBinary::Lt => "<",
        AluBinary::Le => "<=",
        AluBinary::Gt => ">",
        AluBinary::Ge => ">=",
    }
}

fn verilog_unop(op: &AluUnary) -> &'static str {
    match op {
        AluUnary::Neg => "-",
        AluUnary::Not => "!",
        AluUnary::All => "&",
        AluUnary::Any => "|",
        AluUnary::Xor => "^",
        AluUnary::Signed => "$signed",
        AluUnary::Unsigned => "$unsigned",
        AluUnary::Val => "",
    }
}

impl<'a> TranslationContext<'a> {
    fn raise_ice(&self, cause: ICE, id: (FunctionId, NodeId)) -> RHDLError {
        rhdl_error(RHDLCompileError {
            cause,
            src: self.rtl.symbols[&id.0].source.source.clone(),
            err_span: self.rtl.symbols[&id.0].node_span(id.1).into(),
        })
    }
    fn translate_as_bits(&mut self, cast: &tl::Cast) -> Result<()> {
        // Check for the extension case
        let arg_kind = self.rtl.kind(cast.arg);
        if cast.len <= arg_kind.len() {
            self.body.push_str(&format!(
                "    {lhs} = {arg}[{len}:0];\n",
                lhs = self.rtl.op_name(cast.lhs),
                arg = self.rtl.op_name(cast.arg),
                len = cast.len - 1
            ));
        } else {
            // zero extend
            let num_z = cast.len - arg_kind.len();
            self.body.push_str(&format!(
                "    {lhs} = {{ {num_z}'b0, {arg} }};\n",
                lhs = self.rtl.op_name(cast.lhs),
                arg = self.rtl.op_name(cast.arg),
                num_z = num_z
            ));
        }
        Ok(())
    }
    fn translate_as_signed(&mut self, cast: &tl::Cast, id: (FunctionId, NodeId)) -> Result<()> {
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
        self.body.push_str(&format!(
            "    {lhs} = $signed({arg}[{len}:0]);\n",
            lhs = self.rtl.op_name(cast.lhs),
            arg = self.rtl.op_name(cast.arg),
            len = cast.len - 1
        ));
        Ok(())
    }
    fn translate_cast(&mut self, cast: &tl::Cast, id: (FunctionId, NodeId)) -> Result<()> {
        if matches!(cast.kind, CastKind::Signed) {
            self.translate_as_signed(cast, id)
        } else {
            self.translate_as_bits(cast)
        }
    }
    fn translate_assign(&mut self, assign: &tl::Assign) -> Result<()> {
        self.body.push_str(&format!(
            "    {lhs} = {rhs};\n",
            lhs = self.rtl.op_name(assign.lhs),
            rhs = self.rtl.op_name(assign.rhs)
        ));
        Ok(())
    }
    fn translate_binary(&mut self, binary: &tl::Binary) -> Result<()> {
        self.body.push_str(&format!(
            "    {lhs} = {arg1} {op} {arg2};\n",
            op = verilog_binop(&binary.op),
            lhs = self.rtl.op_name(binary.lhs),
            arg1 = self.rtl.op_name(binary.arg1),
            arg2 = self.rtl.op_name(binary.arg2)
        ));
        Ok(())
    }
    fn translate_case(&mut self, case: &tl::Case) -> Result<()> {
        self.body.push_str(&format!(
            "    case ({sel})\n",
            sel = self.rtl.op_name(case.discriminant)
        ));
        for (val, block) in &case.table {
            match val {
                tl::CaseArgument::Literal(lit) => {
                    self.body.push_str(&format!(
                        "      {}: ",
                        verilog_literal(&self.rtl.literals[lit])
                    ));
                    self.body.push_str(&format!(
                        "{} = {};\n",
                        self.rtl.op_name(case.lhs),
                        self.rtl.op_name(*block),
                    ));
                }
                tl::CaseArgument::Wild => {
                    self.body.push_str("      default: ");
                    self.body.push_str(&format!(
                        "{} = {};\n",
                        self.rtl.op_name(case.lhs),
                        self.rtl.op_name(*block),
                    ));
                }
            }
        }
        self.body.push_str("    endcase\n");
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
        self.body.push_str(&format!("    {lhs} = {{{args}}};\n",));
        Ok(())
    }
    fn translate_dynamic_index(&mut self, index: &tl::DynamicIndex) -> Result<()> {
        let lhs = self.rtl.op_name(index.lhs);
        let arg = self.rtl.op_name(index.arg);
        let offset = self.rtl.op_name(index.offset);
        let len = index.len;
        self.body
            .push_str(&format!("    {lhs} = {arg}[{offset} +: {len}];\n",));
        Ok(())
    }
    fn translate_dynamic_splice(&mut self, splice: &tl::DynamicSplice) -> Result<()> {
        let lhs = self.rtl.op_name(splice.lhs);
        let arg = self.rtl.op_name(splice.arg);
        let offset = self.rtl.op_name(splice.offset);
        let value = self.rtl.op_name(splice.value);
        let len = splice.len;
        self.body.push_str(&format!(
            "    {lhs} = {arg}; {lhs}[{offset} +: {len}] = {value};\n",
        ));
        Ok(())
    }
    fn translate_index(&mut self, index: &tl::Index) -> Result<()> {
        let lhs = self.rtl.op_name(index.lhs);
        let arg = self.rtl.op_name(index.arg);
        let start = index.bit_range.start;
        let end = index.bit_range.end - 1;
        self.body
            .push_str(&format!("    {lhs} = {arg}[{end}:{start}];\n",));
        Ok(())
    }
    fn translate_select(&mut self, select: &tl::Select) -> Result<()> {
        let lhs = self.rtl.op_name(select.lhs);
        let cond = self.rtl.op_name(select.cond);
        let true_value = self.rtl.op_name(select.true_value);
        let false_value = self.rtl.op_name(select.false_value);
        self.body.push_str(&format!(
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
        self.body.push_str(&format!(
            "    {lhs} = {orig}; {lhs}[{end}:{start}] = {value};\n",
        ));
        Ok(())
    }
    fn translate_unary(&mut self, unary: &tl::Unary) -> Result<()> {
        let lhs = self.rtl.op_name(unary.lhs);
        let arg1 = self.rtl.op_name(unary.arg1);
        match &unary.op {
            AluUnary::Signed => {
                self.body
                    .push_str(&format!("    {lhs} = $signed({arg1});\n",));
            }
            AluUnary::Unsigned => {
                self.body
                    .push_str(&format!("    {lhs} = $unsigned({arg1});\n",));
            }
            _ => {
                let op = verilog_unop(&unary.op);
                self.body.push_str(&format!("    {lhs} = {op}{arg1};\n",));
            }
        }
        Ok(())
    }
    fn translate_op(&mut self, lop: &LocatedOpCode) -> Result<()> {
        let op = &lop.op;
        match op {
            tl::OpCode::Assign(assign) => self.translate_assign(assign),
            tl::OpCode::Binary(binary) => self.translate_binary(binary),
            tl::OpCode::Case(case) => self.translate_case(case),
            tl::OpCode::Cast(cast) => self.translate_cast(cast, (self.id, lop.loc.node)),
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
            .filter_map(|arg| {
                if let Some(arg) = arg {
                    let kind = self.rtl.register_kind[arg];
                    Some(format!(
                        "input {}",
                        reg_decl(&self.rtl.op_name(Operand::Register(*arg)), kind)
                    ))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let ret_kind = self.rtl.kind(self.rtl.return_register);
        let signed = if ret_kind.is_signed() { "signed " } else { "" };
        let func_name = &self.rtl.name;
        self.body.push_str(&format!(
            "\nfunction {signed} [{}:0] {}({});\n",
            ret_kind.len() - 1,
            func_name,
            arg_decls.join(", ")
        ));
        self.body.push_str("   // Registers\n");
        for reg in self
            .rtl
            .register_kind
            .keys()
            .filter(|x| !self.rtl.arguments.contains(&Some(**x)))
        {
            let alias = self
                .rtl
                .op_alias(Operand::Register(*reg))
                .unwrap_or_default();
            self.body.push_str(&format!(
                "   {}; // {alias}\n",
                reg_decl(
                    &self.rtl.op_name(Operand::Register(*reg)),
                    self.rtl.register_kind[reg]
                )
            ));
        }
        self.body.push_str("    // Literals\n");
        for (lit, bs) in &self.rtl.literals {
            self.body.push_str(&format!(
                "    localparam {} = {};\n",
                self.rtl.op_name(Operand::Literal(*lit)),
                verilog_literal(bs)
            ));
        }
        self.body.push_str("    // Body\n");
        self.body.push_str("begin\n");
        self.translate_block(&self.rtl.ops)?;
        self.body.push_str(&format!(
            "    {} = {};\n",
            func_name,
            self.rtl.op_name(self.rtl.return_register)
        ));
        self.body.push_str("end\n");
        self.body.push_str("endfunction\n");
        let mut module = VerilogModule::default();
        for kernel in self.kernels {
            module.functions.extend(kernel.functions);
        }
        module.functions.push(self.body.to_string());
        Ok(module)
    }
}

fn translate(object: &crate::rtl::Object) -> Result<VerilogModule> {
    let context = TranslationContext {
        kernels: Default::default(),
        body: Default::default(),
        rtl: &object,
        id: object.fn_id,
    };
    context.translate_kernel_for_object()
}

pub fn generate_verilog(object: &crate::rtl::Object) -> Result<VerilogDescriptor> {
    let module = translate(object)?;
    let module = module.deduplicate()?;
    let body = module.functions.join("\n");
    Ok(VerilogDescriptor {
        name: object.name.clone(),
        body,
    })
}
