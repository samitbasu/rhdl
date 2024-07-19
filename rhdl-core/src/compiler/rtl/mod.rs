use std::collections::BTreeMap;

use crate::ast::ast_impl::{FunctionId, NodeId};
use crate::rhif::spec::{ExternalFunction, FuncId, Slot};
use crate::rtl::spec::{LiteralId, OpCode, Operand, RegisterId};
use crate::TypedBits;
use crate::{rhif, RHDLError};
use crate::{rtl, Kind};

use crate::rhif::spec as hf;
use crate::rtl::spec as tl;

use super::mir::error::{RHDLCompileError, ICE};

type Result<T> = std::result::Result<T, RHDLError>;

pub enum BitString {
    Signed(Vec<bool>),
    Unsigned(Vec<bool>),
}

impl From<&TypedBits> for BitString {
    fn from(bits: &TypedBits) -> Self {
        if bits.kind.is_signed() {
            BitString::Signed(bits.bits.clone())
        } else {
            BitString::Unsigned(bits.bits.clone())
        }
    }
}

pub enum RegisterKind {
    Signed(usize),
    Unsigned(usize),
}

struct RTLCompiler<'a> {
    object: &'a rhif::object::Object,
    literals: BTreeMap<LiteralId, BitString>,
    registers: BTreeMap<RegisterId, RegisterKind>,
    operand_map: BTreeMap<Operand, Slot>,
    reverse_operand_map: BTreeMap<Slot, Operand>,
    externals: BTreeMap<FuncId, ExternalFunction>,
    ops: Vec<rtl::object::LocatedOpCode>,
    arguments: Vec<RegisterId>,
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
            arguments: Default::default(),
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
        RHDLError::RHDLInternalCompilerError(Box::new(RHDLCompileError {
            cause,
            src: self.object.symbols.source.source.clone(),
            err_span: self.object.symbols.node_span(id).into(),
        }))
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
    fn make_enum(&mut self, enumerate: &hf::Enum, id: NodeId) -> Result<()> {
        let hf::Enum {
            lhs,
            fields,
            template,
        } = enumerate;
        if lhs.is_empty() {
            return Ok(());
        }
        let Kind::Enum(kind) = &template.kind else {
            return Err(self.raise_ice(
                ICE::ExpectedEnumTemplate {
                    kind: template.kind.clone(),
                },
                id,
            ));
        };
        let discriminant = template.discriminant()?;
        let discriminant = Operand::Literal(self.allocate_literal(&discriminant));
        let mut args = fields
            .iter()
            .filter_map(|x| {
                if x.value.is_empty() {
                    None
                } else {
                    Some(self.operand(x.value, id))
                }
            })
            .collect::<Result<Vec<_>>>()?;
        match kind.discriminant_layout.alignment {
            crate::DiscriminantAlignment::Msb => args.push(discriminant),
            crate::DiscriminantAlignment::Lsb => args.insert(0, discriminant),
        };
        let lhs = self.operand(*lhs, id)?;
        self.ops
            .push((tl::OpCode::Concat(tl::Concat { lhs, args }), id).into());
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
                _ => {
                    todo!()
                }
            }
        }
        todo!()
    }
}

pub fn compile_rtl(object: rhif::object::Object) -> rtl::object::Object {
    let mut compiler = RTLCompiler::new(&object);
    rtl::object::Object {
        symbols: object.symbols,
        literals: object.literals,
        operand_map: Default::default(),
        return_register: Default::default(),
        externals: Default::default(),
        ops: Default::default(),
        arguments: object.arguments,
        name: object.name,
        fn_id: object.fn_id,
    }
}
