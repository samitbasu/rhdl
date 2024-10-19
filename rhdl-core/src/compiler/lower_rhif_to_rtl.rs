use std::collections::BTreeMap;
use std::iter::once;

use crate::ast::ast_impl::{FunctionId, NodeId, WrapOp};
use crate::error::rhdl_error;
use crate::rhif::spec::{AluBinary, Slot};
use crate::rtl::object::{lop, RegisterKind};
use crate::rtl::remap::remap_operands;
use crate::rtl::spec::{CastKind, Concat, LiteralId, Operand, RegisterId};
use crate::rtl::symbols::SymbolMap;
use crate::types::bit_string::BitString;
use crate::types::path::{bit_range, Path};
use crate::util::clog2;
use crate::Digital;
use crate::TypedBits;
use crate::{rhif, RHDLError};
use crate::{rtl, Kind};

use crate::rhif::spec as hf;
use crate::rtl::spec as tl;

use crate::compiler::mir::error::{RHDLCompileError, ICE};

type Result<T> = std::result::Result<T, RHDLError>;

struct RTLCompiler<'a> {
    symbols: SymbolMap,
    object: &'a rhif::object::Object,
    literals: BTreeMap<LiteralId, BitString>,
    registers: BTreeMap<RegisterId, RegisterKind>,
    operand_map: BTreeMap<Operand, (FunctionId, Slot)>,
    reverse_operand_map: BTreeMap<(FunctionId, Slot), Operand>,
    ops: Vec<rtl::object::LocatedOpCode>,
    literal_count: usize,
    register_count: usize,
}

