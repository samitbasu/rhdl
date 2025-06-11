use std::collections::HashMap;

use crate::core::ntl;
use crate::core::rtl;
use crate::prelude::BitX;
use crate::rhdl_core::ast::source::source_location::SourceLocation;
use crate::rhdl_core::ntl::spec::Assign;
use crate::rhdl_core::ntl::spec::Binary;
use crate::rhdl_core::ntl::spec::DynamicIndex;
use crate::rhdl_core::ntl::spec::DynamicSplice;
use crate::rhdl_core::ntl::spec::Not;
use crate::rhdl_core::ntl::spec::RegisterId;
use crate::rhdl_core::ntl::spec::Select;
use crate::rhdl_core::ntl::spec::Unary;
use crate::rhdl_core::ntl::spec::Vector;

use ntl::spec as bt;
use rtl::spec as tl;

struct NtlBuilder<'a> {
    object: &'a rtl::Object,
    btl: ntl::object::Object,
    operand_map: HashMap<tl::Operand, Vec<bt::Operand>>,
    reg_count: u32,
}

pub fn build_ntl_from_rtl(object: &rtl::Object) -> ntl::object::Object {
    let mut bob = NtlBuilder::new(object);
    for lop in &object.ops {
        bob.op(lop);
    }
    let inputs = object
        .arguments
        .iter()
        .map(|x| match x {
            Some(id) => bob
                .operand(tl::Operand::Register(*id))
                .iter()
                .map(|x| match x {
                    bt::Operand::Register(rid) => *rid,
                    _ => panic!("Argument mapped to a constant!"),
                })
                .collect(),
            None => vec![],
        })
        .collect();
    let outputs = bob.operand(object.return_register);
    bob.btl.inputs = inputs;
    bob.btl.outputs = outputs;
    bob.btl
}

