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
                            vlog::instance_stmt(
                                stringify!(bar_0),
                                stringify!(bar),
                                {
                                    let mut ret = Vec::with_capacity(2usize);
                                    ret.push(
                                        vlog::connection(
                                            stringify!(c),
                                            vlog::paren_expr(vlog::ident_expr(stringify!(a))),
                                        ),
                                    );
                                    ret.push(
                                        vlog::connection(
                                            stringify!(d),
                                            vlog::paren_expr(vlog::ident_expr(stringify!(b))),
                                        ),
                                    );
                                    ret
                                },
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
