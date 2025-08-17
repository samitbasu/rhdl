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
                    let mut ret = Vec::with_capacity(1usize);
                    ret.push(
                        vlog::stmt_item(
                            vlog::always_stmt(
                                {
                                    let mut ret = Vec::with_capacity(1usize);
                                    ret.push(vlog::star());
                                    ret
                                },
                                vlog::block_stmt({
                                    let mut ret = Vec::with_capacity(1usize);
                                    ret.push(
                                        vlog::function_call_stmt(
                                            stringify!(_dollar_display),
                                            {
                                                let mut ret = Vec::with_capacity(1usize);
                                                ret.push(vlog::string_expr("Hello, World!"));
                                                ret
                                            },
                                        ),
                                    );
                                    ret
                                }),
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