impl<'a> NtlBuilder<'a> {
    fn new(object: &'a rtl::Object) -> Self {
        let btl = ntl::object::Object {
            code: object.symbols.source_set.clone(),
            name: object.name.clone(),
            ..Default::default()
        };
        Self {
            object,
            btl,
            operand_map: HashMap::default(),
            reg_count: 0,
        }
    }
    fn reg(&mut self) -> bt::RegisterId {
        let num = self.reg_count;
        self.reg_count += 1;
        RegisterId::new(num)
    }
    fn push(&mut self, loc: SourceLocation, op: bt::OpCode) {
        self.btl.ops.push(ntl::object::LocatedOpCode { loc, op });
    }
    fn op(&mut self, lop: &rtl::object::LocatedOpCode) {
        let loc = lop.loc;
        match &lop.op {
            tl::OpCode::Noop => {}
            tl::OpCode::Assign(assign) => self.build_assign(loc, assign),
            tl::OpCode::Binary(binary) => self.build_binary(loc, binary),
            tl::OpCode::Case(case) => self.build_case(loc, case),
            tl::OpCode::Cast(cast) => self.build_cast(loc, cast),
            tl::OpCode::Comment(comment) => self.push(loc, bt::OpCode::Comment(comment.clone())),
            tl::OpCode::Concat(concat) => self.build_concat(loc, concat),
            tl::OpCode::DynamicIndex(dynamic_index) => self.build_dynamic_index(loc, dynamic_index),
            tl::OpCode::DynamicSplice(dynamic_splice) => {
                self.build_dynamic_splice(loc, dynamic_splice)
            }
            tl::OpCode::Index(index) => self.build_index(loc, index),
            tl::OpCode::Select(select) => self.build_select(loc, select),
            tl::OpCode::Splice(splice) => self.build_splice(loc, splice),
            tl::OpCode::Unary(unary) => self.build_unary(loc, unary),
        }
    }
    fn operand(&mut self, operand: tl::Operand) -> Vec<bt::Operand> {
        if let Some(port) = self.operand_map.get(&operand) {
            return port.clone();
        }
        let ndx = match operand {
            tl::Operand::Literal(literal_id) => {
                let bs = &self.object.literals[&literal_id];
                bs.bits()
                    .iter()
                    .map(|b| match b {
                        BitX::Zero => bt::Operand::Zero,
                        BitX::One => bt::Operand::One,
                        BitX::X => bt::Operand::X,
                    })
                    .collect::<Vec<_>>()
            }
            tl::Operand::Register(register_id) => {
                let reg = self.object.register_kind[&register_id];
                (0..reg.len())
                    .map(|_| bt::Operand::Register(self.reg()))
                    .collect::<Vec<_>>()
            }
        };
        self.operand_map.insert(operand, ndx.clone());
        ndx
    }
    fn build_assign(&mut self, loc: SourceLocation, assign: &tl::Assign) {
        let rhs = self.operand(assign.rhs);
        let lhs = self.operand(assign.lhs);
        for (&lhs, &rhs) in lhs.iter().zip(rhs.iter()) {
            self.push(loc, bt::OpCode::Assign(Assign { lhs, rhs }));
        }
    }
    fn build_binary(&mut self, loc: SourceLocation, binary: &tl::Binary) {
        let arg1 = self.operand(binary.arg1);
        let arg2 = self.operand(binary.arg2);
        let lhs = self.operand(binary.lhs);
        let signed = self.object.kind(binary.lhs).is_signed();
        match classify_binary(binary.op) {
            BinOpClass::Bitwise(binop) => {
                for (&lhs, (&arg1, &arg2)) in lhs.iter().zip(arg1.iter().zip(arg2.iter())) {
                    self.push(
                        loc,
                        bt::OpCode::Binary(Binary {
                            op: binop,
                            lhs,
                            arg1,
                            arg2,
                        }),
                    );
                }
            }
            BinOpClass::Vector(vectorop) => self.push(
                loc,
                bt::OpCode::Vector(Vector {
                    op: vectorop,
                    lhs,
                    arg1,
                    arg2,
                    signed,
                }),
            ),
        }
    }
    fn build_case(&mut self, loc: SourceLocation, case: &tl::Case) {
        let lhs = self.operand(case.lhs);
        let discriminant = self.operand(case.discriminant);
        let table = case
            .table
            .iter()
            .map(|(x, _)| {
                if let tl::CaseArgument::Literal(lit_id) = x {
                    bt::CaseEntry::Literal(self.object.literals[lit_id].clone())
                } else {
                    bt::CaseEntry::WildCard
                }
            })
            .collect::<Vec<_>>();
        let arguments = case
            .table
            .iter()
            .map(|(_, x)| self.operand(*x))
            .collect::<Vec<_>>();
        let num_bits = lhs.len();
        for bit in 0..num_bits {
            let entries = table
                .iter()
                .cloned()
                .zip(arguments.iter())
                .map(|(case_entry, argument)| (case_entry, argument[bit]))
                .collect();
            self.push(
                loc,
                bt::OpCode::Case(bt::Case {
                    lhs: lhs[bit],
                    discriminant: discriminant.clone(),
                    entries,
                }),
            );
        }
    }
    fn build_cast(&mut self, loc: SourceLocation, cast: &tl::Cast) {
        let lhs = self.operand(cast.lhs);
        let arg = self.operand(cast.arg);
        for (&lhs, &rhs) in lhs.iter().zip(arg.iter()) {
            self.push(loc, bt::OpCode::Assign(Assign { lhs, rhs }));
        }
        let lhs_signed = self.object.kind(cast.lhs).is_signed();
        let use_unsigned = matches!(cast.kind, tl::CastKind::Unsigned)
            || (matches!(cast.kind, tl::CastKind::Resize) && !lhs_signed);
        if use_unsigned {
            for &lhs in lhs.iter().skip(arg.len()) {
                self.push(
                    loc,
                    bt::OpCode::Assign(Assign {
                        lhs,
                        rhs: bt::Operand::Zero,
                    }),
                );
            }
        } else {
            let &msb = arg.last().unwrap();
            for &lhs in lhs.iter().skip(arg.len()) {
                self.push(loc, bt::OpCode::Assign(Assign { lhs, rhs: msb }));
            }
        }
    }
    fn build_concat(&mut self, loc: SourceLocation, concat: &tl::Concat) {
        let lhs = self.operand(concat.lhs);
        let args = concat
            .args
            .iter()
            .map(|x| self.operand(*x))
            .collect::<Vec<_>>();
        for (&lhs, &rhs) in lhs.iter().zip(args.iter().flatten()) {
            self.push(loc, bt::OpCode::Assign(Assign { lhs, rhs }));
        }
    }
    fn build_dynamic_index(&mut self, loc: SourceLocation, dynamic_index: &tl::DynamicIndex) {
        let lhs = self.operand(dynamic_index.lhs);
        let arg = self.operand(dynamic_index.arg);
        let offset = self.operand(dynamic_index.offset);
        self.push(
            loc,
            bt::OpCode::DynamicIndex(DynamicIndex { lhs, arg, offset }),
        )
    }
    fn build_dynamic_splice(&mut self, loc: SourceLocation, dynamic_splice: &tl::DynamicSplice) {
        let lhs = self.operand(dynamic_splice.lhs);
        let arg = self.operand(dynamic_splice.arg);
        let offset = self.operand(dynamic_splice.offset);
        let value = self.operand(dynamic_splice.value);
        self.push(
            loc,
            bt::OpCode::DynamicSplice(DynamicSplice {
                lhs,
                arg,
                offset,
                value,
            }),
        )
    }
    fn build_index(&mut self, loc: SourceLocation, index: &tl::Index) {
        let lhs = self.operand(index.lhs);
        let arg = self.operand(index.arg);
        for (&lhs, &rhs) in lhs.iter().zip(arg.iter().skip(index.bit_range.start)) {
            self.push(loc, bt::OpCode::Assign(Assign { lhs, rhs }));
        }
    }
    fn build_select(&mut self, loc: SourceLocation, select: &tl::Select) {
        let lhs = self.operand(select.lhs);
        let cond = self.operand(select.cond);
        let true_case = self.operand(select.true_value);
        let false_case = self.operand(select.false_value);
        for (&lhs, (&true_case, &false_case)) in
            lhs.iter().zip(true_case.iter().zip(false_case.iter()))
        {
            self.push(
                loc,
                bt::OpCode::Select(Select {
                    lhs,
                    selector: cond[0],
                    true_case,
                    false_case,
                }),
            );
        }
    }
    fn build_splice(&mut self, loc: SourceLocation, splice: &tl::Splice) {
        let lhs = self.operand(splice.lhs);
        let orig = self.operand(splice.orig);
        let value = self.operand(splice.value);
        let lsb_iter = orig.iter().take(splice.bit_range.start);
        let center_iter = value.iter();
        let msb_iter = orig.iter().skip(splice.bit_range.end);
        let rhs = lsb_iter.chain(center_iter).chain(msb_iter);
        for (&lhs, &rhs) in lhs.iter().zip(rhs) {
            self.push(loc, bt::OpCode::Assign(Assign { lhs, rhs }));
        }
    }
    fn build_unary(&mut self, loc: SourceLocation, unary: &tl::Unary) {
        let lhs = self.operand(unary.lhs);
        let arg = self.operand(unary.arg1);
        match classify_unary(unary.op) {
            UnaryOpClass::Not => {
                for (&lhs, arg) in lhs.iter().zip(arg) {
                    self.push(loc, bt::OpCode::Not(Not { lhs, arg }))
                }
            }
            UnaryOpClass::Copy => {
                for (&lhs, arg) in lhs.iter().zip(arg) {
                    self.push(loc, bt::OpCode::Assign(Assign { lhs, rhs: arg }))
                }
            }
            UnaryOpClass::Unary(unary_op) => self.push(
                loc,
                bt::OpCode::Unary(Unary {
                    op: unary_op,
                    lhs,
                    arg,
                }),
            ),
        }
    }
}

