use std::collections::BTreeSet;

use crate::ast::ast_impl::NodeId;
use crate::compiler::mir::error::{RHDLCompileError, ICE};
use crate::error::RHDLError;
use crate::kernel::ExternalKernelDef;
use crate::path::{bit_range, Path, PathElement};
use crate::rhif::object::SourceLocation;
use crate::rhif::spec::{
    AluBinary, AluUnary, Array, Assign, Binary, Case, CaseArgument, Cast, Enum, Exec,
    ExternalFunctionCode, Index, Member, OpCode, Repeat, Retime, Select, Slot, Splice, Struct,
    Tuple, Unary,
};
use crate::test_module::VerilogDescriptor;
use crate::util::binary_string;
use crate::{ast::ast_impl::FunctionId, rhif::Object, Module, TypedBits};

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
    design: &'a Module,
    obj: &'a Object,
    id: FunctionId,
}

fn compute_base_offset_path(path: &Path) -> Path {
    Path {
        elements: path
            .elements
            .iter()
            .cloned()
            .map(|x| match x {
                PathElement::DynamicIndex(_) => PathElement::Index(0),
                _ => x,
            })
            .collect(),
    }
}

fn compute_stride_path_for_slot(path: &Path, slot: &Slot) -> Path {
    Path {
        elements: path
            .elements
            .iter()
            .map(|x| match x {
                PathElement::DynamicIndex(path_slot) => {
                    if path_slot == slot {
                        PathElement::Index(1)
                    } else {
                        PathElement::Index(0)
                    }
                }
                o => o.clone(),
            })
            .collect(),
    }
}

impl<'a> TranslationContext<'a> {
    fn raise_ice(&self, cause: ICE, id: NodeId) -> RHDLError {
        RHDLError::RHDLInternalCompilerError(Box::new(RHDLCompileError {
            cause,
            src: self.obj.symbols.source.source.clone(),
            err_span: self.obj.symbols.node_span(id).into(),
        }))
    }
    fn compute_dynamic_index_expression(&self, target: &Slot, path: &Path) -> Result<String> {
        if !path.any_dynamic() {
            return Err(self.raise_ice(
                ICE::PathDoesNotContainDynamicIndices { path: path.clone() },
                self.obj.symbols.slot_map[target].node,
            ));
        }
        // Collect the list of dynamic index registers
        let dynamic_slots: Vec<Slot> = path.dynamic_slots().copied().collect();
        // First, to get the base offset, we construct a path that
        // replaces all dynamic indices with 0
        let arg_kind = self.obj.kind[target].clone();
        let base_path = compute_base_offset_path(path);
        let base_range = bit_range(arg_kind.clone(), &base_path)?;
        // Next for each index register, we compute a range where only that index
        // is advanced by one.
        let slot_ranges = dynamic_slots
            .iter()
            .map(|slot| {
                let stride_path = compute_stride_path_for_slot(path, slot);
                bit_range(arg_kind.clone(), &stride_path)
            })
            .collect::<Result<Vec<_>>>()?;
        // Now for validation.  All of the kinds should be the same.
        for slot_range in &slot_ranges {
            if slot_range.1 != base_range.1 {
                return Err(self.raise_ice(
                    ICE::MismatchedTypesFromDynamicIndexing {
                        base: base_range.1.clone(),
                        slot: slot_range.1.clone(),
                    },
                    self.obj.symbols.slot_map[target].node,
                ));
            }
            if slot_range.0.len() != base_range.0.len() {
                return Err(self.raise_ice(
                    ICE::MismatchedBitWidthsFromDynamicIndexing {
                        base: base_range.0.len(),
                        slot: slot_range.0.len(),
                    },
                    self.obj.symbols.slot_map[target].node,
                ));
            }
        }
        let base_offset = base_range.0.start;
        let base_length = base_range.0.len();
        let slot_strides = slot_ranges
            .iter()
            .map(|x| x.0.start - base_range.0.start)
            .collect::<Vec<_>>();
        let dynamic_slot_names = dynamic_slots
            .iter()
            .map(|x| self.reg_name(x))
            .collect::<Result<Vec<_>>>()?;
        let indexing_expression = dynamic_slot_names
            .iter()
            .zip(slot_strides.iter())
            .map(|(slot, stride)| format!("({} * {})", slot, stride))
            .collect::<Vec<_>>()
            .join(" + ");
        Ok(format!(
            "({base_offset} + {indexing_expression}) +: {base_length}"
        ))
    }

