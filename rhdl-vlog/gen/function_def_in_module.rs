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
                    vlog::declaration(vlog::wire(), vlog::unsigned(0..=2), stringify!(a)),
                );
                let elem1 = vlog::port(
                    vlog::output(),
                    vlog::declaration(vlog::reg(), vlog::unsigned(0..=1), stringify!(b)),
                );
                vec![elem0, elem1]
            },
            {
                let elem0 = vlog::function_def_item(
                    vlog::function_def(
                        vlog::unsigned(0..=1),
                        stringify!(my_function),
                        {
                            let elem0 = vlog::port(
                                vlog::input(),
                                vlog::declaration(
                                    vlog::wire(),
                                    vlog::unsigned(0..=1),
                                    stringify!(x),
                                ),
                            );
                            vec![elem0]
                        },
                        {
                            let elem0 = vlog::stmt_item(
                                vlog::continuous_assign_stmt(
                                    stringify!(my_function),
                                    vlog::binary_expr(
                                        vlog::ident_expr(stringify!(x)),
                                        vlog::binary_plus(),
                                        vlog::literal_expr(1),
                                    ),
                                ),
                            );
                            vec![elem0]
                        },
                    ),
                );
                let elem1 = vlog::stmt_item(
                    vlog::assign_stmt(
                        stringify!(b),
                        vlog::function_expr(
                            stringify!(my_function),
                            {
                                let elem0 = vlog::ident_expr(stringify!(a));
                                vec![elem0]
                            },
                        ),
                    ),
                );
                vec![elem0, elem1]
            },
        );
        vec![elem0]
    });
}
