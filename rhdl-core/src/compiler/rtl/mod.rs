use std::collections::BTreeMap;

use crate::ast::ast_impl::NodeId;
use crate::rhif::spec::{AluBinary, ExternalFunction, FuncId, Slot};
use crate::rtl::object::{BitString, RegisterKind};
use crate::rtl::spec::{LiteralId, Operand, RegisterId};
use crate::types::path::{bit_range, Path};
use crate::TypedBits;
use crate::{rhif, RHDLError};
use crate::{rtl, Kind};

use crate::rhif::spec as hf;
use crate::rtl::spec as tl;

use super::mir::error::{RHDLCompileError, ICE};

type Result<T> = std::result::Result<T, RHDLError>;

fn clog2(x: usize) -> usize {
    if x == 0 {
        0
    } else {
        (x as f64).log2().ceil() as usize
    }
}

#[test]
fn test_clog2() {
    assert_eq!(clog2(0), 0);
    assert_eq!(clog2(1), 0);
    assert_eq!(clog2(2), 1);
    assert_eq!(clog2(3), 2);
    assert_eq!(clog2(255), 8);
}

struct RTLCompiler<'a> {
    object: &'a rhif::object::Object,
    literals: BTreeMap<LiteralId, BitString>,
    registers: BTreeMap<RegisterId, RegisterKind>,
    operand_map: BTreeMap<Operand, Slot>,
    reverse_operand_map: BTreeMap<Slot, Operand>,
    externals: BTreeMap<FuncId, ExternalFunction>,
    ops: Vec<rtl::object::LocatedOpCode>,
    literal_count: usize,
    register_count: usize,
}