    fn translate_dynamic_splice(
        &mut self,
        lhs: &Slot,
        orig: &Slot,
        path: &Path,
        subst: &Slot,
    ) -> Result<()> {
        if !path.any_dynamic() {
            return Err(self.raise_ice(
                ICE::PathDoesNotContainDynamicIndices { path: path.clone() },
                self.obj.symbols.slot_map[lhs].node,
            ));
        }
        let index_expression = self.compute_dynamic_index_expression(orig, path)?;
        self.body.push_str(&format!(
            "    {lhs} = {orig};\n    {lhs}[{index_expression}] = {subst};\n",
            lhs = self.reg_name(lhs)?,
            orig = self.reg_name(orig)?,
            index_expression = index_expression,
            subst = self.reg_name(subst)?
        ));
        Ok(())
    }

    fn translate_dynamic_index(&mut self, lhs: &Slot, arg: &Slot, path: &Path) -> Result<()> {
        if !path.any_dynamic() {
            return Err(self.raise_ice(
                ICE::PathDoesNotContainDynamicIndices { path: path.clone() },
                self.obj.symbols.slot_map[lhs].node,
            ));
        }
        let index_expression = self.compute_dynamic_index_expression(arg, path)?;
        self.body.push_str(&format!(
            "    {lhs} = {arg}[{index_expression}];\n",
            lhs = self.reg_name(lhs)?,
            arg = self.reg_name(arg)?,
            index_expression = index_expression,
        ));
        Ok(())
    }

    fn translate_index(&mut self, lhs: &Slot, arg: &Slot, path: &Path) -> Result<()> {
        if path.any_dynamic() {
            return Err(self.raise_ice(
                ICE::PathContainsDynamicIndices { path: path.clone() },
                self.obj.symbols.slot_map[lhs].node,
            ));
        }
        let arg_ty = self.obj.kind[arg].clone();
        let (bit_range, _) = bit_range(arg_ty, path)?;
        self.body.push_str(&format!(
            "    {lhs} = {arg}[{end}:{start}];\n",
            lhs = self.reg_name(lhs)?,
            arg = self.reg_name(arg)?,
            end = bit_range.end - 1,
            start = bit_range.start
        ));
        Ok(())
    }

    fn translate_splice(
        &mut self,
        lhs: &Slot,
        orig: &Slot,
        path: &Path,
        subst: &Slot,
    ) -> Result<()> {
        if path.any_dynamic() {
            return Err(self.raise_ice(
                ICE::PathContainsDynamicIndices { path: path.clone() },
                self.obj.symbols.slot_map[lhs].node,
            ));
        }
        let orig_ty = self.obj.kind[orig].clone();
        let (bit_range, _) = bit_range(orig_ty, path)?;
        self.body.push_str(&format!(
            "     {lhs} = {orig};\n    {lhs}[{end}:{start}] = {subst};\n",
            lhs = self.reg_name(lhs)?,
            orig = self.reg_name(orig)?,
            subst = self.reg_name(subst)?,
            end = bit_range.end - 1,
            start = bit_range.start,
        ));
        Ok(())
    }

