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
                    vlog::declaration(vlog::wire(), vlog::unsigned(0..=1), stringify!(b)),
                );
                vec![elem0, elem1]
            },
            {
                let elem0 = vlog::stmt_item(
                    vlog::if_stmt(
                        vlog::ident_expr(stringify!(a)),
                        vlog::block_stmt({
                            let elem0 = vlog::assign_stmt(
                                stringify!(b),
                                vlog::literal_expr(1),
                            );
                            vec![elem0]
                        }),
                        Some(
                            vlog::block_stmt({
                                let elem0 = vlog::assign_stmt(
                                    stringify!(b),
                                    vlog::literal_expr(0),
                                );
                                vec![elem0]
                            }),
                        ),
                    ),
                );
                vec![elem0]
            },
        );
        vec![elem0]
    });
}
