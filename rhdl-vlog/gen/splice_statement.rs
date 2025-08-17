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
                        vlog::stmt_item(
                            vlog::splice_stmt(
                                stringify!(a),
                                vlog::literal_expr(1),
                                None,
                                vlog::ident_expr(stringify!(b)),
                            ),
                        ),
                    );
                    ret.push(
                        vlog::stmt_item(
                            vlog::splice_stmt(
                                stringify!(a),
                                vlog::literal_expr(1),
                                Some(vlog::literal_expr(0)),
                                vlog::concat_expr({
                                    let mut ret = Vec::with_capacity(2usize);
                                    ret.push(vlog::ident_expr(stringify!(b)));
                                    ret.push(vlog::ident_expr(stringify!(b)));
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
