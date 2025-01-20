use std::collections::BTreeMap;

use crate::rhdl_bits::alias::b8;

use crate::rhdl_core::ast::ast_impl::{FunctionId, WrapOp};
use crate::rhdl_core::ast::source::source_location::SourceLocation;
use crate::rhdl_core::error::rhdl_error;
use crate::rhdl_core::rhif::spec::{AluBinary, Slot};
use crate::rhdl_core::rtl::object::{lop, RegisterKind};
use crate::rhdl_core::rtl::remap::remap_operands;
use crate::rhdl_core::rtl::spec::{CastKind, Concat, LiteralId, Operand, RegisterId};
use crate::rhdl_core::rtl::symbols::SymbolMap;
use crate::rhdl_core::types::bit_string::BitString;
use crate::rhdl_core::types::path::{bit_range, Path};
use crate::rhdl_core::util::clog2;
use crate::rhdl_core::Digital;
use crate::rhdl_core::TypedBits;
use crate::rhdl_core::{rhif, RHDLError};
use crate::rhdl_core::{rtl, Kind};

use crate::rhdl_core::rhif::spec as hf;
use crate::rhdl_core::rtl::spec as tl;

use crate::rhdl_core::compiler::mir::error::{RHDLCompileError, ICE};

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

// map a binary op from RHIF to RTL if possible
fn map_binop(op: hf::AluBinary) -> Option<tl::AluBinary> {
    match op {
        AluBinary::Add => Some(tl::AluBinary::Add),
        AluBinary::Sub => Some(tl::AluBinary::Sub),
        AluBinary::Mul => Some(tl::AluBinary::Mul),
        AluBinary::BitXor => Some(tl::AluBinary::BitXor),
        AluBinary::BitAnd => Some(tl::AluBinary::BitAnd),
        AluBinary::BitOr => Some(tl::AluBinary::BitOr),
        AluBinary::Shl => Some(tl::AluBinary::Shl),
        AluBinary::Shr => Some(tl::AluBinary::Shr),
        AluBinary::Eq => Some(tl::AluBinary::Eq),
        AluBinary::Lt => Some(tl::AluBinary::Lt),
        AluBinary::Le => Some(tl::AluBinary::Le),
        AluBinary::Ne => Some(tl::AluBinary::Ne),
        AluBinary::Ge => Some(tl::AluBinary::Ge),
        AluBinary::Gt => Some(tl::AluBinary::Gt),
        _ => None,
    }
}

