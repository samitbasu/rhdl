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
                                vlog::unsigned(0..=1),
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
                        vlog::stmt_item(
                            vlog::function_call_stmt(
                                stringify!(_dollar_my_function),
                                {
                                    let mut ret = Vec::with_capacity(2usize);
                                    ret.push(vlog::ident_expr(stringify!(a)));
                                    ret.push(vlog::ident_expr(stringify!(b)));
                                    ret
                                },
                            ),
                        ),
                    );
                    ret.push(
                        vlog::stmt_item(
                            vlog::function_call_stmt(stringify!(_dollar_finish), vec![]),
                        ),
                    );
                    ret
                },
            ),
        );
        ret
    });
}
