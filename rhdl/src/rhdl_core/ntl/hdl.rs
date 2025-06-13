use std::collections::BTreeSet;

use crate::prelude::Module;
use crate::prelude::RHDLError;
use crate::rhdl_core::ast::source::source_location::SourceLocation;
use crate::rhdl_core::error::rhdl_error;
use crate::rhdl_core::hdl;
use crate::rhdl_core::hdl::ast::CaseItem;
use crate::rhdl_core::ntl::error::NetListError;
use crate::rhdl_core::ntl::error::NetListICE;
use crate::rhdl_core::ntl::remap::visit_operands;
use crate::rhdl_core::ntl::spec;
use crate::rhdl_core::ntl::spec::CaseEntry;
use crate::rhdl_core::ntl::spec::Operand;
use crate::rhdl_core::ntl::spec::VectorOp;
use crate::rhdl_core::ntl::Object;
use crate::rhdl_core::rtl::spec::AluBinary;
use crate::rhdl_core::rtl::spec::AluUnary;

struct NetListHDLBuilder<'a> {
    ntl: &'a Object,
    body: Vec<hdl::ast::Statement>,
    decls: Vec<hdl::ast::Declaration>,
    name: String,
}

fn opex(operand: Operand) -> hdl::ast::Expression {
    use hdl::ast::id;
    match operand {
        Operand::One => id("1'b1"),
        Operand::Zero => id("1'b0"),
        Operand::X => id("1'bX"),
        Operand::Register(rid) => id(&format!("r{}", rid.raw())),
    }
}

fn opex_v(operands: &[Operand]) -> hdl::ast::Expression {
    hdl::ast::concatenate(operands.iter().copied().map(opex).collect())
}