    fn translate_op(&mut self, op: &OpCode, op_id: NodeId) -> Result<()> {
        eprintln!("Verilog translate of {op:?}");
        match op {
            OpCode::Noop => {}
            OpCode::Binary(Binary {
                op,
                lhs,
                arg1,
                arg2,
            }) => {
                self.body.push_str(&format!(
                    "    {lhs} = {arg1} {op} {arg2};\n",
                    op = verilog_binop(op),
                    lhs = self.reg_name(lhs)?,
                    arg1 = self.reg_name(arg1)?,
                    arg2 = self.reg_name(arg2)?,
                ));
            }
            OpCode::Unary(Unary { op, lhs, arg1 }) => {
                self.body.push_str(&format!(
                    "    {lhs} = {op}({arg1});\n",
                    op = verilog_unop(op),
                    lhs = self.reg_name(lhs)?,
                    arg1 = self.reg_name(arg1)?,
                ));
            }
            OpCode::Select(Select {
                lhs,
                cond,
                true_value,
                false_value,
            }) => {
                if !lhs.is_empty() {
                    self.body.push_str(&format!(
                        "    {lhs} = {cond} ? {true_value} : {false_value};\n",
                        lhs = self.reg_name(lhs)?,
                        cond = self.reg_name(cond)?,
                        true_value = self.reg_name(true_value)?,
                        false_value = self.reg_name(false_value)?,
                    ));
                }
            }
            OpCode::Index(Index { lhs, arg, path }) => {
                if path.any_dynamic() {
                    self.translate_dynamic_index(lhs, arg, path)?;
                } else {
                    self.translate_index(lhs, arg, path)?;
                }
            }
            OpCode::Assign(Assign { lhs, rhs }) => {
                self.body.push_str(&format!(
                    "    {lhs} = {rhs};\n",
                    lhs = self.reg_name(lhs)?,
                    rhs = self.reg_name(rhs)?
                ));
            }
            OpCode::Splice(Splice {
                lhs,
                orig,
                path,
                subst,
            }) => {
                if path.any_dynamic() {
                    self.translate_dynamic_splice(lhs, orig, path, subst)?;
                } else {
                    self.translate_splice(lhs, orig, path, subst)?;
                }
            }
            OpCode::Comment(s) => {
                self.body.push_str(&format!(
                    "    // {}\n",
                    s.trim_end().replace('\n', "\n    // ")
                ));
            }
            OpCode::Tuple(Tuple { lhs, fields }) => {
                self.body.push_str(&format!(
                    "    {lhs} = {{ {fields} }};\n",
                    lhs = self.reg_name(lhs)?,
                    fields = fields
                        .iter()
                        .rev()
                        .filter(|x| !x.is_empty())
                        .map(|x| self.reg_name(x))
                        .collect::<Result<Vec<_>>>()?
                        .join(", ")
                ));
            }
            OpCode::Array(Array { lhs, elements }) => {
                self.body.push_str(&format!(
                    "    {lhs} = {{ {elements} }};\n",
                    lhs = self.reg_name(lhs)?,
                    elements = elements
                        .iter()
                        .rev()
                        .map(|x| self.reg_name(x))
                        .collect::<Result<Vec<_>>>()?
                        .join(", ")
                ));
            }
            OpCode::Struct(Struct {
                lhs,
                fields,
                rest,
                template,
            }) => {
                let initial = if let Some(rest) = rest {
                    self.reg_name(rest)?
                } else {
                    as_verilog_literal(template)
                };
                // Assign LHS = initial
                self.body.push_str(&format!(
                    "    {lhs} = {initial};\n",
                    lhs = self.reg_name(lhs)?,
                    initial = initial
                ));
                // Now assign each of the fields
                let kind = template.kind.clone();
                for field in fields {
                    let path = match &field.member {
                        Member::Unnamed(ndx) => Path::default().tuple_index(*ndx as usize),
                        Member::Named(name) => Path::default().field(name),
                    };
                    let (bit_range, _) = bit_range(kind.clone(), &path)?;
                    self.body.push_str(&format!(
                        "    {lhs}[{end}:{start}] = {field};\n",
                        lhs = self.reg_name(lhs)?,
                        end = bit_range.end - 1,
                        start = bit_range.start,
                        field = self.reg_name(&field.value)?
                    ));
                }
            }
            OpCode::Enum(Enum {
                lhs,
                fields,
                template,
            }) => {
                let initial = as_verilog_literal(template);
                // Assign LHS = initial
                self.body.push_str(&format!(
                    "    {lhs} = {initial};\n",
                    lhs = self.reg_name(lhs)?,
                    initial = initial
                ));
                // Now assign each of the fields
                let kind = template.kind.clone();
                for field in fields {
                    let base_path =
                        Path::default().payload_by_value(template.discriminant()?.as_i64()?);
                    let path = match &field.member {
                        Member::Unnamed(ndx) => base_path.tuple_index(*ndx as usize),
                        Member::Named(name) => base_path.field(name),
                    };
                    let (bit_range, _) = bit_range(kind.clone(), &path)?;
                    self.body.push_str(&format!(
                        "    {lhs}[{end}:{start}] = {field};\n",
                        lhs = self.reg_name(lhs)?,
                        end = bit_range.end - 1,
                        start = bit_range.start,
                        field = self.reg_name(&field.value)?
                    ));
                }
            }
            OpCode::Case(Case {
                lhs,
                discriminant,
                table,
            }) => {
                if lhs.is_empty() {
                    return Ok(());
                }
                self.body
                    .push_str(&format!("    case ({})\n", self.reg_name(discriminant)?));
                for (cond, slot) in table {
                    match cond {
                        CaseArgument::Slot(s) => {
                            let s = self.obj.literal(*s)?;
                            self.body
                                .push_str(&format!("      {}: ", as_verilog_literal(s)));
                            self.body.push_str(&format!(
                                "{} = {};\n",
                                self.reg_name(lhs)?,
                                self.reg_name(slot)?
                            ));
                        }
                        CaseArgument::Wild => {
                            self.body.push_str("      default: ");
                            self.body.push_str(&format!(
                                "{} = {};\n",
                                self.reg_name(lhs)?,
                                self.reg_name(slot)?
                            ));
                        }
                    }
                }
                self.body.push_str("    endcase\n");
            }
            OpCode::Exec(Exec { lhs, id, args }) => {
                let func = &self.obj.externals[id.0];
                let args = args
                    .iter()
                    .map(|x| self.reg_name(x))
                    .collect::<Result<Vec<_>>>()?
                    .join(", ");
                match &func.code {
                    ExternalFunctionCode::Kernel(kernel) => {
                        let func_name = self.design.func_name(kernel.inner().fn_id)?;
                        let kernel = translate(self.design, kernel.inner().fn_id)?;
                        self.kernels.push(kernel);
                        self.body.push_str(&format!(
                            "    {lhs} = {func_name}({args});\n",
                            lhs = self.reg_name(lhs)?,
                            func_name = func_name,
                            args = args
                        ));
                    }
                    ExternalFunctionCode::Extern(ExternalKernelDef {
                        name,
                        body,
                        vm_stub: _,
                    }) => {
                        self.body.push_str(&format!(
                            "    {lhs} = {name}({args});\n",
                            lhs = self.reg_name(lhs)?,
                            name = name,
                            args = args
                        ));
                        self.kernels.push(VerilogModule {
                            functions: vec![body.clone()],
                        });
                    }
                }
            }
            OpCode::Repeat(Repeat { lhs, value, len }) => {
                self.body.push_str(&format!(
                    "    {lhs} = {{ {len} {{ {value} }} }};\n",
                    lhs = self.reg_name(lhs)?,
                    len = len,
                    value = self.reg_name(value)?,
                ));
            }
            OpCode::AsBits(Cast { lhs, arg, len }) => {
                let len = len.ok_or(self.raise_ice(ICE::BitCastMissingRequiredLength, op_id))?;
                self.body.push_str(&format!(
                    "    {lhs} = {arg}[{len}:0];\n",
                    lhs = self.reg_name(lhs)?,
                    arg = self.reg_name(arg)?,
                    len = len - 1
                ));
            }
            OpCode::AsSigned(Cast { lhs, arg, len }) => {
                let len = len.ok_or(self.raise_ice(ICE::BitCastMissingRequiredLength, op_id))?;
                self.body.push_str(&format!(
                    "    {lhs} = $signed({arg}[{len}:0]);\n",
                    lhs = self.reg_name(lhs)?,
                    arg = self.reg_name(arg)?,
                    len = len - 1
                ));
            }
            OpCode::Retime(Retime { lhs, arg, color: _ }) => {
                self.body.push_str(&format!(
                    "    {lhs} = {arg};\n",
                    lhs = self.reg_name(lhs)?,
                    arg = self.reg_name(arg)?
                ));
            }
        }
        Ok(())
    }

