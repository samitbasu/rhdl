use std::collections::BTreeSet;
use std::collections::HashSet;

use crate::prelude::Module;
use crate::prelude::RHDLError;
use crate::rhdl_core::ast::source::source_location::SourceLocation;
use crate::rhdl_core::error::rhdl_error;
use crate::rhdl_core::hdl;
use crate::rhdl_core::hdl::ast;
use crate::rhdl_core::hdl::ast::CaseItem;
use crate::rhdl_core::ntl::error::NetListError;
use crate::rhdl_core::ntl::error::NetListICE;
use crate::rhdl_core::ntl::remap::visit_operands;
use crate::rhdl_core::ntl::remap::Sense;
use crate::rhdl_core::ntl::spec;
use crate::rhdl_core::ntl::spec::CaseEntry;
use crate::rhdl_core::ntl::spec::DynamicIndex;
use crate::rhdl_core::ntl::spec::DynamicSplice;
use crate::rhdl_core::ntl::spec::Operand;
use crate::rhdl_core::ntl::spec::VectorOp;
use crate::rhdl_core::ntl::Object;
use crate::rhdl_core::rtl::spec::AluBinary;
use crate::rhdl_core::rtl::spec::AluUnary;

struct NetListHDLBuilder<'a> {
    ntl: &'a Object,
    body: Vec<ast::Statement>,
    decls: Vec<ast::Declaration>,
    name: String,
    temporary_counter: usize,
}

fn opex(operand: Operand) -> ast::Expression {
    use ast::id;
    match operand {
        Operand::One => id("1'b1"),
        Operand::Zero => id("1'b0"),
        Operand::X => id("1'bX"),
        Operand::Register(rid) => id(&format!("r{}", rid.raw())),
    }
}

fn opex_v(operands: &[Operand]) -> ast::Expression {
    ast::concatenate(operands.iter().rev().copied().map(opex).collect())
}

