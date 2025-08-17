pub mod vlog {
    include!("../src/ast.rs");
}
fn main() {
    let _ = vlog::module_list({
        let elem0 = vlog::module_def(
            stringify!(foo),
            {
                let elem0 = vlog::port(
                    vlog::input(),
                    vlog::declaration(vlog::wire(), vlog::unsigned(0..=1), stringify!(a)),
                );
                let elem1 = vlog::port(
                    vlog::output(),
                    vlog::declaration(vlog::reg(), vlog::unsigned(0..=1), stringify!(b)),
                );
                vec![elem0, elem1]
            },
            {
                let elem0 = vlog::stmt_item(
                    vlog::instance_stmt(
                        stringify!(bar_0),
                        stringify!(bar),
                        {
                            let elem0 = vlog::connection(
                                stringify!(c),
                                vlog::paren_expr(vlog::ident_expr(stringify!(a))),
                            );
                            let elem1 = vlog::connection(
                                stringify!(d),
                                vlog::paren_expr(vlog::ident_expr(stringify!(b))),
                            );
                            vec![elem0, elem1]
                        },
                    ),
                );
                vec![elem0]
            },
        );
        vec![elem0]
    });
}
