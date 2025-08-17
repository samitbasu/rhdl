pub mod vlog {
    include!("../src/ast.rs");
}
fn main() {
    let _ = vlog::module_list({
        let mut ret = Vec::with_capacity(1usize);
        ret.push(
            vlog::module_def(
                stringify!(foo),
                {
                    let mut ret = Vec::with_capacity(2usize);
                    ret.push(
                        vlog::port(
                            vlog::input(),
                            vlog::declaration(
                                vlog::wire(),
                                vlog::unsigned(0..=2),
                                stringify!(a),
                            ),
                        ),
                    );
                    ret.push(
                        vlog::port(
                            vlog::output(),
                            vlog::declaration(
                                vlog::reg(),
                                vlog::unsigned(0..=1),
                                stringify!(b),
                            ),
                        ),
                    );
                    ret
                },
                {
                    let mut ret = Vec::with_capacity(2usize);
                    ret.push(
                        vlog::function_def_item(
                            vlog::function_def(
                                vlog::unsigned(0..=1),
                                stringify!(my_function),
                                {
                                    let mut ret = Vec::with_capacity(1usize);
                                    ret.push(
                                        vlog::port(
                                            vlog::input(),
                                            vlog::declaration(
                                                vlog::wire(),
                                                vlog::unsigned(0..=1),
                                                stringify!(x),
                                            ),
                                        ),
                                    );
                                    ret
                                },
                                {
                                    let mut ret = Vec::with_capacity(1usize);
                                    ret.push(
                                        vlog::stmt_item(
                                            vlog::continuous_assign_stmt(
                                                stringify!(my_function),
                                                vlog::binary_expr(
                                                    vlog::ident_expr(stringify!(x)),
                                                    vlog::binary_plus(),
                                                    vlog::literal_expr(1),
                                                ),
                                            ),
                                        ),
                                    );
                                    ret
                                },
                            ),
                        ),
                    );
                    ret.push(
                        vlog::stmt_item(
                            vlog::assign_stmt(
                                stringify!(b),
                                vlog::function_expr(
                                    stringify!(my_function),
                                    {
                                        let mut ret = Vec::with_capacity(1usize);
                                        ret.push(vlog::ident_expr(stringify!(a)));
                                        ret
                                    },
                                ),
                            ),
                        ),
                    );
                    ret
                },
            ),
        );
        ret
    });
}