impl<'a> NetListHDLBuilder<'a> {
    fn new(name: &'_ str, ntl: &'a Object) -> Self {
        Self {
            ntl,
            body: vec![],
            decls: vec![],
            name: name.into(),
            temporary_counter: 0,
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
            .rev()
            .map(|&op| self.reg(op, location))
            .collect::<Result<Vec<String>, RHDLError>>()?;
        Ok(format!("{{ {} }}", args.join(",")))
    }
    fn stmt(&mut self, statement: ast::Statement) {
        self.body.push(statement);
    }
    fn select_op(
        &mut self,
        op: &spec::Select,
        location: Option<SourceLocation>,
    ) -> Result<(), RHDLError> {
        let target = self.reg(op.lhs, location)?;
        self.stmt(ast::assign(
            &target,
            ast::select(opex(op.selector), opex(op.true_case), opex(op.false_case)),
        ));
        Ok(())
    }
    fn assign_op(
        &mut self,
        op: &spec::Assign,
        location: Option<SourceLocation>,
    ) -> Result<(), RHDLError> {
        let target = self.reg(op.lhs, location)?;
        self.stmt(ast::assign(&target, opex(op.rhs)));
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
        let expr = ast::binary(alu, opex(op.arg1), opex(op.arg2));
        self.stmt(ast::assign(&target, expr));
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
            ast::unary(crate::rhdl_core::rtl::spec::AluUnary::Signed, arg1)
        } else {
            arg1
        };
        let arg2 = if op.signed {
            ast::unary(crate::rhdl_core::rtl::spec::AluUnary::Signed, arg2)
        } else {
            arg2
        };
        self.stmt(ast::assign(&target, ast::binary(alu, arg1, arg2)));
        Ok(())
    }
    fn not_op(
        &mut self,
        op: &spec::Not,
        location: Option<SourceLocation>,
    ) -> Result<(), RHDLError> {
        let target = self.reg(op.lhs, location)?;
        self.stmt(ast::assign(
            &target,
            ast::unary(crate::rhdl_core::rtl::spec::AluUnary::Not, opex(op.arg)),
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
                let statement = ast::assign(&target, opex(*operand));
                (item, statement)
            })
            .collect();
        self.stmt(ast::case(discriminant, table));
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
        self.stmt(ast::assign(&target, ast::unary(alu, arg)));
        Ok(())
    }
    fn dynamic_index(
        &mut self,
        op: &DynamicIndex,
        location: Option<SourceLocation>,
    ) -> Result<(), RHDLError> {
        let target = self.reg_v(&op.lhs, location)?;
        let offset = opex_v(&op.offset);
        let arg = opex_v(&op.arg);
        // The target of a dynamic index expression ( e.g., foo[expr])
        // must be a name. It cannot be an expression like ({{r0, r1..}}[expr]).  So we
        // need a temporary to assign the argument to.
        let temp_reg = format!("dyn_ndx_{}", self.temporary_counter);
        self.temporary_counter += 1;
        self.decls.push(ast::declaration(
            ast::HDLKind::Reg,
            &temp_reg,
            ast::unsigned_width(op.arg.len()),
            Some("Dynamic index temporary register".to_string()),
        ));
        self.stmt(ast::assign(&temp_reg, arg));
        self.stmt(ast::assign(
            &target,
            ast::dynamic_index(&temp_reg, offset, op.lhs.len()),
        ));
        Ok(())
    }
    fn dynamic_splice(
        &mut self,
        op: &DynamicSplice,
        location: Option<SourceLocation>,
    ) -> Result<(), RHDLError> {
        let arg = opex_v(&op.arg);
        let lhs = self.reg_v(&op.lhs, location)?;
        // Now collect the splice bits (which are the substitution)
        let value = opex_v(&op.value);
        // Now collect the offset bits
        let offset = opex_v(&op.offset);
        self.stmt(ast::dynamic_splice(
            &lhs,
            arg,
            offset,
            value,
            op.value.len(),
        ));
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
                self.stmt(ast::comment(comment));
                Ok(())
            }
            spec::OpCode::DynamicIndex(dynamic_index) => {
                self.dynamic_index(dynamic_index, location)
            }
            spec::OpCode::DynamicSplice(dynamic_splice) => {
                self.dynamic_splice(dynamic_splice, location)
            }
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
                    ast::port(
                        &format!("arg_{ndx}"),
                        ast::Direction::Input,
                        ast::HDLKind::Wire,
                        ast::unsigned_width(x.len()),
                    )
                })
            })
            .chain(std::iter::once(ast::port(
                "out",
                ast::Direction::Output,
                ast::HDLKind::Reg,
                ast::unsigned_width(self.ntl.outputs.len()),
            )))
            .collect();
        let mut registers = BTreeSet::default();
        for lop in &self.ntl.ops {
            visit_operands(&lop.op, |_sense, op| {
                if let Some(reg) = op.reg() {
                    registers.insert(reg);
                }
            });
        }
        registers.extend(self.ntl.inputs.iter().flatten());
        let mut declarations = registers
            .iter()
            .map(|ndx| {
                ast::declaration(
                    ast::HDLKind::Reg,
                    &format!("r{}", ndx.raw()),
                    ast::unsigned_width(1),
                    None,
                )
            })
            .collect::<Vec<_>>();
        let mut statements = vec![];
        let mut submodules = vec![];
        // Connect the input registers to their module input names
        for (ndx, arg) in self.ntl.inputs.iter().enumerate() {
            for (bit, &reg) in arg.iter().enumerate() {
                self.stmt(ast::assign(
                    &self.reg(Operand::Register(reg), None)?,
                    ast::index_bit(&format!("arg_{ndx}"), bit),
                ))
            }
        }
        assert!(self.ntl.black_boxes.is_empty(), "Black boxes are TODO");
        for lop in &self.ntl.ops {
            self.op_code(&lop.op, lop.loc)?;
        }
        let output_bits =
            ast::concatenate(self.ntl.outputs.iter().rev().copied().map(opex).collect());
        self.stmt(ast::assign("out", output_bits));
        // Check if any of the inputs are used by the body of the module
        let input_set = self.ntl.inputs.iter().flatten().collect::<HashSet<_>>();
        let inputs_used = self.ntl.ops.iter().any(|lop| {
            let mut uses_input = false;
            visit_operands(&lop.op, |sense, op| {
                if let Some(reg_id) = op.reg() {
                    if (sense == Sense::Read) && input_set.contains(&reg_id) {
                        uses_input = true;
                    }
                }
            });
            uses_input
        });
        let inputs_used = inputs_used
            || self.ntl.outputs.iter().any(|out| {
                if let Some(reg) = out.reg() {
                    input_set.contains(&reg)
                } else {
                    false
                }
            });
        if inputs_used {
            statements.push(ast::always(vec![ast::Events::Star], self.body));
        } else {
            statements.push(ast::initial(self.body));
        }
        declarations.extend(self.decls);
        Ok(Module {
            name: self.name,
            ports,
            declarations,
            statements,
            submodules,
            ..Default::default()
        })
    }
}

pub(crate) fn generate_hdl(module_name: &str, ntl: &Object) -> Result<Module, RHDLError> {
    NetListHDLBuilder::new(module_name, ntl).build()
}
