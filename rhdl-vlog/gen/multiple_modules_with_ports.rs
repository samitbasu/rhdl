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
                    kind: rhdl::vlog::HDLKind::Wire,
                    signed_width: rhdl::vlog::SignedWidth::Unsigned(0..=1),
                    name: stringify!(b).into(),
                },
            };
            let args_vec = vec![arg0, arg1,];
            let items_vec = vec![];
            rhdl::vlog::ModuleDef {
                name: stringify!(foo).into(),
                args: args_vec,
                items: items_vec,
            }
        };
        let module1 = {
            let arg0 = rhdl::vlog::Port {
                direction: rhdl::vlog::Direction::Input,
                decl: rhdl::vlog::Declaration {
                    kind: rhdl::vlog::HDLKind::Wire,
                    signed_width: rhdl::vlog::SignedWidth::Unsigned(0..=1),
                    name: stringify!(c).into(),
                },
            };
            let arg1 = rhdl::vlog::Port {
                direction: rhdl::vlog::Direction::Output,
                decl: rhdl::vlog::Declaration {
                    kind: rhdl::vlog::HDLKind::Wire,
                    signed_width: rhdl::vlog::SignedWidth::Unsigned(0..=1),
                    name: stringify!(d).into(),
                },
            };
            let args_vec = vec![arg0, arg1,];
            let items_vec = vec![];
            rhdl::vlog::ModuleDef {
                name: stringify!(bar).into(),
                args: args_vec,
                items: items_vec,
            }
        };
        rhdl::vlog::ModuleList(vec![module0, module1,])
    };
}
