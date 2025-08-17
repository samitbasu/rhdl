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
                rhdl::vlog::Stmt::LocalParam(rhdl::vlog::LocalParam {
                    target: stringify!(my_param).into(),
                    rhs: rhdl::vlog::LitVerilog {
                        width: 5,
                        value: stringify!(b1_1001).into(),
                    },
                }),
            );
            let item1 = rhdl::vlog::Item::Statement(
                rhdl::vlog::Stmt::Always(rhdl::vlog::Always {
                    sensitivity: vec![
                        rhdl::vlog::Sensitivity::PosEdge(stringify!(a) .into())
                    ],
                    body: Box::new(
                        rhdl::vlog::Stmt::Block({
                            let stmt0 = rhdl::vlog::Stmt::NonblockAssign(rhdl::vlog::Assign {
                                target: stringify!(b).into(),
                                rhs: Box::new(
                                    rhdl::vlog::Expr::Ident(stringify!(my_param).into()),
                                ),
                            });
                            vec![stmt0,]
                        }),
                    ),
                }),
            );
            let items_vec = vec![item0, item1,];
            rhdl::vlog::ModuleDef {
                name: stringify!(foo).into(),
                args: args_vec,
                items: items_vec,
            }
        };
        rhdl::vlog::ModuleList(vec![module0,])
    };
}
