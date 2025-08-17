mod rhdl {
    pub mod vlog {
        include!("../src/ast.rs");
    }
}
fn main() {
    let _ = {
        let module0 = {
            let arg0 = rhdl::vlog::Port {
                direction: rhdl::vlog::Direction::Input,
                decl: rhdl::vlog::Declaration {
                    kind: rhdl::vlog::HDLKind::Wire,
                    signed_width: rhdl::vlog::SignedWidth::Unsigned(0..=1),
                    name: stringify!(a).into(),
                },
            };
            let arg1 = rhdl::vlog::Port {
                direction: rhdl::vlog::Direction::Output,
                decl: rhdl::vlog::Declaration {
                    kind: rhdl::vlog::HDLKind::Reg,
                    signed_width: rhdl::vlog::SignedWidth::Unsigned(0..=1),
                    name: stringify!(b).into(),
                },
            };
            let args_vec = vec![arg0, arg1,];
            let item0 = rhdl::vlog::Item::Statement(
                rhdl::vlog::Stmt::Case(rhdl::vlog::Case {
                    discriminant: Box::new(
                        rhdl::vlog::Expr::Ident(stringify!(a).into()),
                    ),
                    lines: vec![
                        rhdl::vlog::CaseLine { item :
                        rhdl::vlog::CaseItem::Literal(rhdl::vlog::LitVerilog { width : 2,
                        value : stringify!(b00) .into(), }), stmt :
                        Box::new(rhdl::vlog::Stmt::Assign(rhdl::vlog::Assign { target :
                        stringify!(b) .into(), rhs :
                        Box::new(rhdl::vlog::Expr::Literal(1)), })), },
                        rhdl::vlog::CaseLine { item :
                        rhdl::vlog::CaseItem::Literal(rhdl::vlog::LitVerilog { width : 2,
                        value : stringify!(b01) .into(), }), stmt :
                        Box::new(rhdl::vlog::Stmt::Assign(rhdl::vlog::Assign { target :
                        stringify!(b) .into(), rhs :
                        Box::new(rhdl::vlog::Expr::Literal(2)), })), },
                        rhdl::vlog::CaseLine { item :
                        rhdl::vlog::CaseItem::Literal(rhdl::vlog::LitVerilog { width : 2,
                        value : stringify!(b10) .into(), }), stmt :
                        Box::new(rhdl::vlog::Stmt::Assign(rhdl::vlog::Assign { target :
                        stringify!(b) .into(), rhs :
                        Box::new(rhdl::vlog::Expr::Literal(3)), })), },
                        rhdl::vlog::CaseLine { item :
                        rhdl::vlog::CaseItem::Literal(rhdl::vlog::LitVerilog { width : 2,
                        value : stringify!(b11) .into(), }), stmt :
                        Box::new(rhdl::vlog::Stmt::Assign(rhdl::vlog::Assign { target :
                        stringify!(b) .into(), rhs :
                        Box::new(rhdl::vlog::Expr::Literal(4)), })), }
                    ],
                }),
            );
            let items_vec = vec![item0,];
            rhdl::vlog::ModuleDef {
                name: stringify!(foo).into(),
                args: args_vec,
                items: items_vec,
            }
        };
        rhdl::vlog::ModuleList(vec![module0,])
    };
}
