use std::collections::BTreeSet;
use std::collections::HashSet;

use crate::BitX;
use crate::HDLDescriptor;
use crate::RHDLError;
use crate::ast::SourceLocation;
use crate::error::rhdl_error;
use crate::ntl::Object;
use crate::ntl::error::NetListError;
use crate::ntl::error::NetListICE;
use crate::ntl::object::BlackBoxMode;
use crate::ntl::spec;
use crate::ntl::spec::BlackBox;
use crate::ntl::spec::CaseEntry;
use crate::ntl::spec::OpCode;
use crate::ntl::spec::VectorOp;
use crate::ntl::spec::Wire;
use crate::ntl::visit::visit_wires;

use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use rhdl_vlog as vlog;
use syn::parse_quote;

struct NetListHDLBuilder<'a> {
    ntl: &'a Object,
    body: Vec<vlog::Stmt>,
    instances: Vec<vlog::stmt::Instance>,
    name: String,
    temporary_counter: usize,
}

impl From<BitX> for vlog::LitVerilog {
    fn from(bit: BitX) -> Self {
        match bit {
            BitX::Zero => vlog::lit_verilog(1, "b0"),
            BitX::One => vlog::lit_verilog(1, "b1"),
            BitX::X => vlog::lit_verilog(1, "bX"),
        }
    }
}