impl<'a> RTLCompiler<'a> {
    fn new(object: &'a rhif::object::Object) -> Self {
        let mut symbols = SymbolMap::default();
        symbols.source_set = object.symbols.source_set.clone();
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
    fn associate(&mut self, operand: Operand, loc: SourceLocation) {
        self.symbols.operand_map.insert(operand, loc);
    }
    fn allocate_literal(&mut self, bits: &TypedBits, loc: SourceLocation) -> Operand {
        let literal_id = LiteralId::new(self.literal_count);
        self.literal_count += 1;
        self.literals.insert(literal_id, bits.into());
        let literal = Operand::Literal(literal_id);
        self.associate(literal, loc);
        literal
    }
    fn allocate_literal_from_bit_string(
        &mut self,
        bits: &BitString,
        loc: SourceLocation,
    ) -> Operand {
        let literal_id = LiteralId::new(self.literal_count);
        self.literal_count += 1;
        self.literals.insert(literal_id, bits.clone());
        let literal = Operand::Literal(literal_id);
        self.associate(literal, loc);
        literal
    }
    fn allocate_literal_from_usize_and_length(
        &mut self,
        value: usize,
        bits: usize,
        loc: SourceLocation,
    ) -> Result<Operand> {
        let value = value as u64;
        let value: TypedBits = value.into();
        let value = value.unsigned_cast(bits)?;
        Ok(self.allocate_literal(&value, loc))
    }
    fn allocate_signed(&mut self, length: usize, loc: SourceLocation) -> Operand {
        let register_id = RegisterId::new(self.register_count);
        self.register_count += 1;
        self.registers
            .insert(register_id, RegisterKind::Signed(length));
        let register = Operand::Register(register_id);
        self.associate(register, loc);
        register
    }
    fn allocate_unsigned(&mut self, length: usize, loc: SourceLocation) -> Operand {
        let register_id = RegisterId::new(self.register_count);
        self.register_count += 1;
        self.registers
            .insert(register_id, RegisterKind::Unsigned(length));
        let register = Operand::Register(register_id);
        self.associate(register, loc);
        register
    }
    fn allocate_register(&mut self, kind: &Kind, loc: SourceLocation) -> Operand {
        let len = kind.bits();
        if kind.is_signed() {
            self.allocate_signed(len, loc)
        } else {
            self.allocate_unsigned(len, loc)
        }
    }
    fn allocate_register_with_register_kind(
        &mut self,
        kind: &RegisterKind,
        loc: SourceLocation,
    ) -> Operand {
        let register_id = RegisterId::new(self.register_count);
        self.register_count += 1;
        self.registers.insert(register_id, *kind);
        let register = Operand::Register(register_id);
        self.associate(register, loc);
        register
    }
    fn operand_bit_width(&self, operand: Operand) -> usize {
        match operand {
            Operand::Literal(literal_id) => self.literals[&literal_id].len(),
            Operand::Register(register_id) => self.registers[&register_id].len(),
        }
    }
    fn operand_is_signed(&self, operand: Operand) -> bool {
        match operand {
            Operand::Literal(literal_id) => self.literals[&literal_id].is_signed(),
            Operand::Register(register_id) => self.registers[&register_id].is_signed(),
        }
    }
    fn raise_ice(&self, cause: ICE, loc: SourceLocation) -> RHDLError {
        rhdl_error(RHDLCompileError {
            cause,
            src: self.object.symbols.source(),
            err_span: self.object.symbols.span(loc).into(),
        })
    }
    fn lop(&mut self, opcode: tl::OpCode, loc: SourceLocation) {
        self.ops.push((opcode, loc).into())
    }
    fn build_dynamic_index(
        &mut self,
        arg: Slot,
        path: &Path,
        loc: SourceLocation,
    ) -> Result<(Operand, usize)> {
        let dynamic_slots: Vec<Slot> = path.dynamic_slots().copied().collect();
        // First to get the base offset, we construct a path that replaces all
        // dynamic indices with 0
        let arg_kind = self.object.kind(arg);
        let base_path = path.zero_out_dynamic_indices();
        let (base_range, base_kind) = bit_range(arg_kind, &base_path)?;
        // Next for each index register, we compute a range where only that index
        // is advanced by one.
        let (slot_ranges, slot_kinds) = dynamic_slots
            .iter()
            .map(|slot| {
                let stride_path = path.stride_path(*slot);
                bit_range(arg_kind, &stride_path)
            })
            .collect::<Result<(Vec<_>, Vec<_>)>>()?;
        // Validation - all of the kinds should be the same
        if let Some(kind) = slot_kinds.iter().find(|kind| **kind != base_kind) {
            return Err(self.raise_ice(
                ICE::MismatchedTypesFromDynamicIndexing {
                    base: base_kind,
                    slot: *kind,
                },
                loc,
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
                loc,
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
            self.allocate_literal_from_usize_and_length(base_offset, index_bits, loc)?;
        for (slot, stride) in dynamic_slots.iter().zip(slot_strides.iter()) {
            // Store the stride in a literal
            let stride = self.allocate_literal_from_usize_and_length(*stride, index_bits, loc)?;
            let reg = self.allocate_register(&Kind::make_bits(index_bits), loc);
            let arg = self.operand(*slot, loc)?;
            // reg <- arg as L
            self.lop(
                tl::OpCode::Cast(tl::Cast {
                    lhs: reg,
                    arg,
                    len: index_bits,
                    kind: CastKind::Unsigned,
                }),
                loc,
            );
            let prod_uncast = self.allocate_register(&Kind::make_bits(index_bits * 2), loc);
            self.lop(
                tl::OpCode::Binary(tl::Binary {
                    lhs: prod_uncast,
                    op: crate::rhdl_core::rtl::spec::AluBinary::Mul,
                    arg1: reg,
                    arg2: stride,
                }),
                loc,
            );
            let prod = self.allocate_register(&Kind::make_bits(index_bits), loc);
            // prod <- (reg * stride) as L
            self.lop(
                tl::OpCode::Cast(tl::Cast {
                    lhs: prod,
                    arg: prod_uncast,
                    len: index_bits,
                    kind: CastKind::Unsigned,
                }),
                loc,
            );
            let sum = self.allocate_register(&Kind::make_bits(index_bits), loc);
            // sum <- sum + prod
            self.lop(
                tl::OpCode::Binary(tl::Binary {
                    lhs: sum,
                    op: crate::rhdl_core::rtl::spec::AluBinary::Add,
                    arg1: index_sum,
                    arg2: prod,
                }),
                loc,
            );
            index_sum = sum;
        }
        Ok((index_sum, base_range.len()))
    }
    fn operand(&mut self, slot: Slot, loc: SourceLocation) -> Result<Operand> {
        if let Some(operand) = self.reverse_operand_map.get(&(self.object.fn_id, slot)) {
            return Ok(*operand);
        }
        match slot {
            Slot::Literal(literal_id) => {
                let bits = &self.object.literals[&literal_id];
                let operand = self.allocate_literal(bits, loc);
                let loc = self.object.symbols.slot_map[&slot];
                self.symbols.operand_map.insert(operand, loc);
                self.reverse_operand_map
                    .insert((self.object.fn_id, slot), operand);
                self.operand_map.insert(operand, (self.object.fn_id, slot));
                Ok(operand)
            }
            Slot::Register(register_id) => {
                let kind = &self.object.kind[&register_id];
                let operand = self.allocate_register(kind, loc);
                let loc = self.object.symbols.slot_map[&slot];
                self.symbols.operand_map.insert(operand, loc);
                self.reverse_operand_map
                    .insert((self.object.fn_id, slot), operand);
                self.operand_map.insert(operand, (self.object.fn_id, slot));
                Ok(operand)
            }
            Slot::Empty => panic!("empty slot"), //Err(self.raise_ice(ICE::EmptySlotInRTL, id)),
        }
    }
    fn make_operand_list(&mut self, args: &[Slot], loc: SourceLocation) -> Result<Vec<Operand>> {
        args.iter()
            .filter_map(|a| {
                if a.is_empty() {
                    None
                } else {
                    Some(self.operand(*a, loc))
                }
            })
            .collect()
    }
    fn make_array(&mut self, args: &hf::Array, loc: SourceLocation) -> Result<()> {
        let hf::Array { lhs, elements } = args;
        if lhs.is_empty() {
            return Ok(());
        }
        let lhs = self.operand(*lhs, loc)?;
        let elements = self.make_operand_list(elements, loc)?;
        self.lop(
            tl::OpCode::Concat(tl::Concat {
                lhs,
                args: elements,
            }),
            loc,
        );
        Ok(())
    }
    fn make_resize(&mut self, cast: &hf::Cast, loc: SourceLocation) -> Result<()> {
        let hf::Cast { lhs, arg, len } = cast;
        let len = len.ok_or_else(|| self.raise_ice(ICE::BitCastMissingRequiredLength, loc))?;
        if !lhs.is_empty() && !arg.is_empty() {
            let lhs = self.operand(*lhs, loc)?;
            let op_arg = self.operand(*arg, loc)?;
            self.lop(
                tl::OpCode::Cast(tl::Cast {
                    lhs,
                    arg: op_arg,
                    len,
                    kind: CastKind::Resize,
                }),
                loc,
            );
        }
        Ok(())
    }
    fn make_wrap(&mut self, wrap: &hf::Wrap, loc: SourceLocation) -> Result<()> {
        let hf::Wrap { lhs, op, arg, kind } = wrap;
        let kind = kind.ok_or_else(|| self.raise_ice(ICE::WrapMissingKind, loc))?;
        let discriminant = match op {
            WrapOp::Ok | WrapOp::Some => self.allocate_literal(&true.typed_bits(), loc),
            WrapOp::Err | WrapOp::None => self.allocate_literal(&false.typed_bits(), loc),
        };
        let width = kind.bits() - 1;
        let lhs = self.operand(*lhs, loc)?;
        if width != 0 {
            let payload =
                self.allocate_register_with_register_kind(&RegisterKind::Unsigned(width), loc);
            let arg = *arg;
            let arg = match arg {
                Slot::Empty => self.allocate_literal(&false.typed_bits(), loc),
                _ => self.operand(arg, loc)?,
            };
            self.lop(
                tl::OpCode::Cast(tl::Cast {
                    lhs: payload,
                    arg,
                    len: width,
                    kind: CastKind::Resize,
                }),
                loc,
            );
            self.lop(
                tl::OpCode::Concat(Concat {
                    lhs,
                    args: vec![payload, discriminant],
                }),
                loc,
            );
        } else {
            self.lop(
                tl::OpCode::Assign(tl::Assign {
                    lhs,
                    rhs: discriminant,
                }),
                loc,
            );
        };
        Ok(())
    }
    fn make_as_bits(&mut self, cast: &hf::Cast, loc: SourceLocation) -> Result<()> {
        let hf::Cast { lhs, arg, len } = cast;
        let len = len.ok_or_else(|| self.raise_ice(ICE::BitCastMissingRequiredLength, loc))?;
        if !lhs.is_empty() && !arg.is_empty() {
            let lhs = self.operand(*lhs, loc)?;
            let arg = self.operand(*arg, loc)?;
            self.lop(
                tl::OpCode::Cast(tl::Cast {
                    lhs,
                    arg,
                    len,
                    kind: CastKind::Unsigned,
                }),
                loc,
            );
        }
        Ok(())
    }
    fn make_as_signed(&mut self, cast: &hf::Cast, loc: SourceLocation) -> Result<()> {
        let hf::Cast { lhs, arg, len } = cast;
        let len = len.ok_or_else(|| self.raise_ice(ICE::BitCastMissingRequiredLength, loc))?;
        if !lhs.is_empty() && !arg.is_empty() {
            let lhs = self.operand(*lhs, loc)?;
            let arg = self.operand(*arg, loc)?;
            self.lop(
                tl::OpCode::Cast(tl::Cast {
                    lhs,
                    arg,
                    len,
                    kind: CastKind::Signed,
                }),
                loc,
            );
        }
        Ok(())
    }
    fn make_assign(&mut self, assign: &hf::Assign, loc: SourceLocation) -> Result<()> {
        let hf::Assign { lhs, rhs } = assign;
        if !lhs.is_empty() && !rhs.is_empty() {
            let lhs = self.operand(*lhs, loc)?;
            let rhs = self.operand(*rhs, loc)?;
            self.lop(tl::OpCode::Assign(tl::Assign { lhs, rhs }), loc);
        }
        Ok(())
    }
    fn make_xadd_or_xmul(
        &mut self,
        lhs: Operand,
        arg1: Operand,
        arg2: Operand,
        loc: SourceLocation,
        op: tl::AluBinary,
    ) -> Result<()> {
        let Operand::Register(lhs_id) = lhs else {
            return Err(self.raise_ice(ICE::XopsResultMustBeRegister, loc));
        };
        let lhs_reg_kind = self.registers[&lhs_id];
        let arg1_cast = self.allocate_register_with_register_kind(&lhs_reg_kind, loc);
        let arg2_cast = self.allocate_register_with_register_kind(&lhs_reg_kind, loc);
        self.lop(
            tl::OpCode::Cast(tl::Cast {
                lhs: arg1_cast,
                arg: arg1,
                len: lhs_reg_kind.len(),
                kind: CastKind::Resize,
            }),
            loc,
        );
        self.lop(
            tl::OpCode::Cast(tl::Cast {
                lhs: arg2_cast,
                arg: arg2,
                len: lhs_reg_kind.len(),
                kind: CastKind::Resize,
            }),
            loc,
        );
        self.lop(
            tl::OpCode::Binary(tl::Binary {
                lhs,
                op,
                arg1: arg1_cast,
                arg2: arg2_cast,
            }),
            loc,
        );
        Ok(())
    }
    fn make_xsub(
        &mut self,
        lhs: Operand,
        arg1: Operand,
        arg2: Operand,
        loc: SourceLocation,
    ) -> Result<()> {
        let Operand::Register(lhs_id) = lhs else {
            return Err(self.raise_ice(ICE::XopsResultMustBeRegister, loc));
        };
        // The cast operation has to be split into two steps depending
        // on the sign of the operands.  The result is always signed.
        let lhs_reg_kind = self.registers[&lhs_id];
        let lhs_len = lhs_reg_kind.len();
        let extension_kind = if self.operand_is_signed(arg1) {
            RegisterKind::Signed(lhs_len)
        } else {
            RegisterKind::Unsigned(lhs_len)
        };
        // First we extend the operands to the required number of bits
        let arg1_extend = self.allocate_register_with_register_kind(&extension_kind, loc);
        let arg2_extend = self.allocate_register_with_register_kind(&extension_kind, loc);
        self.lop(
            tl::OpCode::Cast(tl::Cast {
                lhs: arg1_extend,
                arg: arg1,
                len: lhs_len,
                kind: CastKind::Resize,
            }),
            loc,
        );
        self.lop(
            tl::OpCode::Cast(tl::Cast {
                lhs: arg2_extend,
                arg: arg2,
                len: lhs_len,
                kind: CastKind::Resize,
            }),
            loc,
        );
        // This guarantees that the sign bit will be zero when we reinterepret them as signed values
        let arg1_cast = self.allocate_register_with_register_kind(&lhs_reg_kind, loc);
        let arg2_cast = self.allocate_register_with_register_kind(&lhs_reg_kind, loc);
        self.lop(
            tl::OpCode::Cast(tl::Cast {
                lhs: arg1_cast,
                arg: arg1_extend,
                len: lhs_reg_kind.len(),
                kind: CastKind::Signed,
            }),
            loc,
        );
        self.lop(
            tl::OpCode::Cast(tl::Cast {
                lhs: arg2_cast,
                arg: arg2_extend,
                len: lhs_reg_kind.len(),
                kind: CastKind::Signed,
            }),
            loc,
        );
        self.lop(
            tl::OpCode::Binary(tl::Binary {
                lhs,
                op: tl::AluBinary::Sub,
                arg1: arg1_cast,
                arg2: arg2_cast,
            }),
            loc,
        );
        Ok(())
    }
    fn make_binary(&mut self, binary: &hf::Binary, loc: SourceLocation) -> Result<()> {
        let hf::Binary {
            lhs,
            op,
            arg1,
            arg2,
        } = *binary;
        if lhs.is_empty() {
            return Ok(());
        }
        let lhs = self.operand(lhs, loc)?;
        let arg1 = self.operand(arg1, loc)?;
        let arg2 = self.operand(arg2, loc)?;
        let mut rtl_binop = |op| {
            self.lop(
                tl::OpCode::Binary(tl::Binary {
                    lhs,
                    op,
                    arg1,
                    arg2,
                }),
                loc,
            );
            Ok(())
        };
        match op {
            AluBinary::Add => rtl_binop(tl::AluBinary::Add),
            AluBinary::Sub => rtl_binop(tl::AluBinary::Sub),
            AluBinary::Mul => rtl_binop(tl::AluBinary::Mul),
            AluBinary::BitAnd => rtl_binop(tl::AluBinary::BitAnd),
            AluBinary::BitOr => rtl_binop(tl::AluBinary::BitOr),
            AluBinary::BitXor => rtl_binop(tl::AluBinary::BitXor),
            AluBinary::Shl => rtl_binop(tl::AluBinary::Shl),
            AluBinary::Shr => rtl_binop(tl::AluBinary::Shr),
            AluBinary::Eq => rtl_binop(tl::AluBinary::Eq),
            AluBinary::Ne => rtl_binop(tl::AluBinary::Ne),
            AluBinary::Lt => rtl_binop(tl::AluBinary::Lt),
            AluBinary::Le => rtl_binop(tl::AluBinary::Le),
            AluBinary::Gt => rtl_binop(tl::AluBinary::Gt),
            AluBinary::Ge => rtl_binop(tl::AluBinary::Ge),
            AluBinary::XAdd => self.make_xadd_or_xmul(lhs, arg1, arg2, loc, tl::AluBinary::Add),
            AluBinary::XSub => self.make_xsub(lhs, arg1, arg2, loc),
            AluBinary::XMul => self.make_xadd_or_xmul(lhs, arg1, arg2, loc, tl::AluBinary::Mul),
        }
    }
    fn make_case_argument(
        &mut self,
        case_argument: &hf::CaseArgument,
        loc: SourceLocation,
    ) -> Result<tl::CaseArgument> {
        match case_argument {
            hf::CaseArgument::Slot(slot) => {
                if !slot.is_literal() {
                    return Err(self.raise_ice(ICE::MatchPatternValueMustBeLiteral, loc));
                };
                let operand = self.operand(*slot, loc)?;
                let Operand::Literal(literal_id) = operand else {
                    return Err(self.raise_ice(ICE::MatchPatternValueMustBeLiteral, loc));
                };
                Ok(tl::CaseArgument::Literal(literal_id))
            }
            hf::CaseArgument::Wild => Ok(tl::CaseArgument::Wild),
        }
    }
    fn make_case(&mut self, case: &hf::Case, loc: SourceLocation) -> Result<()> {
        let hf::Case {
            lhs,
            discriminant,
            table,
        } = case;
        if lhs.is_empty() {
            return Ok(());
        }
        let lhs = self.operand(*lhs, loc)?;
        let discriminant = self.operand(*discriminant, loc)?;
        let table = table
            .iter()
            .map(|(cond, val)| {
                let cond = self.make_case_argument(cond, loc)?;
                let val = self.operand(*val, loc)?;
                Ok((cond, val))
            })
            .collect::<Result<Vec<_>>>()?;
        self.lop(
            tl::OpCode::Case(tl::Case {
                lhs,
                discriminant,
                table,
            }),
            loc,
        );
        Ok(())
    }
    fn make_dynamic_splice(&mut self, splice: &hf::Splice, loc: SourceLocation) -> Result<()> {
        let hf::Splice {
            lhs,
            orig,
            path,
            subst,
        } = splice;
        if lhs.is_empty() {
            return Ok(());
        }
        let (index_reg, len) = self.build_dynamic_index(*orig, path, loc)?;
        let lhs = self.operand(*lhs, loc)?;
        let orig = self.operand(*orig, loc)?;
        let subst = self.operand(*subst, loc)?;
        self.lop(
            tl::OpCode::DynamicSplice(tl::DynamicSplice {
                lhs,
                arg: orig,
                offset: index_reg,
                len,
                value: subst,
            }),
            loc,
        );
        Ok(())
    }
    fn make_enum(&mut self, enumerate: &hf::Enum, id: SourceLocation) -> Result<()> {
        let hf::Enum {
            lhs,
            fields,
            template,
        } = enumerate;
        if lhs.is_empty() {
            return Ok(());
        }
        let kind = template.kind;
        let discriminant = template.discriminant()?.as_i64()?;
        let mut rhs = self.allocate_literal(template, id);
        for field in fields {
            let field_value = self.operand(field.value, id)?;
            let path = Path::default()
                .payload_by_value(discriminant)
                .member(&field.member);
            let (field_range, _) = bit_range(kind, &path)?;
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
    fn make_exec(&mut self, exec: &hf::Exec, loc: SourceLocation) -> Result<()> {
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
                let fn_reg_in_our_space =
                    self.allocate_register_with_register_kind(&func_rtl.register_kind[fn_reg], loc);
                operand_translation.insert(Operand::Register(*fn_reg), fn_reg_in_our_space);
                let arg = self.operand(*arg, loc)?;
                self.lop(
                    tl::OpCode::Assign(tl::Assign {
                        lhs: fn_reg_in_our_space,
                        rhs: arg,
                    }),
                    loc,
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
                    self.allocate_literal_from_bit_string(&old_lit, loc)
                }
                Operand::Register(old_reg_id) => {
                    let kind = func_rtl.register_kind[&old_reg_id];
                    self.allocate_register_with_register_kind(&kind, loc)
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
        let lhs = self.operand(*lhs, loc)?;
        self.lop(
            tl::OpCode::Assign(tl::Assign {
                lhs,
                rhs: return_register,
            }),
            loc,
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
    fn make_dynamic_index(&mut self, index: &hf::Index, loc: SourceLocation) -> Result<()> {
        let hf::Index { lhs, arg, path } = index;
        if lhs.is_empty() {
            return Ok(());
        }
        let (index_reg, len) = self.build_dynamic_index(*arg, path, loc)?;
        let lhs = self.operand(*lhs, loc)?;
        let arg = self.operand(*arg, loc)?;
        self.lop(
            tl::OpCode::DynamicIndex(tl::DynamicIndex {
                lhs,
                arg,
                offset: index_reg,
                len,
            }),
            loc,
        );
        Ok(())
    }
    fn make_index(&mut self, index: &hf::Index, loc: SourceLocation) -> Result<()> {
        if index.lhs.is_empty() {
            return Ok(());
        }
        if index.path.any_dynamic() {
            return self.make_dynamic_index(index, loc);
        }
        let arg_ty = self.object.kind(index.arg);
        let (bit_range, _) = bit_range(arg_ty, &index.path)?;
        let lhs = self.operand(index.lhs, loc)?;
        let arg = self.operand(index.arg, loc)?;
        self.lop(
            tl::OpCode::Index(tl::Index {
                lhs,
                arg,
                bit_range,
            }),
            loc,
        );
        Ok(())
    }
    fn make_repeat(&mut self, repeat: &hf::Repeat, loc: SourceLocation) -> Result<()> {
        let hf::Repeat { lhs, value, len } = *repeat;
        if lhs.is_empty() {
            return Ok(());
        }
        let lhs = self.operand(lhs, loc)?;
        let arg = self.operand(value, loc)?;
        let args = vec![arg; len as usize];
        self.lop(tl::OpCode::Concat(tl::Concat { lhs, args }), loc);
        Ok(())
    }
    fn make_retime(&mut self, retime: &hf::Retime, loc: SourceLocation) -> Result<()> {
        let hf::Retime { lhs, arg, color: _ } = *retime;
        if lhs.is_empty() {
            return Ok(());
        }
        let lhs = self.operand(lhs, loc)?;
        let rhs = self.operand(arg, loc)?;
        self.lop(tl::OpCode::Assign(tl::Assign { lhs, rhs }), loc);
        Ok(())
    }
    fn make_select(&mut self, select: &hf::Select, loc: SourceLocation) -> Result<()> {
        let hf::Select {
            lhs,
            cond,
            true_value,
            false_value,
        } = *select;
        if lhs.is_empty() {
            return Ok(());
        }
        let lhs = self.operand(lhs, loc)?;
        let cond = self.operand(cond, loc)?;
        let true_value = self.operand(true_value, loc)?;
        let false_value = self.operand(false_value, loc)?;
        self.lop(
            tl::OpCode::Select(tl::Select {
                lhs,
                cond,
                true_value,
                false_value,
            }),
            loc,
        );
        Ok(())
    }
    fn make_splice(&mut self, splice: &hf::Splice, loc: SourceLocation) -> Result<()> {
        if splice.lhs.is_empty() {
            return Ok(());
        }
        if splice.path.any_dynamic() {
            return self.make_dynamic_splice(splice, loc);
        }
        let hf::Splice {
            lhs,
            orig,
            path,
            subst,
        } = splice;
        let orig_ty = self.object.kind(*orig);
        let (bit_range, _) = bit_range(orig_ty, path)?;
        let lhs = self.operand(*lhs, loc)?;
        let orig = self.operand(*orig, loc)?;
        let subst = self.operand(*subst, loc)?;
        self.lop(
            tl::OpCode::Splice(tl::Splice {
                lhs,
                orig,
                bit_range,
                value: subst,
            }),
            loc,
        );
        Ok(())
    }
    fn make_struct(&mut self, strukt: &hf::Struct, loc: SourceLocation) -> Result<()> {
        let hf::Struct {
            lhs,
            fields,
            rest,
            template,
        } = strukt;
        if lhs.is_empty() {
            return Ok(());
        }
        let lhs = self.operand(*lhs, loc)?;
        let kind = template.kind;
        let mut rhs = if let Some(rest) = rest {
            self.operand(*rest, loc)?
        } else {
            self.allocate_literal(template, loc)
        };
        for field in fields {
            let field_value = self.operand(field.value, loc)?;
            let path = Path::default().member(&field.member);
            let (field_range, _) = bit_range(kind, &path)?;
            let reg = self.allocate_register(&kind, loc);
            self.lop(
                tl::OpCode::Splice(tl::Splice {
                    lhs: reg,
                    orig: rhs,
                    bit_range: field_range,
                    value: field_value,
                }),
                loc,
            );
            rhs = reg;
        }
        self.lop(tl::OpCode::Assign(tl::Assign { lhs, rhs }), loc);
        Ok(())
    }
    fn make_tuple(&mut self, tuple: &hf::Tuple, loc: SourceLocation) -> Result<()> {
        let hf::Tuple { lhs, fields } = tuple;
        if lhs.is_empty() {
            return Ok(());
        }
        let lhs = self.operand(*lhs, loc)?;
        let args = self.make_operand_list(fields, loc)?;
        self.lop(tl::OpCode::Concat(tl::Concat { lhs, args }), loc);
        Ok(())
    }
    fn make_xshr(&mut self, lhs: Operand, arg: Operand, shift: usize, loc: SourceLocation) {
        // First apply the right shift operation
        let count = b8(shift as u128);
        let right_shift_amount = self.allocate_literal(&count.typed_bits(), loc);
        let operand_bits = self.operand_bit_width(arg);
        let operand_signed = self.operand_is_signed(arg);
        let arg_shifted = if operand_signed {
            self.allocate_register_with_register_kind(&RegisterKind::Signed(operand_bits), loc)
        } else {
            self.allocate_register_with_register_kind(&RegisterKind::Unsigned(operand_bits), loc)
        };
        self.lop(
            tl::OpCode::Binary(tl::Binary {
                lhs: arg_shifted,
                op: tl::AluBinary::Shr,
                arg1: arg,
                arg2: right_shift_amount,
            }),
            loc,
        );
        // Now resize the result into the LHS
        self.lop(
            tl::OpCode::Cast(tl::Cast {
                lhs,
                arg: arg_shifted,
                len: operand_bits.saturating_sub(shift),
                kind: CastKind::Resize,
            }),
            loc,
        )
    }
    fn make_xshl(&mut self, lhs: Operand, arg: Operand, count: usize, loc: SourceLocation) {
        // First pad the operand by the shift count
        let arg_len = self.operand_bit_width(arg);
        let arg_padded = if self.operand_is_signed(arg) {
            self.allocate_register_with_register_kind(&RegisterKind::Signed(arg_len + count), loc)
        } else {
            self.allocate_register_with_register_kind(&RegisterKind::Unsigned(arg_len + count), loc)
        };
        // Now we resize cast the argument into this larger register
        self.lop(
            tl::OpCode::Cast(tl::Cast {
                lhs: arg_padded,
                arg,
                len: arg_len + count,
                kind: CastKind::Resize,
            }),
            loc,
        );
        let count = b8(count as u128);
        let left_shift_amount = self.allocate_literal(&count.typed_bits(), loc);
        // Now we issue the shl operation (lossy)
        self.lop(
            tl::OpCode::Binary(tl::Binary {
                lhs,
                op: tl::AluBinary::Shl,
                arg1: arg_padded,
                arg2: left_shift_amount,
            }),
            loc,
        );
    }
    fn make_xsgn(&mut self, lhs: Operand, arg: Operand, loc: SourceLocation) {
        // The argument must be unsigned.
        // First pad the width by 1 bit
        let arg_len = self.operand_bit_width(arg);
        let arg_padded =
            self.allocate_register_with_register_kind(&RegisterKind::Unsigned(arg_len + 1), loc);
        // Now we resize cast the argument into this larger register
        self.lop(
            tl::OpCode::Cast(tl::Cast {
                lhs: arg_padded,
                arg,
                len: arg_len + 1,
                kind: CastKind::Resize,
            }),
            loc,
        );
        // Next, we cast it as signed in this larger size
        self.lop(
            tl::OpCode::Cast(tl::Cast {
                lhs,
                arg: arg_padded,
                len: arg_len + 1,
                kind: CastKind::Signed,
            }),
            loc,
        );
    }
    fn make_xneg(&mut self, lhs: Operand, arg: Operand, loc: SourceLocation) {
        // First pad the width by 1 bit
        let arg_len = self.operand_bit_width(arg);
        let mut arg_padded = if self.operand_is_signed(arg) {
            self.allocate_register_with_register_kind(&RegisterKind::Signed(arg_len + 1), loc)
        } else {
            self.allocate_register_with_register_kind(&RegisterKind::Unsigned(arg_len + 1), loc)
        };
        // Now we resize cast the argument into this larger register
        self.lop(
            tl::OpCode::Cast(tl::Cast {
                lhs: arg_padded,
                arg,
                len: arg_len + 1,
                kind: CastKind::Resize,
            }),
            loc,
        );
        // We need an extra step if the argument is unsigned
        if !self.operand_is_signed(arg) {
            let padded_and_signed =
                self.allocate_register_with_register_kind(&RegisterKind::Signed(arg_len + 1), loc);
            self.lop(
                tl::OpCode::Cast(tl::Cast {
                    lhs: padded_and_signed,
                    arg: arg_padded,
                    len: arg_len + 1,
                    kind: CastKind::Signed,
                }),
                loc,
            );
            arg_padded = padded_and_signed;
        }
        // Now we can negate the value
        self.lop(
            tl::OpCode::Unary(tl::Unary {
                lhs,
                op: tl::AluUnary::Neg,
                arg1: arg_padded,
            }),
            loc,
        );
    }
    fn make_unary(&mut self, unary: &hf::Unary, loc: SourceLocation) -> Result<()> {
        let hf::Unary { lhs, op, arg1 } = *unary;
        if lhs.is_empty() {
            return Ok(());
        }
        let lhs = self.operand(lhs, loc)?;
        let arg1 = self.operand(arg1, loc)?;
        let mut unop = |op| self.lop(tl::OpCode::Unary(tl::Unary { lhs, op, arg1 }), loc);
        match op {
            hf::AluUnary::Neg => unop(tl::AluUnary::Neg),
            hf::AluUnary::Not => unop(tl::AluUnary::Not),
            hf::AluUnary::All => unop(tl::AluUnary::All),
            hf::AluUnary::Any => unop(tl::AluUnary::Any),
            hf::AluUnary::Xor => unop(tl::AluUnary::Xor),
            hf::AluUnary::Signed => unop(tl::AluUnary::Signed),
            hf::AluUnary::Unsigned => unop(tl::AluUnary::Unsigned),
            hf::AluUnary::Val => unop(tl::AluUnary::Val),
            hf::AluUnary::XExt(_) => {
                let lhs_len = self.operand_bit_width(lhs);
                self.lop(
                    tl::OpCode::Cast(tl::Cast {
                        lhs,
                        arg: arg1,
                        len: lhs_len,
                        kind: CastKind::Resize,
                    }),
                    loc,
                );
            }
            hf::AluUnary::XShl(cnt) => self.make_xshl(lhs, arg1, cnt, loc),
            hf::AluUnary::XShr(cnt) => self.make_xshr(lhs, arg1, cnt, loc),
            hf::AluUnary::XNeg => self.make_xneg(lhs, arg1, loc),
            hf::AluUnary::XSgn => self.make_xsgn(lhs, arg1, loc),
        };
        Ok(())
    }
    fn translate(mut self) -> Result<Self> {
        for lop in &self.object.ops {
            let loc = lop.loc;
            match &lop.op {
                hf::OpCode::Array(array) => {
                    self.make_array(array, loc)?;
                }
                hf::OpCode::AsBits(cast) => {
                    self.make_as_bits(cast, loc)?;
                }
                hf::OpCode::AsSigned(cast) => {
                    self.make_as_signed(cast, loc)?;
                }
                hf::OpCode::Assign(assign) => {
                    self.make_assign(assign, loc)?;
                }
                hf::OpCode::Binary(binary) => {
                    self.make_binary(binary, loc)?;
                }
                hf::OpCode::Case(case) => {
                    self.make_case(case, loc)?;
                }
                hf::OpCode::Comment(comment) => {
                    self.lop(tl::OpCode::Comment(comment.clone()), loc);
                }
                hf::OpCode::Enum(enumerate) => {
                    self.make_enum(enumerate, loc)?;
                }
                hf::OpCode::Exec(exec) => {
                    self.make_exec(exec, loc)?;
                }
                hf::OpCode::Index(index) => {
                    self.make_index(index, loc)?;
                }
                hf::OpCode::Noop => {}
                hf::OpCode::Resize(cast) => {
                    self.make_resize(cast, loc)?;
                }
                hf::OpCode::Repeat(repeat) => {
                    self.make_repeat(repeat, loc)?;
                }
                hf::OpCode::Retime(retime) => {
                    self.make_retime(retime, loc)?;
                }
                hf::OpCode::Select(select) => {
                    self.make_select(select, loc)?;
                }
                hf::OpCode::Splice(splice) => {
                    self.make_splice(splice, loc)?;
                }
                hf::OpCode::Struct(strukt) => {
                    self.make_struct(strukt, loc)?;
                }
                hf::OpCode::Tuple(tuple) => {
                    self.make_tuple(tuple, loc)?;
                }
                hf::OpCode::Unary(unary) => {
                    self.make_unary(unary, loc)?;
                }
                hf::OpCode::Wrap(wrap) => {
                    self.make_wrap(wrap, loc)?;
                }
            }
        }
        Ok(self)
    }
}

fn compile_rtl(object: &rhif::Object) -> Result<rtl::object::Object> {
    let mut compiler = RTLCompiler::new(object).translate()?;
    let fallback = object.symbols.source_set.fallback(object.fn_id);
    let arguments = object
        .arguments
        .iter()
        .map(|x| {
            if object.kind[x].is_empty() {
                None
            } else if let Ok(Operand::Register(reg_id)) =
                compiler.operand(Slot::Register(*x), fallback)
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