impl<'a> RTLCompiler<'a> {
    fn new(object: &'a rhif::object::Object) -> Self {
        Self {
            object,
            literals: Default::default(),
            registers: Default::default(),
            operand_map: Default::default(),
            reverse_operand_map: Default::default(),
            externals: Default::default(),
            ops: Default::default(),
            literal_count: 0,
            register_count: 0,
        }
    }
    fn allocate_literal(&mut self, bits: &TypedBits) -> LiteralId {
        let literal_id = LiteralId(self.literal_count);
        self.literal_count += 1;
        self.literals.insert(literal_id, bits.into());
        literal_id
    }
    fn allocate_literal_from_usize_and_length(
        &mut self,
        value: usize,
        bits: usize,
    ) -> Result<LiteralId> {
        let value = value as u64;
        let value: TypedBits = value.into();
        let value = value.unsigned_cast(bits)?;
        Ok(self.allocate_literal(&value))
    }
    fn allocate_signed(&mut self, length: usize) -> RegisterId {
        let register_id = RegisterId(self.register_count);
        self.register_count += 1;
        self.registers
            .insert(register_id, RegisterKind::Signed(length));
        register_id
    }
    fn allocate_unsigned(&mut self, length: usize) -> RegisterId {
        let register_id = RegisterId(self.register_count);
        self.register_count += 1;
        self.registers
            .insert(register_id, RegisterKind::Unsigned(length));
        register_id
    }
    fn allocate_register(&mut self, kind: &Kind) -> RegisterId {
        let len = kind.bits();
        if kind.is_signed() {
            self.allocate_signed(len)
        } else {
            self.allocate_unsigned(len)
        }
    }
    fn raise_ice(&self, cause: ICE, id: NodeId) -> RHDLError {
        panic!("ICE: {:?}", cause);
        RHDLError::RHDLInternalCompilerError(Box::new(RHDLCompileError {
            cause,
            src: self.object.symbols.source.source.clone(),
            err_span: self.object.symbols.node_span(id).into(),
        }))
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
            Operand::Literal(self.allocate_literal_from_usize_and_length(base_offset, index_bits)?);
        for (slot, stride) in dynamic_slots.iter().zip(slot_strides.iter()) {
            // Store the stride in a literal
            let stride = self.allocate_literal_from_usize_and_length(*stride, index_bits)?;
            let reg = self.allocate_register(&Kind::make_bits(index_bits));
            let arg = self.operand(*slot, node_id)?;
            // reg <- arg as L
            self.ops.push(
                (
                    tl::OpCode::AsBits(tl::Cast {
                        lhs: Operand::Register(reg),
                        arg,
                        len: index_bits,
                    }),
                    node_id,
                )
                    .into(),
            );
            let prod = self.allocate_register(&Kind::make_bits(index_bits));
            // prod <- reg * stride
            self.ops.push(
                (
                    tl::OpCode::Binary(tl::Binary {
                        lhs: Operand::Register(prod),
                        op: AluBinary::Mul,
                        arg1: Operand::Register(reg),
                        arg2: Operand::Literal(stride),
                    }),
                    node_id,
                )
                    .into(),
            );
            let sum = self.allocate_register(&Kind::make_bits(index_bits));
            // sum <- sum + prod
            self.ops.push(
                (
                    tl::OpCode::Binary(tl::Binary {
                        lhs: Operand::Register(sum),
                        op: AluBinary::Add,
                        arg1: index_sum,
                        arg2: Operand::Register(prod),
                    }),
                    node_id,
                )
                    .into(),
            );
            index_sum = Operand::Register(sum);
        }
        Ok((index_sum, base_range.len()))
    }
    fn operand(&mut self, slot: Slot, id: NodeId) -> Result<Operand> {
        if let Some(operand) = self.reverse_operand_map.get(&slot) {
            return Ok(*operand);
        }
        match slot {
            Slot::Literal(literal_id) => {
                let bits = &self.object.literals[&literal_id];
                let literal_id = self.allocate_literal(bits);
                let operand = Operand::Literal(literal_id);
                self.reverse_operand_map.insert(slot, operand);
                self.operand_map.insert(operand, slot);
                Ok(operand)
            }
            Slot::Register(register_id) => {
                let kind = &self.object.kind[&register_id];
                let register_id = self.allocate_register(kind);
                let operand = Operand::Register(register_id);
                self.reverse_operand_map.insert(slot, operand);
                self.operand_map.insert(operand, slot);
                Ok(operand)
            }
            Slot::Empty => Err(self.raise_ice(ICE::EmptySlotInRTL, id)),
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
        let elements = self.make_operand_list(&elements, id)?;
        self.ops.push(
            (
                tl::OpCode::Concat(tl::Concat {
                    lhs,
                    args: elements,
                }),
                id,
            )
                .into(),
        );
        Ok(())
    }
    fn make_as_bits(&mut self, cast: &hf::Cast, id: NodeId) -> Result<()> {
        let hf::Cast { lhs, arg, len } = cast;
        let len = len.ok_or_else(|| self.raise_ice(ICE::BitCastMissingRequiredLength, id))?;
        if !lhs.is_empty() && !arg.is_empty() {
            let lhs = self.operand(*lhs, id)?;
            let arg = self.operand(*arg, id)?;
            self.ops
                .push((tl::OpCode::AsBits(tl::Cast { lhs, arg, len }), id).into());
        }
        Ok(())
    }
    fn make_as_signed(&mut self, cast: &hf::Cast, id: NodeId) -> Result<()> {
        let hf::Cast { lhs, arg, len } = cast;
        let len = len.ok_or_else(|| self.raise_ice(ICE::BitCastMissingRequiredLength, id))?;
        if !lhs.is_empty() && !arg.is_empty() {
            let lhs = self.operand(*lhs, id)?;
            let arg = self.operand(*arg, id)?;
            self.ops
                .push((tl::OpCode::AsSigned(tl::Cast { lhs, arg, len }), id).into());
        }
        Ok(())
    }
    fn make_assign(&mut self, assign: &hf::Assign, id: NodeId) -> Result<()> {
        let hf::Assign { lhs, rhs } = assign;
        if !lhs.is_empty() && !rhs.is_empty() {
            let lhs = self.operand(*lhs, id)?;
            let rhs = self.operand(*rhs, id)?;
            self.ops
                .push((tl::OpCode::Assign(tl::Assign { lhs, rhs }), id).into());
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
            self.ops.push(
                (
                    tl::OpCode::Binary(tl::Binary {
                        lhs,
                        op,
                        arg1,
                        arg2,
                    }),
                    id,
                )
                    .into(),
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
        self.ops.push(
            (
                tl::OpCode::Case(tl::Case {
                    lhs,
                    discriminant,
                    table,
                }),
                id,
            )
                .into(),
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
        self.ops.push(
            (
                tl::OpCode::DynamicSplice(tl::DynamicSplice {
                    lhs,
                    arg: orig,
                    offset: index_reg,
                    len,
                    value: subst,
                }),
                node_id,
            )
                .into(),
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
        let mut rhs = Operand::Literal(self.allocate_literal(template));
        for field in fields {
            let field_value = self.operand(field.value, id)?;
            let path = Path::default()
                .payload_by_value(discriminant)
                .member(&field.member);
            let (field_range, _) = bit_range(kind.clone(), &path)?;
            let reg = Operand::Register(self.allocate_register(&kind));
            self.ops.push(
                (
                    tl::OpCode::Splice(tl::Splice {
                        lhs: reg,
                        orig: rhs,
                        bit_range: field_range,
                        value: field_value,
                    }),
                    id,
                )
                    .into(),
            );
            rhs = reg;
        }
        let lhs = self.operand(*lhs, id)?;
        self.ops
            .push((tl::OpCode::Assign(tl::Assign { lhs, rhs }), id).into());
        Ok(())
    }
    fn make_exec(&mut self, exec: &hf::Exec, node_id: NodeId) -> Result<()> {
        let hf::Exec { lhs, id, args } = exec;
        if lhs.is_empty() {
            return Ok(());
        }
        let lhs = self.operand(*lhs, node_id)?;
        let args = args
            .iter()
            .map(|x| self.operand(*x, node_id).ok())
            .collect();
        self.ops
            .push((tl::OpCode::Exec(tl::Exec { lhs, id: *id, args }), node_id).into());
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
        self.ops.push(
            (
                tl::OpCode::DynamicIndex(tl::DynamicIndex {
                    lhs,
                    arg,
                    offset: index_reg,
                    len,
                }),
                node_id,
            )
                .into(),
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
        self.ops.push(
            (
                tl::OpCode::Index(tl::Index {
                    lhs,
                    arg,
                    bit_range,
                }),
                node_id,
            )
                .into(),
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
        self.ops
            .push((tl::OpCode::Concat(tl::Concat { lhs, args }), node_id).into());
        Ok(())
    }
    fn make_retime(&mut self, retime: &hf::Retime, node_id: NodeId) -> Result<()> {
        let hf::Retime { lhs, arg, color: _ } = *retime;
        if lhs.is_empty() {
            return Ok(());
        }
        let lhs = self.operand(lhs, node_id)?;
        let rhs = self.operand(arg, node_id)?;
        self.ops
            .push((tl::OpCode::Assign(tl::Assign { lhs, rhs }), node_id).into());
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
        self.ops.push(
            (
                tl::OpCode::Select(tl::Select {
                    lhs,
                    cond,
                    true_value,
                    false_value,
                }),
                node_id,
            )
                .into(),
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
        self.ops.push(
            (
                tl::OpCode::Splice(tl::Splice {
                    lhs,
                    orig,
                    bit_range,
                    value: subst,
                }),
                node_id,
            )
                .into(),
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
            Operand::Literal(self.allocate_literal(template))
        };
        for field in fields {
            let field_value = self.operand(field.value, node_id)?;
            let path = Path::default().member(&field.member);
            let (field_range, _) = bit_range(kind.clone(), &path)?;
            let reg = Operand::Register(self.allocate_register(&kind));
            self.ops.push(
                (
                    tl::OpCode::Splice(tl::Splice {
                        lhs: reg,
                        orig: rhs,
                        bit_range: field_range,
                        value: field_value,
                    }),
                    node_id,
                )
                    .into(),
            );
            rhs = reg;
        }
        self.ops
            .push((tl::OpCode::Assign(tl::Assign { lhs, rhs }), node_id).into());
        Ok(())
    }
    fn make_tuple(&mut self, tuple: &hf::Tuple, node_id: NodeId) -> Result<()> {
        let hf::Tuple { lhs, fields } = tuple;
        if lhs.is_empty() {
            return Ok(());
        }
        let lhs = self.operand(*lhs, node_id)?;
        let args = self.make_operand_list(fields, node_id)?;
        self.ops
            .push((tl::OpCode::Concat(tl::Concat { lhs, args }), node_id).into());
        Ok(())
    }
    fn make_unary(&mut self, unary: &hf::Unary, node_id: NodeId) -> Result<()> {
        let hf::Unary { lhs, op, arg1 } = *unary;
        if lhs.is_empty() {
            return Ok(());
        }
        let lhs = self.operand(lhs, node_id)?;
        let arg1 = self.operand(arg1, node_id)?;
        self.ops
            .push((tl::OpCode::Unary(tl::Unary { lhs, op, arg1 }), node_id).into());
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
                    self.ops
                        .push((tl::OpCode::Comment(comment.clone()), lop.id).into());
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
            }
        }
        Ok(self)
    }
}

pub fn compile_rtl(object: &rhif::object::Object) -> Result<rtl::object::Object> {
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
    let return_register = if object.return_slot.is_empty() {
        None
    } else {
        Some(compiler.reverse_operand_map[&object.return_slot])
    };
    Ok(rtl::object::Object {
        symbols: object.symbols.clone(),
        literals: compiler.literals,
        operand_map: compiler.operand_map,
        return_register,
        register_kind: compiler.registers,
        externals: compiler.externals,
        ops: compiler.ops,
        arguments,
        name: object.name.clone(),
        fn_id: object.fn_id,
    })
}