enum UnaryOpClass {
    Not,
    Copy,
    Unary(bt::UnaryOp),
}

fn classify_unary(op: tl::AluUnary) -> UnaryOpClass {
    match op {
        tl::AluUnary::Neg => UnaryOpClass::Unary(bt::UnaryOp::Neg),
        tl::AluUnary::Not => UnaryOpClass::Not,
        tl::AluUnary::All => UnaryOpClass::Unary(bt::UnaryOp::All),
        tl::AluUnary::Any => UnaryOpClass::Unary(bt::UnaryOp::Any),
        tl::AluUnary::Xor => UnaryOpClass::Unary(bt::UnaryOp::Xor),
        tl::AluUnary::Signed => UnaryOpClass::Copy,
        tl::AluUnary::Unsigned => UnaryOpClass::Copy,
        tl::AluUnary::Val => UnaryOpClass::Copy,
    }
}

enum BinOpClass {
    Bitwise(bt::BinaryOp),
    Vector(bt::VectorOp),
}

fn classify_binary(op: tl::AluBinary) -> BinOpClass {
    match op {
        tl::AluBinary::BitAnd => BinOpClass::Bitwise(bt::BinaryOp::And),
        tl::AluBinary::BitXor => BinOpClass::Bitwise(bt::BinaryOp::Xor),
        tl::AluBinary::BitOr => BinOpClass::Bitwise(bt::BinaryOp::Or),
        tl::AluBinary::Add => BinOpClass::Vector(bt::VectorOp::Add),
        tl::AluBinary::Sub => BinOpClass::Vector(bt::VectorOp::Sub),
        tl::AluBinary::Eq => BinOpClass::Vector(bt::VectorOp::Eq),
        tl::AluBinary::Mul => BinOpClass::Vector(bt::VectorOp::Mul),
        tl::AluBinary::Shl => BinOpClass::Vector(bt::VectorOp::Shl),
        tl::AluBinary::Shr => BinOpClass::Vector(bt::VectorOp::Shr),
        tl::AluBinary::Lt => BinOpClass::Vector(bt::VectorOp::Lt),
        tl::AluBinary::Le => BinOpClass::Vector(bt::VectorOp::Le),
        tl::AluBinary::Ne => BinOpClass::Vector(bt::VectorOp::Ne),
        tl::AluBinary::Ge => BinOpClass::Vector(bt::VectorOp::Ge),
        tl::AluBinary::Gt => BinOpClass::Vector(bt::VectorOp::Gt),
    }
}
