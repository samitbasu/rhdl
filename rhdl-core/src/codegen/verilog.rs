use std::collections::BTreeSet;

use crate::digital::binary_string;
use crate::kernel::ExternalKernelDef;
use crate::path::{bit_range, Path, PathElement};
use crate::rhif::{
    AluBinary, AluUnary, Array, Assign, Binary, BlockId, Case, CaseArgument, Cast, Discriminant,
    Enum, Exec, If, Index, Member, OpCode, Repeat, Slot, Struct, Tuple, Unary,
};
use crate::test_module::VerilogDescriptor;
use crate::{ast::FunctionId, design::Design, object::Object, rhif::Block, TypedBits};
use crate::{KernelFnKind, Kind};
use anyhow::{anyhow, ensure};
use anyhow::{bail, Result};

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
    blocks: &'a [Block],
    design: &'a Design,
    obj: &'a Object,
    early_return_encountered: bool,
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
        let arg_kind: Kind = self
            .obj
            .ty
            .get(target)
            .ok_or(anyhow!(
                "No type for slot {} in function {}",
                target,
                self.obj.name
            ))?
            .clone()
            .try_into()?;
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

    fn translate_dynamic_assign(&mut self, lhs: &Slot, rhs: &Slot, path: &Path) -> Result<()> {
        ensure!(path.any_dynamic());
        let index_expression = self.compute_dynamic_index_expression(lhs, path)?;
        self.body
            .push_str(&format!("    {lhs}[{index_expression}] = {rhs};\n"));
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
            .ty
            .get(arg)
            .ok_or(anyhow!(
                "No type for slot {} in function {}",
                arg,
                self.obj.name
            ))?
            .clone();
        let arg_kind: Kind = arg_ty.try_into()?;
        let (bit_range, _) = bit_range(arg_kind, path)?;
        self.body.push_str(&format!(
            "    {lhs} = {arg}[{}:{}];\n",
            bit_range.end - 1,
            bit_range.start
        ));
        Ok(())
    }

    fn translate_assign(&mut self, lhs: &Slot, rhs: &Slot, path: &Path) -> Result<()> {
        ensure!(!path.any_dynamic());
        let lhs_ty = self
            .obj
            .ty
            .get(lhs)
            .ok_or(anyhow!(
                "No type for slot {} in function {}",
                lhs,
                self.obj.name
            ))?
            .clone();
        let lhs_kind: Kind = lhs_ty.try_into()?;
        let (bit_range, _) = bit_range(lhs_kind, path)?;
        self.body.push_str(&format!(
            "    {lhs}[{}:{}] = {rhs};\n",
            bit_range.end - 1,
            bit_range.start,
        ));
        Ok(())
    }

    fn translate_op(&mut self, op: &OpCode) -> Result<()> {
        match op {
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
            OpCode::If(If {
                lhs: _,
                cond,
                then_branch,
                else_branch,
            }) => {
                self.body.push_str(&format!("    if ({cond})\n"));
                self.translate_block(*then_branch)?;
                self.body.push_str("    else\n");
                self.translate_block(*else_branch)?;
            }
            OpCode::Block(block_id) => {
                self.translate_block(*block_id)?;
            }
            OpCode::Index(Index { lhs, arg, path }) => {
                if path.any_dynamic() {
                    self.translate_dynamic_index(lhs, arg, path)?;
                } else {
                    self.translate_index(lhs, arg, path)?;
                }
            }
            OpCode::Assign(Assign { lhs, rhs, path }) => {
                if path.any_dynamic() {
                    self.translate_dynamic_assign(lhs, rhs, path)?;
                } else {
                    self.translate_assign(lhs, rhs, path)?;
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
                    .ty
                    .get(arg)
                    .ok_or(anyhow!(
                        "No type for slot {} in function {}",
                        arg,
                        self.obj.name
                    ))?
                    .clone();
                let arg_kind: Kind = arg_ty.try_into()?;
                let path = Path::default().discriminant();
                let (bit_range, _) = bit_range(arg_kind, &path)?;
                self.body.push_str(&format!(
                    "    {lhs} = {arg}[{}:{}];\n",
                    bit_range.end - 1,
                    bit_range.start,
                ));
            }
            OpCode::Case(Case {
                discriminant,
                table,
            }) => {
                self.body
                    .push_str(&format!("    case ({})\n", discriminant));
                for (cond, block) in table {
                    match cond {
                        CaseArgument::Constant(c) => {
                            self.body
                                .push_str(&format!("      {}: ", as_verilog_literal(c)));
                            self.translate_block(*block)?;
                        }
                        CaseArgument::Wild => {
                            self.body.push_str("      default: ");
                            self.translate_block(*block)?;
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
                    KernelFnKind::Kernel(kernel) => {
                        let func_name = self.design.func_name(kernel.fn_id)?;
                        let kernel = translate(self.design, kernel.fn_id)?;
                        self.kernels.push(kernel);
                        self.body
                            .push_str(&format!("    {lhs} = {func_name}({args});\n"));
                    }
                    KernelFnKind::Extern(ExternalKernelDef {
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
                    _ => todo!("Translate non-kernel functions"),
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
            OpCode::Return => {
                self.body.push_str("    __abort = 1;\n");
                self.early_return_encountered = true;
            }
            _ => todo!("{op:?} is not implemented yet"),
        }
        Ok(())
    }

    fn translate_block(&mut self, block: BlockId) -> Result<()> {
        self.body.push_str("    begin\n");
        let block = self
            .blocks
            .get(block.0)
            .ok_or(anyhow!("Block {} not found", block.0))?;
        for op in &block.ops {
            if self.early_return_encountered && !matches!(op, OpCode::Comment(_)) {
                self.body.push_str(" if (!__abort)\n");
                self.body.push_str("    begin\n");
                self.translate_op(op)?;
                self.body.push_str("    end\n");
            } else {
                self.translate_op(op)?;
            }
        }
        self.body.push_str("    end\n");
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
        .map(|a| decl(a, obj))
        .collect::<Result<Vec<_>>>()?
        .iter()
        .map(|x| format!("input {}", x))
        .collect::<Vec<_>>();
    let ret_ty = obj.ty.get(&obj.return_slot).ok_or(anyhow!(
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
    // Allocate the registers
    let max_reg = obj.reg_count() + 1;
    // Skip the arguments..
    let start = obj.arguments.len();
    func.push_str("    // Registers\n");
    for reg in start..max_reg {
        func.push_str(&format!("    {};\n", decl(&Slot::Register(reg), obj)?));
    }
    func.push_str("    // Literals\n");
    // Allocate the literals
    for (i, lit) in obj.literals.iter().enumerate() {
        func.push_str(&format!(
            "    localparam l{i} = {};\n",
            as_verilog_literal(lit)
        ));
    }
    func.push_str("    // Early return flag\n");
    func.push_str("    reg __abort;\n");
    func.push_str("    // Body\n");
    func.push_str("begin\n");
    func.push_str("    __abort = 0;\n");
    let kernels = {
        let mut context = TranslationContext {
            kernels: Vec::new(),
            body: &mut func,
            blocks: &obj.blocks,
            design,
            obj,
            early_return_encountered: false,
        };
        context.translate_block(obj.main_block)?;
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

fn as_verilog_literal(tb: &TypedBits) -> String {
    let signed = if tb.kind.is_signed() { "s" } else { "" };
    let width = tb.bits.len();
    format!("{}'{}b{}", width, signed, binary_string(&tb.bits))
}

fn decl(slot: &Slot, obj: &Object) -> Result<String> {
    let ty = obj
        .ty
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
