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
                direction: rhdl::vlog::Direction::Input,
                decl: rhdl::vlog::Declaration {
                    kind: rhdl::vlog::HDLKind::Wire,
                    signed_width: rhdl::vlog::SignedWidth::Unsigned(0..=1),
                    name: stringify!(c).into(),
                },
            };
            let arg2 = rhdl::vlog::Port {
                direction: rhdl::vlog::Direction::Output,
                decl: rhdl::vlog::Declaration {
                    kind: rhdl::vlog::HDLKind::Reg,
                    signed_width: rhdl::vlog::SignedWidth::Unsigned(0..=1),
                    name: stringify!(b).into(),
                },
            };
            let args_vec = vec![arg0, arg1, arg2];
            let item0 = rhdl::vlog::Item::Statement(rhdl::vlog::Stmt::ConcatAssign(
                rhdl::vlog::ConcatAssign {
                    target: {
                        let elem0 = rhdl::vlog::Expr::Ident(stringify!(a).into());
                        let elem1 = rhdl::vlog::Expr::Ident(stringify!(c).into());
                        vec![elem0, elem1]
                    },
                    rhs: Box::new(rhdl::vlog::Expr::Concat({
                        let elem0 = rhdl::vlog::Expr::Constant(rhdl::vlog::LitVerilog {
                            width: 1,
                            value: stringify!(b0).into(),
                        });
                        let elem1 = rhdl::vlog::Expr::Ident(stringify!(a).into());
                        vec![elem0, elem1]
                    })),
                },
            ));
            let items_vec = vec![item0];
            rhdl::vlog::ModuleDef {
                name: stringify!(foo).into(),
                args: args_vec,
                items: items_vec,
            }
        };
        rhdl::vlog::ModuleList(vec![module0])
    };
}