impl<'a> NetListHDLBuilder<'a> {
    fn new(name: &'_ str, ntl: &'a Object) -> Self {
        Self {
            ntl,
            body: vec![],
            instances: vec![],
            name: name.into(),
            temporary_counter: 0,
        }
    }
    fn opex(&self, operand: Wire) -> vlog::Expr {
        match operand {
            Wire::Literal(lid) => {
                let val = self.ntl.symtab[lid];
                vlog::Expr::Constant(val.into())
            }
            Wire::Register(rid) => vlog::Expr::Ident(rid.to_string()),
        }
    }
    fn opex_v(&self, operands: &[Wire]) -> vlog::Expr {
        vlog::Expr::Concat(vlog::ExprConcat {
            elements: operands
                .iter()
                .rev()
                .copied()
                .map(|x| self.opex(x))
                .collect(),
        })
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
    fn reg(
        &self,
        operand: Wire,
        location: Option<SourceLocation>,
    ) -> Result<syn::Ident, RHDLError> {
        if let Some(rid) = operand.reg() {
            Ok(syn::Ident::new(
                &rid.to_string(),
                proc_macro2::Span::call_site(),
            ))
        } else {
            Err(self.raise_ice(NetListICE::ExpectedRegisterNotConstant, location))
        }
    }
    fn reg_v(
        &self,
        operands: &[Wire],
        location: Option<SourceLocation>,
    ) -> Result<TokenStream, RHDLError> {
        let args = operands
            .iter()
            .rev() // <--- Super important!  Concat operator is MSB -> LSB
            .map(|&op| self.reg(op, location))
            .collect::<Result<Vec<syn::Ident>, RHDLError>>()?;
        Ok(quote! { { #(#args),* } })
    }
    fn add_stmt(&mut self, stmt: vlog::Stmt) {
        self.body.push(stmt);
    }
    fn select_op(
        &mut self,
        op: &spec::Select,
        location: Option<SourceLocation>,
    ) -> Result<(), RHDLError> {
        let target = self.reg(op.lhs, location)?;
        let selector = self.opex(op.selector);
        let true_case = self.opex(op.true_case);
        let false_case = self.opex(op.false_case);
        self.add_stmt(parse_quote!(
            #target = #selector ? #true_case : #false_case
        ));
        Ok(())
    }
    fn assign_op(
        &mut self,
        op: &spec::Assign,
        location: Option<SourceLocation>,
    ) -> Result<(), RHDLError> {
        let target = self.reg(op.lhs, location)?;
        let rhs = self.opex(op.rhs);
        self.add_stmt(parse_quote!(
            #target = #rhs
        ));
        Ok(())
    }
    fn binary_op(
        &mut self,
        op: &spec::Binary,
        location: Option<SourceLocation>,
    ) -> Result<(), RHDLError> {
        let target = self.reg(op.lhs, location)?;
        let alu = match op.op {
            spec::BinaryOp::Xor => vlog::BinaryOp::Xor,
            spec::BinaryOp::And => vlog::BinaryOp::And,
            spec::BinaryOp::Or => vlog::BinaryOp::Or,
        };
        let arg1 = self.opex(op.arg1);
        let arg2 = self.opex(op.arg2);
        self.add_stmt(parse_quote!(
            #target = #arg1 #alu #arg2
        ));
        Ok(())
    }
    fn vector_op(
        &mut self,
        op: &spec::Vector,
        location: Option<SourceLocation>,
    ) -> Result<(), RHDLError> {
        let target = self.reg_v(&op.lhs, location)?;
        let alu = match op.op {
            VectorOp::Add => vlog::BinaryOp::Plus,
            VectorOp::Sub => vlog::BinaryOp::Minus,
            VectorOp::Mul => vlog::BinaryOp::Mul,
            VectorOp::Eq => vlog::BinaryOp::Eq,
            VectorOp::Ne => vlog::BinaryOp::Ne,
            VectorOp::Lt => vlog::BinaryOp::Lt,
            VectorOp::Le => vlog::BinaryOp::Le,
            VectorOp::Gt => vlog::BinaryOp::Gt,
            VectorOp::Ge => vlog::BinaryOp::Ge,
            VectorOp::Shl => vlog::BinaryOp::Shl,
            VectorOp::Shr => vlog::BinaryOp::SignedRightShift,
        };
        let arg1 = self.opex_v(&op.arg1);
        let arg2 = self.opex_v(&op.arg2);
        let arg1 = if op.signed {
            parse_quote! { $signed(#arg1) }
        } else {
            arg1
        };
        let arg2 = if op.signed {
            parse_quote! { $signed(#arg2) }
        } else {
            arg2
        };
        self.add_stmt(parse_quote! { #target = #arg1 #alu #arg2 });
        Ok(())
    }
    fn not_op(
        &mut self,
        op: &spec::Not,
        location: Option<SourceLocation>,
    ) -> Result<(), RHDLError> {
        let target = self.reg(op.lhs, location)?;
        let arg = self.opex(op.arg);
        self.add_stmt(parse_quote! {
            #target = ~(#arg)
        });
        Ok(())
    }
    fn case_op(
        &mut self,
        op: &spec::Case,
        location: Option<SourceLocation>,
    ) -> Result<(), RHDLError> {
        let discriminant = self.opex_v(&op.discriminant);
        let lhs = self.reg(op.lhs, location)?;
        let table = op.entries.iter().map(|(entry, operand)| {
            let value = self.opex(*operand);
            match entry {
                CaseEntry::Literal(lit) => {
                    let lit: vlog::LitVerilog = lit.into();
                    quote! { #lit : #lhs = #value }
                }
                CaseEntry::WildCard => quote! { default : #lhs = #value },
            }
        });
        self.add_stmt(parse_quote! {
            case (#discriminant)
                #(#table;)*
            endcase
        });
        Ok(())
    }
    fn unary_op(
        &mut self,
        op: &spec::Unary,
        location: Option<SourceLocation>,
    ) -> Result<(), RHDLError> {
        let target = self.reg_v(&op.lhs, location)?;
        let arg = self.opex_v(&op.arg);
        let alu = match op.op {
            spec::UnaryOp::All => vlog::UnaryOp::And,
            spec::UnaryOp::Any => vlog::UnaryOp::Or,
            spec::UnaryOp::Neg => vlog::UnaryOp::Minus,
            spec::UnaryOp::Xor => vlog::UnaryOp::Xor,
        };
        self.add_stmt(parse_quote!(#target = #alu #arg));
        Ok(())
    }
    fn black_box_op(&mut self, black_box: &BlackBox) -> Result<(), RHDLError> {
        let bb_core = &self.ntl.black_boxes[black_box.code.raw()];
        let out = self.opex_v(&black_box.lhs);
        let mut connections: Vec<vlog::stmt::Connection> = vec![parse_quote! { .o(#out) }];
        match bb_core.mode {
            BlackBoxMode::Asynchronous => {
                let i = self.opex_v(&black_box.arg[0]);
                connections.push(parse_quote! { .i(#i) });
            }
            BlackBoxMode::Synchronous => {
                let cr = self.opex_v(&black_box.arg[0]);
                let i = self.opex_v(&black_box.arg[1]);
                connections.push(parse_quote! { .clock_reset(#cr) });
                connections.push(parse_quote! { .i(#i) });
            }
        }
        let core_id = self.temporary_counter;
        self.temporary_counter += 1;
        let core_name = format_ident!("{}", bb_core.code.name);
        let instance_name = format_ident!("bb_{}", core_id);
        self.instances.push(parse_quote! {
            #core_name #instance_name (
                #(#connections),*
            )
        });
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
            spec::OpCode::Select(select) => self.select_op(select, location),
            spec::OpCode::Not(not) => self.not_op(not, location),
            spec::OpCode::BlackBox(black_box) => self.black_box_op(black_box),
            spec::OpCode::Unary(unary) => self.unary_op(unary, location),
        }
    }
    fn build(mut self) -> Result<vlog::ModuleList, RHDLError> {
        // Declare the input ports
        let ports = self
            .ntl
            .inputs
            .iter()
            .enumerate()
            .flat_map(|(ndx, x)| {
                (!x.is_empty()).then(|| {
                    let name = format_ident!("arg_{ndx}");
                    let width = vlog::unsigned_width(x.len());
                    parse_quote! { input wire #width #name }
                })
            })
            .chain(std::iter::once({
                let name = format_ident!("out");
                let width = vlog::unsigned_width(self.ntl.outputs.len());
                parse_quote! { output reg #width #name }
            }))
            .collect::<Vec<vlog::Port>>();
        let mut registers = BTreeSet::default();
        for lop in &self.ntl.ops {
            visit_wires(&lop.op, |_sense, op| {
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
                wires.extend(black_box.lhs.iter().copied().flat_map(Wire::reg))
            }
        }
        let declarations = registers
            .difference(&wires)
            .map(|ndx| {
                let name = format_ident!("{}", ndx.to_string());
                parse_quote! {reg #name}
            })
            .chain(wires.iter().map(|ndx| {
                let name = format_ident!("{}", ndx.to_string());
                parse_quote! {wire #name}
            }))
            .collect::<Vec<vlog::Declaration>>();
        // Connect the input registers to their module input names
        for (ndx, arg) in self.ntl.inputs.iter().enumerate() {
            for (bit, &reg) in arg.iter().enumerate() {
                let target = self.reg(Wire::Register(reg), None)?;
                let bit = syn::Index::from(bit);
                let arg = format_ident!("arg_{ndx}");
                self.add_stmt(parse_quote! {#target = #arg[#bit]})
            }
        }
        let submodules = self
            .ntl
            .black_boxes
            .iter()
            .flat_map(|bb| bb.code.modules.modules.iter().cloned())
            .collect::<Vec<vlog::ModuleDef>>();
        for lop in &self.ntl.ops {
            self.op_code(&lop.op, None)?;
        }
        let outputs = self.ntl.outputs.iter().rev().copied().map(|t| self.opex(t));
        let output_bits: vlog::Expr = parse_quote! { { #(#outputs),* } };
        let out = format_ident!("out");
        self.add_stmt(parse_quote! { #out = #output_bits });
        // Check if any of the inputs are used by the body of the module
        let input_set = self.ntl.inputs.iter().flatten().collect::<HashSet<_>>();
        let inputs_used = self.ntl.ops.iter().any(|lop| {
            let mut uses_input = false;
            visit_wires(&lop.op, |sense, op| {
                if let Some(reg_id) = op.reg()
                    && sense.is_read()
                    && input_set.contains(&reg_id)
                {
                    uses_input = true;
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
        let instances = &self.instances;
        let body = self.body;
        let body: vlog::Item = if inputs_used {
            parse_quote! {
                always @(*) begin
                    #(#body;)*
                end
            }
        } else {
            parse_quote! {
                initial begin
                    #(#body;)*
                end
            }
        };
        let name = format_ident!("{}", self.name);
        Ok(parse_quote! {
            module #name(#(#ports),*);
                #(#declarations;)*
                #(#instances;)*
                #body
            endmodule
            #(#submodules)*
        })
    }
}

pub(crate) fn build_hdl(module_name: &str, ntl: &Object) -> Result<HDLDescriptor, RHDLError> {
    let modules = NetListHDLBuilder::new(module_name, ntl).build()?;
    Ok(HDLDescriptor {
        name: module_name.to_string(),
        modules,
    })
}
