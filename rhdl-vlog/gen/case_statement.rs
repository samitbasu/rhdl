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
                    let mut ret = Vec::with_capacity(1usize);
                    ret.push(
                        vlog::stmt_item(
                            vlog::case_stmt(
                                vlog::ident_expr(stringify!(a)),
                                {
                                    let mut ret = Vec::with_capacity(4usize);
                                    ret.push(
                                        vlog::case_line(
                                            vlog::case_item_literal(
                                                vlog::lit_verilog(2, stringify!(b00).into()),
                                            ),
                                            vlog::assign_stmt(
                                                vlog::assign_target_ident(stringify!(b)),
                                                vlog::literal_expr(1),
                                            ),
                                        ),
                                    );
                                    ret.push(
                                        vlog::case_line(
                                            vlog::case_item_literal(
                                                vlog::lit_verilog(2, stringify!(b01).into()),
                                            ),
                                            vlog::assign_stmt(
                                                vlog::assign_target_ident(stringify!(b)),
                                                vlog::literal_expr(2),
                                            ),
                                        ),
                                    );
                                    ret.push(
                                        vlog::case_line(
                                            vlog::case_item_literal(
                                                vlog::lit_verilog(2, stringify!(b10).into()),
                                            ),
                                            vlog::assign_stmt(
                                                vlog::assign_target_ident(stringify!(b)),
                                                vlog::literal_expr(3),
                                            ),
                                        ),
                                    );
                                    ret.push(
                                        vlog::case_line(
                                            vlog::case_item_wild(),
                                            vlog::assign_stmt(
                                                vlog::assign_target_ident(stringify!(b)),
                                                vlog::literal_expr(4),
                                            ),
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
