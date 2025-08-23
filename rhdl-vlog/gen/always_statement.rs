pub mod vlog {
    include!("../src/ast.rs");
}
pub mod formatter {
    include!("../src/formatter.rs");
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
                    let mut ret = Vec::with_capacity(1usize);
                    ret.push(
                        vlog::stmt_item(
                            vlog::always_stmt(
                                {
                                    let mut ret = Vec::with_capacity(2usize);
                                    ret.push(vlog::pos_edge(stringify!(a)));
                                    ret.push(vlog::signal(stringify!(b)));
                                    ret
                                },
                                vlog::block_stmt({
                                    let mut ret = Vec::with_capacity(1usize);
                                    ret.push(
                                        vlog::nonblock_assign_stmt(
                                            vlog::assign_target_ident(stringify!(b)),
                                            vlog::literal_expr(1),
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
