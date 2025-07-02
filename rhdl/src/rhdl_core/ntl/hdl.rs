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
use crate::rhdl_core::ntl::object::BlackBoxMode;
use crate::rhdl_core::ntl::spec;
use crate::rhdl_core::ntl::spec::BlackBox;
use crate::rhdl_core::ntl::spec::CaseEntry;
use crate::rhdl_core::ntl::spec::OpCode;
use crate::rhdl_core::ntl::spec::Operand;
use crate::rhdl_core::ntl::spec::VectorOp;
use crate::rhdl_core::ntl::visit::visit_operands;
use crate::rhdl_core::ntl::visit::Sense;
use crate::rhdl_core::ntl::Object;
use crate::rhdl_core::rtl::spec::AluBinary;
use crate::rhdl_core::rtl::spec::AluUnary;

struct NetListHDLBuilder<'a> {
    ntl: &'a Object,
    instances: Vec<ast::Statement>,
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
            instances: vec![],
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
    fn push_body(&mut self, statement: ast::Statement) {
        self.body.push(statement);
    }
    fn select_op(
        &mut self,
        op: &spec::Select,
        location: Option<SourceLocation>,
    ) -> Result<(), RHDLError> {
        let target = self.reg(op.lhs, location)?;
        self.push_body(ast::assign(
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
        self.push_body(ast::assign(&target, opex(op.rhs)));
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
        self.push_body(ast::assign(&target, expr));
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
        self.push_body(ast::assign(&target, ast::binary(alu, arg1, arg2)));
        Ok(())
    }
    fn not_op(
        &mut self,
        op: &spec::Not,
        location: Option<SourceLocation>,
    ) -> Result<(), RHDLError> {
        let target = self.reg(op.lhs, location)?;
        self.push_body(ast::assign(
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
        self.push_body(ast::case(discriminant, table));
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
        self.push_body(ast::assign(&target, ast::unary(alu, arg)));
        Ok(())
    }
    fn black_box_op(&mut self, black_box: &BlackBox) -> Result<(), RHDLError> {
        let bb_core = &self.ntl.black_boxes[black_box.code.raw()];
        let out = opex_v(&black_box.lhs);
        let mut connections = vec![hdl::ast::connection("o", out)];
        match bb_core.mode {
            BlackBoxMode::Asynchronous => {
                let i = opex_v(&black_box.arg[0]);
                connections.push(hdl::ast::connection("i", i));
            }
            BlackBoxMode::Synchronous => {
                let cr = opex_v(&black_box.arg[0]);
                let i = opex_v(&black_box.arg[1]);
                connections.push(hdl::ast::connection("clock_reset", cr));
                connections.push(hdl::ast::connection("i", i));
            }
        }
        let core_id = self.temporary_counter;
        self.temporary_counter += 1;
        self.instances.push(hdl::ast::component_instance(
            &bb_core.code.name,
            &format!("bb_{core_id}"),
            connections,
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
                self.push_body(ast::comment(comment));
                Ok(())
            }
            spec::OpCode::Select(select) => self.select_op(select, location),
            spec::OpCode::Not(not) => self.not_op(not, location),
            spec::OpCode::BlackBox(black_box) => self.black_box_op(black_box),
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
        // Outputs of black boxes must be wires, not registers
        let mut wires = BTreeSet::default();
        for lop in &self.ntl.ops {
            if let OpCode::BlackBox(black_box) = &lop.op {
                wires.extend(black_box.lhs.iter().flat_map(Operand::reg))
            }
        }
        let mut declarations = registers
            .difference(&wires)
            .map(|ndx| {
                ast::declaration(
                    ast::HDLKind::Reg,
                    &format!("r{}", ndx.raw()),
                    ast::unsigned_width(1),
                    None,
                )
            })
            .chain(wires.iter().map(|ndx| {
                ast::declaration(
                    ast::HDLKind::Wire,
                    &format!("r{}", ndx.raw()),
                    ast::unsigned_width(1),
                    None,
                )
            }))
            .collect::<Vec<_>>();
        // Connect the input registers to their module input names
        for (ndx, arg) in self.ntl.inputs.iter().enumerate() {
            for (bit, &reg) in arg.iter().enumerate() {
                self.push_body(ast::assign(
                    &self.reg(Operand::Register(reg), None)?,
                    ast::index_bit(&format!("arg_{ndx}"), bit),
                ))
            }
        }
        let submodules = self
            .ntl
            .black_boxes
            .iter()
            .map(|bb| bb.code.as_module())
            .collect();
        for lop in &self.ntl.ops {
            self.op_code(&lop.op, None)?;
        }
        let output_bits =
            ast::concatenate(self.ntl.outputs.iter().rev().copied().map(opex).collect());
        self.push_body(ast::assign("out", output_bits));
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
        let mut statements = std::mem::take(&mut self.instances);
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
