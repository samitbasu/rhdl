use std::collections::BTreeSet;

use crate::kernel::ExternalKernelDef;
use crate::path::{bit_range, Path, PathElement};
use crate::rhif::spec::{
    AluBinary, AluUnary, Array, Assign, Binary, Case, CaseArgument, Cast, Discriminant, Enum, Exec,
    ExternalFunctionCode, Index, Member, OpCode, Repeat, Select, Slot, Splice, Struct, Tuple,
    Unary,
};
use crate::test_module::VerilogDescriptor;
use crate::util::binary_string;
use crate::{ast::ast_impl::FunctionId, rhif::Object, Design, TypedBits};
use anyhow::Result;
use anyhow::{anyhow, ensure};

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
    body: &'a mut String,
    kernels: Vec<VerilogModule>,
    design: &'a Design,
    obj: &'a Object,
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
    fn compute_dynamic_index_expression(&self, target: &Slot, path: &Path) -> Result<String> {
        ensure!(path.any_dynamic());
        // Collect the list of dynamic index registers
        let dynamic_slots: Vec<Slot> = path.dynamic_slots().copied().collect();
        // First, to get the base offset, we construct a path that
        // replaces all dynamic indices with 0
        let arg_kind = self
            .obj
            .kind
            .get(target)
            .ok_or(anyhow!(
                "No type for slot {} in function {}",
                target,
                self.obj.name
            ))?
            .clone();
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
            ensure!(
                slot_range.1 == base_range.1,
                "Mismatched types arise from dynamic indexing! ICE"
            );
            ensure!(
                slot_range.0.len() == base_range.0.len(),
                "Mismatched bit widths arise from dynamic indexing! ICE"
            );
        }
        let base_offset = base_range.0.start;
        let base_length = base_range.0.len();
        let slot_strides = slot_ranges
            .iter()
            .map(|x| x.0.start - base_range.0.start)
            .collect::<Vec<_>>();
        let indexing_expression = dynamic_slots
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
        ensure!(path.any_dynamic());
        let index_expression = self.compute_dynamic_index_expression(orig, path)?;
        self.body.push_str(&format!(
            "    {lhs} = {orig};\n    {lhs}[{index_expression}] = {subst};\n"
        ));
        Ok(())
    }

    fn translate_dynamic_index(&mut self, lhs: &Slot, arg: &Slot, path: &Path) -> Result<()> {
        ensure!(path.any_dynamic());
        let index_expression = self.compute_dynamic_index_expression(arg, path)?;
        self.body
            .push_str(&format!("    {lhs} = {arg}[{index_expression}];\n",));
        Ok(())
    }

    fn translate_index(&mut self, lhs: &Slot, arg: &Slot, path: &Path) -> Result<()> {
        ensure!(!path.any_dynamic());
        let arg_ty = self
            .obj
            .kind
            .get(arg)
            .ok_or(anyhow!(
                "No type for slot {} in function {}",
                arg,
                self.obj.name
            ))?
            .clone();
        let (bit_range, _) = bit_range(arg_ty, path)?;
        self.body.push_str(&format!(
            "    {lhs} = {arg}[{}:{}];\n",
            bit_range.end - 1,
            bit_range.start
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
        ensure!(!path.any_dynamic());
        let orig_ty = self
            .obj
            .kind
            .get(orig)
            .ok_or(anyhow!(
                "No type for slot {} in function {}",
                orig,
                self.obj.name
            ))?
            .clone();
        let (bit_range, _) = bit_range(orig_ty, path)?;
        self.body.push_str(&format!(
            "     {lhs} = {orig};\n    {lhs}[{}:{}] = {subst};\n",
            bit_range.end - 1,
            bit_range.start,
        ));
        Ok(())
    }

    fn translate_op(&mut self, op: &OpCode) -> Result<()> {
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
                    op = verilog_binop(op)
                ));
            }
            OpCode::Unary(Unary { op, lhs, arg1 }) => {
                self.body.push_str(&format!(
                    "    {lhs} = {op}({arg1});\n",
                    op = verilog_unop(op)
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
                self.body.push_str(&format!("    {lhs} = {rhs};\n",));
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
                    "    {lhs} = {{ {} }};\n",
                    fields
                        .iter()
                        .rev()
                        .filter(|x| !x.is_empty())
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            OpCode::Array(Array { lhs, elements }) => {
                self.body.push_str(&format!(
                    "    {lhs} = {{ {} }};\n",
                    elements
                        .iter()
                        .rev()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
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
                    rest.to_string()
                } else {
                    as_verilog_literal(template)
                };
                // Assign LHS = initial
                self.body.push_str(&format!("    {lhs} = {};\n", initial));
                // Now assign each of the fields
                let kind = template.kind.clone();
                for field in fields {
                    let path = match &field.member {
                        Member::Unnamed(ndx) => Path::default().index(*ndx as usize),
                        Member::Named(name) => Path::default().field(name),
                    };
                    let (bit_range, _) = bit_range(kind.clone(), &path)?;
                    self.body.push_str(&format!(
                        "    {lhs}[{}:{}] = {};\n",
                        bit_range.end - 1,
                        bit_range.start,
                        field.value
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
                self.body.push_str(&format!("    {lhs} = {};\n", initial));
                // Now assign each of the fields
                let kind = template.kind.clone();
                for field in fields {
                    let base_path =
                        Path::default().payload_by_value(template.discriminant()?.as_i64()?);
                    let path = match &field.member {
                        Member::Unnamed(ndx) => base_path.index(*ndx as usize),
                        Member::Named(name) => base_path.field(name),
                    };
                    let (bit_range, _) = bit_range(kind.clone(), &path)?;
                    self.body.push_str(&format!(
                        "    {lhs}[{}:{}] = {};\n",
                        bit_range.end - 1,
                        bit_range.start,
                        field.value
                    ));
                }
            }
            OpCode::Discriminant(Discriminant { lhs, arg }) => {
                let arg_ty = self
                    .obj
                    .kind
                    .get(arg)
                    .ok_or(anyhow!(
                        "No type for slot {} in function {}",
                        arg,
                        self.obj.name
                    ))?
                    .clone();
                let path = Path::default().discriminant();
                let (bit_range, _) = bit_range(arg_ty, &path)?;
                self.body.push_str(&format!(
                    "    {lhs} = {arg}[{}:{}];\n",
                    bit_range.end - 1,
                    bit_range.start,
                ));
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
                    .push_str(&format!("    case ({})\n", discriminant));
                for (cond, slot) in table {
                    match cond {
                        CaseArgument::Constant(c) => {
                            self.body
                                .push_str(&format!("      {}: ", as_verilog_literal(c)));
                            self.body.push_str(&format!("{} = {};\n", lhs, slot));
                        }
                        CaseArgument::Wild => {
                            self.body.push_str("      default: ");
                            self.body.push_str(&format!("{} = {};", lhs, slot));
                        }
                    }
                }
                self.body.push_str("    endcase\n");
            }
            OpCode::Exec(Exec { lhs, id, args }) => {
                let func = &self.obj.externals[id.0];
                let args = args
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                match &func.code {
                    ExternalFunctionCode::Kernel(kernel) => {
                        let func_name = self.design.func_name(kernel.inner().fn_id)?;
                        let kernel = translate(self.design, kernel.inner().fn_id)?;
                        self.kernels.push(kernel);
                        self.body
                            .push_str(&format!("    {lhs} = {func_name}({args});\n"));
                    }
                    ExternalFunctionCode::Extern(ExternalKernelDef {
                        name,
                        body,
                        vm_stub: _,
                    }) => {
                        self.body
                            .push_str(&format!("    {lhs} = {name}({args});\n"));
                        self.kernels.push(VerilogModule {
                            functions: vec![body.clone()],
                        });
                    }
                }
            }
            OpCode::Repeat(Repeat { lhs, value, len }) => {
                self.body
                    .push_str(&format!("    {lhs} = {{ {len} {{ {value} }} }};\n"));
            }
            OpCode::AsBits(Cast { lhs, arg, len }) => {
                self.body
                    .push_str(&format!("    {lhs} = {arg}[{}:0];\n", len - 1));
            }
            OpCode::AsSigned(Cast { lhs, arg, len }) => {
                self.body
                    .push_str(&format!("    {lhs} = $signed({arg}[{}:0]);\n", len - 1));
            }
        }
        Ok(())
    }

    fn translate_block(&mut self, block: &[OpCode]) -> Result<()> {
        for op in block {
            self.translate_op(op)?;
        }
        Ok(())
    }
}

fn translate(design: &Design, fn_id: FunctionId) -> Result<VerilogModule> {
    let obj = design
        .objects
        .get(&fn_id)
        .ok_or(anyhow::anyhow!("Function {fn_id} not found"))?;
    // Determine the sizes of the arguments
    let arg_decls = obj
        .arguments
        .iter()
        .enumerate()
        .map(|(ndx, a)| {
            if a.is_empty() {
                Ok(format!("__empty{}", ndx))
            } else {
                decl(a, obj)
            }
        })
        .collect::<Result<Vec<_>>>()?
        .iter()
        .map(|x| format!("input {}", x))
        .collect::<Vec<_>>();
    let ret_ty = obj.kind.get(&obj.return_slot).ok_or(anyhow!(
        "No type for return slot {} in function {fn_id}",
        obj.return_slot
    ))?;
    let ret_size = ret_ty.bits();
    let ret_signed = if ret_ty.is_signed() { "signed" } else { "" };
    if ret_size == 0 {
        return Err(anyhow!("Function {fn_id} has no return value"));
    }
    let func_name = design.func_name(fn_id)?;
    let mut func = format!(
        "\nfunction {ret_signed} [{}:0] {}({});\n",
        ret_size - 1,
        func_name,
        arg_decls.join(", "),
    );
    func.push_str("    // Registers\n");
    for reg in obj
        .kind
        .keys()
        .filter(|x| !obj.arguments.contains(x))
        .filter(|x| x.is_reg())
    {
        func.push_str(&format!("    {};\n", decl(reg, obj)?));
    }
    func.push_str("    // Literals\n");
    // Allocate the literals
    for (i, lit) in obj.literals.iter().enumerate() {
        func.push_str(&format!(
            "    localparam l{i} = {};\n",
            as_verilog_literal(lit)
        ));
    }
    func.push_str("    // Body\n");
    func.push_str("begin\n");
    let kernels = {
        let mut context = TranslationContext {
            kernels: Vec::new(),
            body: &mut func,
            design,
            obj,
        };
        context.translate_block(&obj.ops)?;
        context.kernels
    };
    func.push_str(&format!("    {} = {};\n", func_name, obj.return_slot));
    func.push_str("end\n");
    func.push_str("endfunction\n");
    let mut module = VerilogModule::default();
    for kernel in kernels {
        module.functions.extend(kernel.functions);
    }
    module.functions.push(func);
    Ok(module)
}

pub fn as_verilog_literal(tb: &TypedBits) -> String {
    let signed = if tb.kind.is_signed() { "s" } else { "" };
    let width = tb.bits.len();
    format!("{}'{}b{}", width, signed, binary_string(&tb.bits))
}

fn decl(slot: &Slot, obj: &Object) -> Result<String> {
    let ty = obj
        .kind
        .get(slot)
        .ok_or(anyhow!("No type for slot {}", slot))?;
    let signed = if ty.is_signed() { "signed" } else { "" };
    let width = ty.bits();
    Ok(format!("reg {} [{}:0] r{}", signed, width - 1, slot.reg()?))
}

pub fn generate_verilog(design: &Design) -> Result<VerilogDescriptor> {
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