impl<'a> RTLCompiler<'a> {
    fn new(object: &'a rhif::object::Object) -> Self {
        let mut symbols = SymbolMap::default();
        symbols
            .source_set
            .extend(once((object.fn_id, object.symbols.source.clone())));
        Self {
            object,
            symbols,
            literals: Default::default(),
            registers: Default::default(),
            operand_map: Default::default(),
            reverse_operand_map: Default::default(),
            ops: Default::default(),
            literal_count: 0,
            register_count: 0,
        }
    }
    fn associate(&mut self, operand: Operand, id: NodeId) {
        self.symbols
            .operand_map
            .insert(operand, (self.object.fn_id, id).into());
    }
    fn allocate_literal(&mut self, bits: &TypedBits, id: NodeId) -> Operand {
        let literal_id = LiteralId::new(self.literal_count);
        self.literal_count += 1;
        self.literals.insert(literal_id, bits.into());
        let literal = Operand::Literal(literal_id);
        self.associate(literal, id);
        literal
    }
    fn allocate_literal_from_bit_string(&mut self, bits: &BitString, id: NodeId) -> Operand {
        let literal_id = LiteralId::new(self.literal_count);
        self.literal_count += 1;
        self.literals.insert(literal_id, bits.clone());
        let literal = Operand::Literal(literal_id);
        self.associate(literal, id);
        literal
    }
    fn allocate_literal_from_usize_and_length(
        &mut self,
        value: usize,
        bits: usize,
        id: NodeId,
    ) -> Result<Operand> {
        let value = value as u64;
        let value: TypedBits = value.into();
        let value = value.unsigned_cast(bits)?;
        Ok(self.allocate_literal(&value, id))
    }
    fn allocate_signed(&mut self, length: usize, id: NodeId) -> Operand {
        let register_id = RegisterId::new(self.register_count);
        self.register_count += 1;
        self.registers
            .insert(register_id, RegisterKind::Signed(length));
        let register = Operand::Register(register_id);
        self.associate(register, id);
        register
    }
    fn allocate_unsigned(&mut self, length: usize, id: NodeId) -> Operand {
        let register_id = RegisterId::new(self.register_count);
        self.register_count += 1;
        self.registers
            .insert(register_id, RegisterKind::Unsigned(length));
        let register = Operand::Register(register_id);
        self.associate(register, id);
        register
    }
    fn allocate_register(&mut self, kind: &Kind, id: NodeId) -> Operand {
        let len = kind.bits();
        if kind.is_signed() {
            self.allocate_signed(len, id)
        } else {
            self.allocate_unsigned(len, id)
        }
    }
    fn allocate_register_with_register_kind(&mut self, kind: &RegisterKind, id: NodeId) -> Operand {
        let register_id = RegisterId::new(self.register_count);
        self.register_count += 1;
        self.registers.insert(register_id, *kind);
        let register = Operand::Register(register_id);
        self.associate(register, id);
        register
    }
    fn raise_ice(&self, cause: ICE, node_id: NodeId) -> RHDLError {
        rhdl_error(RHDLCompileError {
            cause,
            src: self.object.symbols.source.source.clone(),
            err_span: self.object.symbols.node_span(node_id).into(),
        })
    }
    fn lop(&mut self, opcode: tl::OpCode, id: NodeId) {
        self.ops.push((opcode, id, self.object.fn_id).into())
    }
    fn build_dynamic_index(
        &mut self,
        arg: Slot,
        path: &Path,
        node_id: NodeId,
    ) -> Result<(Operand, usize)> {
        let dynamic_slots: Vec<Slot> = path.dynamic_slots().copied().collect();
        // First to get the base offset, we construct a path that replaces all
        // dynamic indices with 0
        let arg_kind = self.object.kind(arg);
        let base_path = path.zero_out_dynamic_indices();
        let (base_range, base_kind) = bit_range(arg_kind.clone(), &base_path)?;
        // Next for each index register, we compute a range where only that index
        // is advanced by one.
        let (slot_ranges, slot_kinds) = dynamic_slots
            .iter()
            .map(|slot| {
                let stride_path = path.stride_path(*slot);
                bit_range(arg_kind.clone(), &stride_path)
            })
            .collect::<Result<(Vec<_>, Vec<_>)>>()?;
        // Validation - all of the kinds should be the same
        if let Some(kind) = slot_kinds.iter().find(|kind| **kind != base_kind) {
            return Err(self.raise_ice(
                ICE::MismatchedTypesFromDynamicIndexing {
                    base: base_kind,
                    slot: kind.clone(),
                },
                node_id,
            ));
        }
        // Next validation - all of the bit ranges should be the same size
        if let Some(range) = slot_ranges
            .iter()
            .find(|range| range.len() != base_range.len())
        {
            return Err(self.raise_ice(
                ICE::MismatchedBitWidthsFromDynamicIndexing {
                    base: base_range.len(),
                    slot: range.len(),
                },
                node_id,
            ));
        }
        let slot_strides = slot_ranges
            .iter()
            .map(|x| x.start - base_range.start)
            .collect::<Vec<_>>();
        // The indexing expression is of the form
        // offset + (dynamic_value) +: len
        // We need to compute the number of bits needed to hold
        // dynamic_value.
        let index_bits = clog2(arg_kind.bits());
        // Compute the series of partial products p_k = i_k * s_k,
        // where i_k <- slot_k as L, and s_k <- stride_k as L, where
        // L is the number of bits needed to index the array.
        // Sums these partial products into a single index register
        // The sum is initialized with the offset of the base index
        let base_offset = base_range.start;
        let mut index_sum =
            self.allocate_literal_from_usize_and_length(base_offset, index_bits, node_id)?;
        for (slot, stride) in dynamic_slots.iter().zip(slot_strides.iter()) {
            // Store the stride in a literal
            let stride =
                self.allocate_literal_from_usize_and_length(*stride, index_bits, node_id)?;
            let reg = self.allocate_register(&Kind::make_bits(index_bits), node_id);
            let arg = self.operand(*slot, node_id)?;
            // reg <- arg as L
            self.lop(
                tl::OpCode::Cast(tl::Cast {
                    lhs: reg,
                    arg,
                    len: index_bits,
                    kind: CastKind::Unsigned,
                }),
                node_id,
            );
            let prod_uncast = self.allocate_register(&Kind::make_bits(index_bits * 2), node_id);
            self.lop(
                tl::OpCode::Binary(tl::Binary {
                    lhs: prod_uncast,
                    op: AluBinary::Mul,
                    arg1: reg,
                    arg2: stride,
                }),
                node_id,
            );
            let prod = self.allocate_register(&Kind::make_bits(index_bits), node_id);
            // prod <- (reg * stride) as L
            self.lop(
                tl::OpCode::Cast(tl::Cast {
                    lhs: prod,
                    arg: prod_uncast,
                    len: index_bits,
                    kind: CastKind::Unsigned,
                }),
                node_id,
            );
            let sum = self.allocate_register(&Kind::make_bits(index_bits), node_id);
            // sum <- sum + prod
            self.lop(
                tl::OpCode::Binary(tl::Binary {
                    lhs: sum,
                    op: AluBinary::Add,
                    arg1: index_sum,
                    arg2: prod,
                }),
                node_id,
            );
            index_sum = sum;
        }
        Ok((index_sum, base_range.len()))
    }
    fn operand(&mut self, slot: Slot, id: NodeId) -> Result<Operand> {
        if let Some(operand) = self.reverse_operand_map.get(&(self.object.fn_id, slot)) {
            return Ok(*operand);
        }
        match slot {
            Slot::Literal(literal_id) => {
                let bits = &self.object.literals[&literal_id];
                let operand = self.allocate_literal(bits, id);
                let node = self.object.symbols.slot_map[&slot];
                self.symbols
                    .operand_map
                    .insert(operand, (self.object.fn_id, node).into());
                self.reverse_operand_map
                    .insert((self.object.fn_id, slot), operand);
                self.operand_map.insert(operand, (self.object.fn_id, slot));
                Ok(operand)
            }
            Slot::Register(register_id) => {
                let kind = &self.object.kind[&register_id];
                let operand = self.allocate_register(kind, id);
                let node = self.object.symbols.slot_map[&slot];
                self.symbols
                    .operand_map
                    .insert(operand, (self.object.fn_id, node).into());
                self.reverse_operand_map
                    .insert((self.object.fn_id, slot), operand);
                self.operand_map.insert(operand, (self.object.fn_id, slot));
                Ok(operand)
            }
            Slot::Empty => panic!("empty slot"), //Err(self.raise_ice(ICE::EmptySlotInRTL, id)),
        }
    }
    fn make_operand_list(&mut self, args: &[Slot], id: NodeId) -> Result<Vec<Operand>> {
        args.iter()
            .filter_map(|a| {
                if a.is_empty() {
                    None
                } else {
                    Some(self.operand(*a, id))
                }
            })
            .collect()
    }
    fn make_array(&mut self, args: &hf::Array, id: NodeId) -> Result<()> {
        let hf::Array { lhs, elements } = args;
        if lhs.is_empty() {
            return Ok(());
        }
        let lhs = self.operand(*lhs, id)?;
        let elements = self.make_operand_list(elements, id)?;
        self.lop(
            tl::OpCode::Concat(tl::Concat {
                lhs,
                args: elements,
            }),
            id,
        );
        Ok(())
    }
    fn make_resize(&mut self, cast: &hf::Cast, id: NodeId) -> Result<()> {
        let hf::Cast { lhs, arg, len } = cast;
        let len = len.ok_or_else(|| self.raise_ice(ICE::BitCastMissingRequiredLength, id))?;
        if !lhs.is_empty() && !arg.is_empty() {
            let lhs = self.operand(*lhs, id)?;
            let op_arg = self.operand(*arg, id)?;
            self.lop(
                tl::OpCode::Cast(tl::Cast {
                    lhs,
                    arg: op_arg,
                    len,
                    kind: CastKind::Resize,
                }),
                id,
            );
        }
        Ok(())
    }
    fn make_wrap(&mut self, wrap: &hf::Wrap, id: NodeId) -> Result<()> {
        let hf::Wrap { lhs, op, arg, kind } = wrap;
        let kind = kind
            .clone()
            .ok_or_else(|| self.raise_ice(ICE::WrapMissingKind, id))?;
        let discriminant = match op {
            WrapOp::Ok | WrapOp::Some => self.allocate_literal(&true.typed_bits(), id),
            WrapOp::Err | WrapOp::None => self.allocate_literal(&false.typed_bits(), id),
        };
        let width = kind.bits() - 1;
        let lhs = self.operand(*lhs, id)?;
        if width != 0 {
            let payload =
                self.allocate_register_with_register_kind(&RegisterKind::Unsigned(width), id);
            let arg = *arg;
            let arg = match arg {
                Slot::Empty => self.allocate_literal(&false.typed_bits(), id),
                _ => self.operand(arg, id)?,
            };
            self.lop(
                tl::OpCode::Cast(tl::Cast {
                    lhs: payload,
                    arg,
                    len: width,
                    kind: CastKind::Resize,
                }),
                id,
            );
            self.lop(
                tl::OpCode::Concat(Concat {
                    lhs,
                    args: vec![payload, discriminant],
                }),
                id,
            );
        } else {
            self.lop(
                tl::OpCode::Assign(tl::Assign {
                    lhs,
                    rhs: discriminant,
                }),
                id,
            );
        };
        Ok(())
    }
    fn make_as_bits(&mut self, cast: &hf::Cast, id: NodeId) -> Result<()> {
        let hf::Cast { lhs, arg, len } = cast;
        let len = len.ok_or_else(|| self.raise_ice(ICE::BitCastMissingRequiredLength, id))?;
        if !lhs.is_empty() && !arg.is_empty() {
            let lhs = self.operand(*lhs, id)?;
            let arg = self.operand(*arg, id)?;
            self.lop(
                tl::OpCode::Cast(tl::Cast {
                    lhs,
                    arg,
                    len,
                    kind: CastKind::Unsigned,
                }),
                id,
            );
        }
        Ok(())
    }
    fn make_as_signed(&mut self, cast: &hf::Cast, id: NodeId) -> Result<()> {
        let hf::Cast { lhs, arg, len } = cast;
        let len = len.ok_or_else(|| self.raise_ice(ICE::BitCastMissingRequiredLength, id))?;
        if !lhs.is_empty() && !arg.is_empty() {
            let lhs = self.operand(*lhs, id)?;
            let arg = self.operand(*arg, id)?;
            self.lop(
                tl::OpCode::Cast(tl::Cast {
                    lhs,
                    arg,
                    len,
                    kind: CastKind::Signed,
                }),
                id,
            );
        }
        Ok(())
    }
    fn make_assign(&mut self, assign: &hf::Assign, id: NodeId) -> Result<()> {
        let hf::Assign { lhs, rhs } = assign;
        if !lhs.is_empty() && !rhs.is_empty() {
            let lhs = self.operand(*lhs, id)?;
            let rhs = self.operand(*rhs, id)?;
            self.lop(tl::OpCode::Assign(tl::Assign { lhs, rhs }), id);
        }
        Ok(())
    }
    fn make_binary(&mut self, binary: &hf::Binary, id: NodeId) -> Result<()> {
        let hf::Binary {
            lhs,
            op,
            arg1,
            arg2,
        } = *binary;
        if !lhs.is_empty() {
            let lhs = self.operand(lhs, id)?;
            let arg1 = self.operand(arg1, id)?;
            let arg2 = self.operand(arg2, id)?;
            self.lop(
                tl::OpCode::Binary(tl::Binary {
                    lhs,
                    op,
                    arg1,
                    arg2,
                }),
                id,
            );
        }
        Ok(())
    }
    fn make_case_argument(
        &mut self,
        case_argument: &hf::CaseArgument,
        id: NodeId,
    ) -> Result<tl::CaseArgument> {
        match case_argument {
            hf::CaseArgument::Slot(slot) => {
                if !slot.is_literal() {
                    return Err(self.raise_ice(ICE::MatchPatternValueMustBeLiteral, id));
                };
                let operand = self.operand(*slot, id)?;
                let Operand::Literal(literal_id) = operand else {
                    return Err(self.raise_ice(ICE::MatchPatternValueMustBeLiteral, id));
                };
                Ok(tl::CaseArgument::Literal(literal_id))
            }
            hf::CaseArgument::Wild => Ok(tl::CaseArgument::Wild),
        }
    }
    fn make_case(&mut self, case: &hf::Case, id: NodeId) -> Result<()> {
        let hf::Case {
            lhs,
            discriminant,
            table,
        } = case;
        if lhs.is_empty() {
            return Ok(());
        }
        let lhs = self.operand(*lhs, id)?;
        let discriminant = self.operand(*discriminant, id)?;
        let table = table
            .iter()
            .map(|(cond, val)| {
                let cond = self.make_case_argument(cond, id)?;
                let val = self.operand(*val, id)?;
                Ok((cond, val))
            })
            .collect::<Result<Vec<_>>>()?;
        self.lop(
            tl::OpCode::Case(tl::Case {
                lhs,
                discriminant,
                table,
            }),
            id,
        );
        Ok(())
    }
    fn make_dynamic_splice(&mut self, splice: &hf::Splice, node_id: NodeId) -> Result<()> {
        let hf::Splice {
            lhs,
            orig,
            path,
            subst,
        } = splice;
        if lhs.is_empty() {
            return Ok(());
        }
        let (index_reg, len) = self.build_dynamic_index(*orig, path, node_id)?;
        let lhs = self.operand(*lhs, node_id)?;
        let orig = self.operand(*orig, node_id)?;
        let subst = self.operand(*subst, node_id)?;
        self.lop(
            tl::OpCode::DynamicSplice(tl::DynamicSplice {
                lhs,
                arg: orig,
                offset: index_reg,
                len,
                value: subst,
            }),
            node_id,
        );
        Ok(())
    }
    fn make_enum(&mut self, enumerate: &hf::Enum, id: NodeId) -> Result<()> {
        let hf::Enum {
            lhs,
            fields,
            template,
        } = enumerate;
        if lhs.is_empty() {
            return Ok(());
        }
        let kind = template.kind.clone();
        let discriminant = template.discriminant()?.as_i64()?;
        let mut rhs = self.allocate_literal(template, id);
        for field in fields {
            let field_value = self.operand(field.value, id)?;
            let path = Path::default()
                .payload_by_value(discriminant)
                .member(&field.member);
            let (field_range, _) = bit_range(kind.clone(), &path)?;
            let reg = self.allocate_register(&kind, id);
            self.lop(
                tl::OpCode::Splice(tl::Splice {
                    lhs: reg,
                    orig: rhs,
                    bit_range: field_range,
                    value: field_value,
                }),
                id,
            );
            rhs = reg;
        }
        let lhs = self.operand(*lhs, id)?;
        self.lop(tl::OpCode::Assign(tl::Assign { lhs, rhs }), id);
        Ok(())
    }
    fn make_exec(&mut self, exec: &hf::Exec, node_id: NodeId) -> Result<()> {
        let hf::Exec { lhs, id, args } = exec;
        if lhs.is_empty() {
            return Ok(());
        }
        // Look up the function ID from the external functions.
        let func = &self.object.externals[id];
        // Compile it...
        let func_rtl = compile_rtl(func)?;
        // Inline it.
        let mut operand_translation = BTreeMap::new();
        // Rebind the arguments to local registers, and copy the values into them
        for (fn_arg, arg) in func_rtl.arguments.iter().zip(args) {
            if let Some(fn_reg) = fn_arg {
                let fn_reg_in_our_space = self
                    .allocate_register_with_register_kind(&func_rtl.register_kind[fn_reg], node_id);
                operand_translation.insert(Operand::Register(*fn_reg), fn_reg_in_our_space);
                let arg = self.operand(*arg, node_id)?;
                self.lop(
                    tl::OpCode::Assign(tl::Assign {
                        lhs: fn_reg_in_our_space,
                        rhs: arg,
                    }),
                    node_id,
                );
            }
        }
        let mut op_remap = |operand| {
            if operand_translation.contains_key(&operand) {
                return operand_translation[&operand];
            }
            let new_operand = match operand {
                Operand::Literal(old_lit_id) => {
                    let old_lit = func_rtl.literals[&old_lit_id].clone();
                    self.allocate_literal_from_bit_string(&old_lit, node_id)
                }
                Operand::Register(old_reg_id) => {
                    let kind = func_rtl.register_kind[&old_reg_id];
                    self.allocate_register_with_register_kind(&kind, node_id)
                }
            };
            operand_translation.insert(operand, new_operand);
            new_operand
        };
        let return_register = op_remap(func_rtl.return_register);
        // Translate each operation and add it to the existing function (inline).
        // Remap the operands of the opcode to allocate from the current function.
        // Note that we need to ensure that if a register is allocated it is reused..
        let translated = func_rtl
            .ops
            .into_iter()
            .map(|old_lop| {
                let op = remap_operands(old_lop.op, &mut op_remap);
                lop(op, old_lop.loc.node, old_lop.loc.func)
            })
            .collect::<Vec<_>>();
        self.ops.extend(translated);
        let lhs = self.operand(*lhs, node_id)?;
        self.lop(
            tl::OpCode::Assign(tl::Assign {
                lhs,
                rhs: return_register,
            }),
            node_id,
        );
        for old_op in func_rtl.symbols.operand_map.keys() {
            let new_op = operand_translation[old_op];
            let loc = func_rtl.symbols.operand_map[old_op];
            self.symbols.operand_map.insert(new_op, loc);
            if let Some(name) = func_rtl.symbols.operand_names.get(old_op) {
                self.symbols.operand_names.insert(new_op, name.clone());
            }
        }
        self.symbols
            .source_set
            .extend(func_rtl.symbols.source_set.sources);
        Ok(())
    }
    fn make_dynamic_index(&mut self, index: &hf::Index, node_id: NodeId) -> Result<()> {
        let hf::Index { lhs, arg, path } = index;
        if lhs.is_empty() {
            return Ok(());
        }
        let (index_reg, len) = self.build_dynamic_index(*arg, path, node_id)?;
        let lhs = self.operand(*lhs, node_id)?;
        let arg = self.operand(*arg, node_id)?;
        self.lop(
            tl::OpCode::DynamicIndex(tl::DynamicIndex {
                lhs,
                arg,
                offset: index_reg,
                len,
            }),
            node_id,
        );
        Ok(())
    }
    fn make_index(&mut self, index: &hf::Index, node_id: NodeId) -> Result<()> {
        if index.lhs.is_empty() {
            return Ok(());
        }
        if index.path.any_dynamic() {
            return self.make_dynamic_index(index, node_id);
        }
        let arg_ty = self.object.kind(index.arg);
        let (bit_range, _) = bit_range(arg_ty, &index.path)?;
        let lhs = self.operand(index.lhs, node_id)?;
        let arg = self.operand(index.arg, node_id)?;
        self.lop(
            tl::OpCode::Index(tl::Index {
                lhs,
                arg,
                bit_range,
            }),
            node_id,
        );
        Ok(())
    }
    fn make_repeat(&mut self, repeat: &hf::Repeat, node_id: NodeId) -> Result<()> {
        let hf::Repeat { lhs, value, len } = *repeat;
        if lhs.is_empty() {
            return Ok(());
        }
        let lhs = self.operand(lhs, node_id)?;
        let arg = self.operand(value, node_id)?;
        let args = vec![arg; len as usize];
        self.lop(tl::OpCode::Concat(tl::Concat { lhs, args }), node_id);
        Ok(())
    }
    fn make_retime(&mut self, retime: &hf::Retime, node_id: NodeId) -> Result<()> {
        let hf::Retime { lhs, arg, color: _ } = *retime;
        if lhs.is_empty() {
            return Ok(());
        }
        let lhs = self.operand(lhs, node_id)?;
        let rhs = self.operand(arg, node_id)?;
        self.lop(tl::OpCode::Assign(tl::Assign { lhs, rhs }), node_id);
        Ok(())
    }
    fn make_select(&mut self, select: &hf::Select, node_id: NodeId) -> Result<()> {
        let hf::Select {
            lhs,
            cond,
            true_value,
            false_value,
        } = *select;
        if lhs.is_empty() {
            return Ok(());
        }
        let lhs = self.operand(lhs, node_id)?;
        let cond = self.operand(cond, node_id)?;
        let true_value = self.operand(true_value, node_id)?;
        let false_value = self.operand(false_value, node_id)?;
        self.lop(
            tl::OpCode::Select(tl::Select {
                lhs,
                cond,
                true_value,
                false_value,
            }),
            node_id,
        );
        Ok(())
    }
    fn make_splice(&mut self, splice: &hf::Splice, node_id: NodeId) -> Result<()> {
        if splice.lhs.is_empty() {
            return Ok(());
        }
        if splice.path.any_dynamic() {
            return self.make_dynamic_splice(splice, node_id);
        }
        let hf::Splice {
            lhs,
            orig,
            path,
            subst,
        } = splice;
        let orig_ty = self.object.kind(*orig);
        let (bit_range, _) = bit_range(orig_ty, path)?;
        let lhs = self.operand(*lhs, node_id)?;
        let orig = self.operand(*orig, node_id)?;
        let subst = self.operand(*subst, node_id)?;
        self.lop(
            tl::OpCode::Splice(tl::Splice {
                lhs,
                orig,
                bit_range,
                value: subst,
            }),
            node_id,
        );
        Ok(())
    }
    fn make_struct(&mut self, strukt: &hf::Struct, node_id: NodeId) -> Result<()> {
        let hf::Struct {
            lhs,
            fields,
            rest,
            template,
        } = strukt;
        if lhs.is_empty() {
            return Ok(());
        }
        let lhs = self.operand(*lhs, node_id)?;
        let kind = template.kind.clone();
        let mut rhs = if let Some(rest) = rest {
            self.operand(*rest, node_id)?
        } else {
            self.allocate_literal(template, node_id)
        };
        for field in fields {
            let field_value = self.operand(field.value, node_id)?;
            let path = Path::default().member(&field.member);
            let (field_range, _) = bit_range(kind.clone(), &path)?;
            let reg = self.allocate_register(&kind, node_id);
            self.lop(
                tl::OpCode::Splice(tl::Splice {
                    lhs: reg,
                    orig: rhs,
                    bit_range: field_range,
                    value: field_value,
                }),
                node_id,
            );
            rhs = reg;
        }
        self.lop(tl::OpCode::Assign(tl::Assign { lhs, rhs }), node_id);
        Ok(())
    }
    fn make_tuple(&mut self, tuple: &hf::Tuple, node_id: NodeId) -> Result<()> {
        let hf::Tuple { lhs, fields } = tuple;
        if lhs.is_empty() {
            return Ok(());
        }
        let lhs = self.operand(*lhs, node_id)?;
        let args = self.make_operand_list(fields, node_id)?;
        self.lop(tl::OpCode::Concat(tl::Concat { lhs, args }), node_id);
        Ok(())
    }
    fn make_unary(&mut self, unary: &hf::Unary, node_id: NodeId) -> Result<()> {
        let hf::Unary { lhs, op, arg1 } = *unary;
        if lhs.is_empty() {
            return Ok(());
        }
        let lhs = self.operand(lhs, node_id)?;
        let arg1 = self.operand(arg1, node_id)?;
        self.lop(tl::OpCode::Unary(tl::Unary { lhs, op, arg1 }), node_id);
        Ok(())
    }
    fn translate(mut self) -> Result<Self> {
        for lop in &self.object.ops {
            match &lop.op {
                hf::OpCode::Array(array) => {
                    self.make_array(array, lop.id)?;
                }
                hf::OpCode::AsBits(cast) => {
                    self.make_as_bits(cast, lop.id)?;
                }
                hf::OpCode::AsSigned(cast) => {
                    self.make_as_signed(cast, lop.id)?;
                }
                hf::OpCode::Assign(assign) => {
                    self.make_assign(assign, lop.id)?;
                }
                hf::OpCode::Binary(binary) => {
                    self.make_binary(binary, lop.id)?;
                }
                hf::OpCode::Case(case) => {
                    self.make_case(case, lop.id)?;
                }
                hf::OpCode::Comment(comment) => {
                    self.lop(tl::OpCode::Comment(comment.clone()), lop.id);
                }
                hf::OpCode::Enum(enumerate) => {
                    self.make_enum(enumerate, lop.id)?;
                }
                hf::OpCode::Exec(exec) => {
                    self.make_exec(exec, lop.id)?;
                }
                hf::OpCode::Index(index) => {
                    self.make_index(index, lop.id)?;
                }
                hf::OpCode::Noop => {}
                hf::OpCode::Resize(cast) => {
                    self.make_resize(cast, lop.id)?;
                }
                hf::OpCode::Repeat(repeat) => {
                    self.make_repeat(repeat, lop.id)?;
                }
                hf::OpCode::Retime(retime) => {
                    self.make_retime(retime, lop.id)?;
                }
                hf::OpCode::Select(select) => {
                    self.make_select(select, lop.id)?;
                }
                hf::OpCode::Splice(splice) => {
                    self.make_splice(splice, lop.id)?;
                }
                hf::OpCode::Struct(strukt) => {
                    self.make_struct(strukt, lop.id)?;
                }
                hf::OpCode::Tuple(tuple) => {
                    self.make_tuple(tuple, lop.id)?;
                }
                hf::OpCode::Unary(unary) => {
                    self.make_unary(unary, lop.id)?;
                }
                hf::OpCode::Wrap(wrap) => {
                    self.make_wrap(wrap, lop.id)?;
                }
            }
        }
        Ok(self)
    }
}

fn compile_rtl(object: &rhif::Object) -> Result<rtl::object::Object> {
    let mut compiler = RTLCompiler::new(object).translate()?;
    let arguments = object
        .arguments
        .iter()
        .map(|x| {
            if object.kind[x].is_empty() {
                None
            } else if let Ok(Operand::Register(reg_id)) =
                compiler.operand(Slot::Register(*x), object.symbols.source.fallback)
            {
                Some(reg_id)
            } else {
                None
            }
        })
        .collect();
    let return_register = compiler.reverse_operand_map[&(object.fn_id, object.return_slot)];
    Ok(rtl::object::Object {
        symbols: compiler.symbols,
        literals: compiler.literals,
        return_register,
        register_kind: compiler.registers,
        ops: compiler.ops,
        arguments,
        name: object.name.clone(),
        fn_id: object.fn_id,
    })
}

pub fn compile_to_rtl(object: &rhif::object::Object) -> Result<rtl::object::Object> {
    compile_rtl(object)
}