impl<'a> NetListHDLBuilder<'a> {
    fn new(name: &'_ str, ntl: &'a Object) -> Self {
        Self {
            ntl,
            body: vec![],
            decls: vec![],
            name: name.into(),
        }
    }
    fn raise_ice(&self, cause: NetListICE, location: Option<SourceLocation>) -> RHDLError {
        rhdl_error(NetListError {
            cause,
            src: self.ntl.code.source(),
            elements: location
                .map(|loc| self.ntl.code.span(loc).into())
                .into_iter()
                .collect(),
        })
    }
    fn reg(&self, operand: Operand, location: Option<SourceLocation>) -> Result<String, RHDLError> {
        if let Some(rid) = operand.reg() {
            Ok(format!("r{}", rid.raw()))
        } else {
            Err(self.raise_ice(NetListICE::ExpectedRegisterNotConstant, location))
        }
    }
    fn reg_v(
        &self,
        operands: &[Operand],
        location: Option<SourceLocation>,
    ) -> Result<String, RHDLError> {
        let args = operands
            .iter()
            .map(|op| self.reg(&op, location))
            .collect::<Result<Vec<String>, RHDLError>>()?;
        Ok(format!("{{ {} }}", args.join(",")))
    }
    fn stmt(&mut self, statement: hdl::ast::Statement) {
        self.body.push(statement);
    }
    fn select_op(
        &mut self,
        op: &spec::Select,
        location: Option<SourceLocation>,
    ) -> Result<(), RHDLError> {
        let target = self.reg(op.lhs, location)?;
        self.stmt(hdl::ast::assign(
            &target,
            hdl::ast::select(opex(op.selector), opex(op.true_case), opex(op.false_case)),
        ));
        Ok(())
    }
    fn assign_op(
        &mut self,
        op: &spec::Assign,
        location: Option<SourceLocation>,
    ) -> Result<(), RHDLError> {
        let target = self.reg(op.lhs, location)?;
        self.stmt(hdl::ast::assign(&target, opex(op.rhs)));
        Ok(())
    }
    fn binary_op(
        &mut self,
        op: &spec::Binary,
        location: Option<SourceLocation>,
    ) -> Result<(), RHDLError> {
        let target = self.reg(op.lhs, location)?;
        let alu = match op.op {
            spec::BinaryOp::Xor => AluBinary::BitXor,
            spec::BinaryOp::And => AluBinary::BitAnd,
            spec::BinaryOp::Or => AluBinary::BitOr,
        };
        let expr = hdl::ast::binary(alu, opex(op.arg1), opex(op.arg2));
        self.stmt(hdl::ast::assign(&target, expr));
        Ok(())
    }
    fn vector_op(
        &mut self,
        op: &spec::Vector,
        location: Option<SourceLocation>,
    ) -> Result<(), RHDLError> {
        let target = self.reg_v(&op.lhs, location)?;
        let alu = match op.op {
            VectorOp::Add => AluBinary::Add,
            VectorOp::Sub => AluBinary::Sub,
            VectorOp::Mul => AluBinary::Mul,
            VectorOp::Eq => AluBinary::Eq,
            VectorOp::Ne => AluBinary::Ne,
            VectorOp::Lt => AluBinary::Lt,
            VectorOp::Le => AluBinary::Le,
            VectorOp::Gt => AluBinary::Gt,
            VectorOp::Ge => AluBinary::Ge,
            VectorOp::Shl => AluBinary::Shl,
            VectorOp::Shr => AluBinary::Shr,
        };
        let arg1 = opex_v(&op.arg1);
        let arg2 = opex_v(&op.arg2);
        let arg1 = if op.signed {
            hdl::ast::unary(crate::rhdl_core::rtl::spec::AluUnary::Signed, arg1)
        } else {
            arg1
        };
        let arg2 = if op.signed {
            hdl::ast::unary(crate::rhdl_core::rtl::spec::AluUnary::Signed, arg2)
        } else {
            arg2
        };
        self.stmt(hdl::ast::assign(&target, hdl::ast::binary(alu, arg1, arg2)));
        Ok(())
    }
    fn not_op(
        &mut self,
        op: &spec::Not,
        location: Option<SourceLocation>,
    ) -> Result<(), RHDLError> {
        let target = self.reg(op.lhs, location)?;
        self.stmt(hdl::ast::assign(
            &target,
            hdl::ast::unary(crate::rhdl_core::rtl::spec::AluUnary::Not, opex(op.arg)),
        ));
        Ok(())
    }
    fn case_op(
        &mut self,
        op: &spec::Case,
        location: Option<SourceLocation>,
    ) -> Result<(), RHDLError> {
        let target = self.reg(op.lhs, location)?;
        let discriminant = opex_v(&op.discriminant);
        let table = op
            .entries
            .iter()
            .map(|(entry, operand)| {
                let item = match entry {
                    CaseEntry::Literal(value) => CaseItem::Literal(value.clone()),
                    CaseEntry::WildCard => CaseItem::Wild,
                };
                let statement = hdl::ast::assign(&target, opex(*operand));
                (item, statement)
            })
            .collect();
        self.stmt(hdl::ast::case(discriminant, table));
        Ok(())
    }
    fn unary_op(
        &mut self,
        op: &spec::Unary,
        location: Option<SourceLocation>,
    ) -> Result<(), RHDLError> {
        let target = self.reg_v(&op.lhs, location)?;
        let arg = opex_v(&op.arg);
        let alu = match op.op {
            spec::UnaryOp::All => AluUnary::All,
            spec::UnaryOp::Any => AluUnary::Any,
            spec::UnaryOp::Neg => AluUnary::Neg,
            spec::UnaryOp::Xor => AluUnary::Xor,
        };
        self.stmt(hdl::ast::assign(&target, hdl::ast::unary(alu, arg)));
        Ok(())
    }
    fn op_code(
        &mut self,
        op: &spec::OpCode,
        location: Option<SourceLocation>,
    ) -> Result<(), RHDLError> {
        match op {
            spec::OpCode::Noop => Ok(()),
            spec::OpCode::Assign(assign) => self.assign_op(assign, location),
            spec::OpCode::Binary(binary) => self.binary_op(binary, location),
            spec::OpCode::Vector(vector) => self.vector_op(vector, location),
            spec::OpCode::Case(case) => self.case_op(case, location),
            spec::OpCode::Comment(comment) => {
                self.stmt(hdl::ast::comment(comment));
                Ok(())
            }
            spec::OpCode::DynamicIndex(dynamic_index) => todo!(),
            spec::OpCode::DynamicSplice(dynamic_splice) => todo!(),
            spec::OpCode::Select(select) => self.select_op(select, location),
            spec::OpCode::Not(not) => self.not_op(not, location),
            spec::OpCode::Dff(dff) => todo!(),
            spec::OpCode::BlackBox(black_box) => todo!(),
            spec::OpCode::Unary(unary) => self.unary_op(unary, location),
        }
    }
    fn build(mut self) -> Result<Module, RHDLError> {
        let ports = self
            .ntl
            .inputs
            .iter()
            .enumerate()
            .flat_map(|(ndx, x)| {
                (!x.is_empty()).then(|| {
                    hdl::ast::port(
                        &format!("arg_{ndx}"),
                        hdl::ast::Direction::Input,
                        hdl::ast::HDLKind::Wire,
                        hdl::ast::unsigned_width(x.len()),
                    )
                })
            })
            .chain(std::iter::once(hdl::ast::port(
                "out",
                hdl::ast::Direction::Output,
                hdl::ast::HDLKind::Reg,
                hdl::ast::unsigned_width(self.ntl.outputs.len()),
            )))
            .collect();
        let mut registers = BTreeSet::default();
        for lop in &self.ntl.ops {
            visit_operands(&lop.op, |op| {
                if let Some(reg) = op.reg() {
                    registers.insert(reg);
                }
            });
        }
        let mut declarations = registers
            .iter()
            .map(|ndx| {
                hdl::ast::declaration(hdl::ast::HDLKind::Reg, &format!("r{}", ndx.raw()), 1, None)
            })
            .collect::<Vec<_>>();
    }
}

pub(crate) fn generate_hdl(module_name: &str, ntl: &Object) -> Result<Module, RHDLError> {
    NetListHDLBuilder::new(module_name, ntl).build()
}