    fn translate_block(&mut self, block: &[OpCode], ids: &[SourceLocation]) -> Result<()> {
        for (op, id) in block.iter().zip(ids.iter()) {
            self.translate_op(op, id.node)?;
        }
        Ok(())
    }

    fn reg_name(&self, slot: &Slot) -> Result<String> {
        let slot_alias = self.obj.symbols.slot_names.get(slot);
        let root = match slot {
            Slot::Empty => {
                return Err(self.raise_ice(
                    ICE::EmptySlotInVerilog,
                    self.obj.symbols.slot_map[slot].node,
                ))
            }
            Slot::Register(x) => format!("r{x}"),
            Slot::Literal(x) => format!("l{x}"),
        };
        if let Some(slot_alias) = slot_alias {
            Ok(format!("{}_{}", root, slot_alias))
        } else {
            Ok(root)
        }
    }

    fn decl(&self, slot: &Slot) -> Result<String> {
        let ty = &self.obj.kind[slot];
        let signed = if ty.is_signed() { "signed" } else { "" };
        let width = ty.bits();
        Ok(format!(
            "reg {} [{}:0] {}",
            signed,
            width.saturating_sub(1),
            self.reg_name(slot)?
        ))
    }

    fn translate_kernel_for_object(mut self) -> Result<VerilogModule> {
        // Determine the sizes of the arguments
        let fn_id = self.id;
        let arg_decls = self
            .obj
            .arguments
            .iter()
            .enumerate()
            .map(|(ndx, a)| {
                if a.is_empty() {
                    Ok(format!("__empty{}", ndx))
                } else {
                    self.decl(a)
                }
            })
            .collect::<Result<Vec<_>>>()?
            .iter()
            .map(|x| format!("input {}", x))
            .collect::<Vec<_>>();
        let ret_ty = &self.obj.kind[&self.obj.return_slot];
        let ret_size = ret_ty.bits();
        let ret_signed = if ret_ty.is_signed() { "signed" } else { "" };
        if ret_size == 0 {
            return Err(self.raise_ice(
                ICE::FunctionWithNoReturnInVerilog,
                self.obj.symbols.slot_map[&self.obj.return_slot].node,
            ));
        }
        let func_name = self.design.func_name(fn_id)?;
        self.body.push_str(&format!(
            "\nfunction {ret_signed} [{}:0] {}({});\n",
            ret_size - 1,
            func_name,
            arg_decls.join(", "),
        ));
        self.body.push_str("    // Registers\n");
        for reg in self
            .obj
            .kind
            .keys()
            .filter(|x| !self.obj.arguments.contains(x))
            .filter(|x| x.is_reg())
        {
            self.body.push_str(&format!("    {};\n", self.decl(reg)?));
        }
        self.body.push_str("    // Literals\n");
        // Allocate the literals
        for (&slot, lit) in self.obj.literals.iter() {
            if lit.bits.is_empty() {
                continue;
            }
            if let Slot::Literal(i) = slot {
                self.body.push_str(&format!(
                    "    localparam {} = {};\n",
                    self.reg_name(&Slot::Literal(i))?,
                    as_verilog_literal(lit)
                ));
            }
        }
        self.body.push_str("    // Body\n");
        self.body.push_str("begin\n");
        self.translate_block(&self.obj.ops, &self.obj.symbols.opcode_map)?;
        let kernels = self.kernels.clone();
        self.body.push_str(&format!(
            "    {} = {};\n",
            func_name,
            self.reg_name(&self.obj.return_slot)?
        ));
        self.body.push_str("end\n");
        self.body.push_str("endfunction\n");
        let mut module = VerilogModule::default();
        for kernel in kernels {
            module.functions.extend(kernel.functions);
        }
        module.functions.push(self.body.to_string());
        Ok(module)
    }
}

fn translate(design: &Module, fn_id: FunctionId) -> Result<VerilogModule> {
    let obj = design
        .objects
        .get(&fn_id)
        .ok_or(anyhow::anyhow!("Function {fn_id:?} not found"))?;
    let context = TranslationContext {
        kernels: Vec::new(),
        body: Default::default(),
        design,
        obj,
        id: fn_id,
    };
    context.translate_kernel_for_object()
}

pub fn as_verilog_literal(tb: &TypedBits) -> String {
    let signed = if tb.kind.is_signed() { "s" } else { "" };
    let width = tb.bits.len();
    format!("{}'{}b{}", width, signed, binary_string(&tb.bits))
}

pub fn generate_verilog(design: &Module) -> Result<VerilogDescriptor> {
    let module = translate(design, design.top)?;
    let module = module.deduplicate()?;
    let body = module.functions.join("\n");
    Ok(VerilogDescriptor {
        name: design.func_name(design.top)?,
        body,
    })
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
    }
}
