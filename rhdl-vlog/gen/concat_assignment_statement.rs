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
                    let mut ret = Vec::with_capacity(3usize);
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
                            vlog::input(),
                            vlog::declaration(
                                vlog::wire(),
                                vlog::unsigned(0..=1),
                                stringify!(c),
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
                            vlog::concat_assign_stmt(
                                {
                                    let mut ret = Vec::with_capacity(2usize);
                                    ret.push(vlog::ident_expr(stringify!(a)));
                                    ret.push(vlog::ident_expr(stringify!(c)));
                                    ret
                                },
                                vlog::concat_expr({
                                    let mut ret = Vec::with_capacity(2usize);
                                    ret.push(
                                        vlog::constant_expr(vlog::lit_verilog(1, stringify!(b0))),
                                    );
                                    ret.push(vlog::ident_expr(stringify!(a)));
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
