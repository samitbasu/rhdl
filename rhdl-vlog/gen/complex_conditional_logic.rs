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
                stringify!(complex_logic),
                {
                    let mut ret = Vec::with_capacity(3usize);
                    ret.push(
                        vlog::port(
                            vlog::input(),
                            vlog::declaration(
                                vlog::wire(),
                                vlog::unsigned(0..=2),
                                stringify!(sel),
                            ),
                        ),
                    );
                    ret.push(
                        vlog::port(
                            vlog::input(),
                            vlog::declaration(
                                vlog::wire(),
                                vlog::unsigned(0..=7),
                                stringify!(data),
                            ),
                        ),
                    );
                    ret.push(
                        vlog::port(
                            vlog::output(),
                            vlog::declaration(
                                vlog::reg(),
                                vlog::unsigned(0..=7),
                                stringify!(result),
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
                                        vlog::case_stmt(
                                            vlog::ident_expr(stringify!(sel)),
                                            {
                                                let mut ret = Vec::with_capacity(5usize);
                                                ret.push(
                                                    vlog::case_line(
                                                        vlog::case_item_literal(
                                                            vlog::lit_verilog(3, stringify!(b000).into()),
                                                        ),
                                                        vlog::assign_stmt(
                                                            vlog::assign_target_ident(stringify!(result)),
                                                            vlog::binary_expr(
                                                                vlog::ident_expr(stringify!(data)),
                                                                vlog::binary_xor(),
                                                                vlog::constant_expr(vlog::lit_verilog(8, stringify!(hFF))),
                                                            ),
                                                        ),
                                                    ),
                                                );
                                                ret.push(
                                                    vlog::case_line(
                                                        vlog::case_item_literal(
                                                            vlog::lit_verilog(3, stringify!(b001).into()),
                                                        ),
                                                        vlog::assign_stmt(
                                                            vlog::assign_target_ident(stringify!(result)),
                                                            vlog::binary_expr(
                                                                vlog::ident_expr(stringify!(data)),
                                                                vlog::binary_shl(),
                                                                vlog::literal_expr(2),
                                                            ),
                                                        ),
                                                    ),
                                                );
                                                ret.push(
                                                    vlog::case_line(
                                                        vlog::case_item_literal(
                                                            vlog::lit_verilog(3, stringify!(b010).into()),
                                                        ),
                                                        vlog::assign_stmt(
                                                            vlog::assign_target_ident(stringify!(result)),
                                                            vlog::concat_expr({
                                                                let mut ret = Vec::with_capacity(2usize);
                                                                ret.push(
                                                                    vlog::index_expr(
                                                                        stringify!(data),
                                                                        vlog::literal_expr(3),
                                                                        Some(vlog::literal_expr(0)),
                                                                    ),
                                                                );
                                                                ret.push(
                                                                    vlog::index_expr(
                                                                        stringify!(data),
                                                                        vlog::literal_expr(7),
                                                                        Some(vlog::literal_expr(4)),
                                                                    ),
                                                                );
                                                                ret
                                                            }),
                                                        ),
                                                    ),
                                                );
                                                ret.push(
                                                    vlog::case_line(
                                                        vlog::case_item_literal(
                                                            vlog::lit_verilog(3, stringify!(b011).into()),
                                                        ),
                                                        vlog::assign_stmt(
                                                            vlog::assign_target_ident(stringify!(result)),
                                                            vlog::ternary_expr(
                                                                vlog::paren_expr(
                                                                    vlog::binary_expr(
                                                                        vlog::ident_expr(stringify!(data)),
                                                                        vlog::binary_gt(),
                                                                        vlog::constant_expr(vlog::lit_verilog(8, stringify!(h80))),
                                                                    ),
                                                                ),
                                                                vlog::binary_expr(
                                                                    vlog::ident_expr(stringify!(data)),
                                                                    vlog::binary_minus(),
                                                                    vlog::constant_expr(vlog::lit_verilog(8, stringify!(h80))),
                                                                ),
                                                                vlog::binary_expr(
                                                                    vlog::ident_expr(stringify!(data)),
                                                                    vlog::binary_plus(),
                                                                    vlog::constant_expr(vlog::lit_verilog(8, stringify!(h80))),
                                                                ),
                                                            ),
                                                        ),
                                                    ),
                                                );
                                                ret.push(
                                                    vlog::case_line(
                                                        vlog::case_item_wild(),
                                                        vlog::block_stmt({
                                                            let mut ret = Vec::with_capacity(1usize);
                                                            ret.push(
                                                                vlog::if_stmt(
                                                                    vlog::index_expr(
                                                                        stringify!(data),
                                                                        vlog::literal_expr(7),
                                                                        None,
                                                                    ),
                                                                    vlog::block_stmt({
                                                                        let mut ret = Vec::with_capacity(1usize);
                                                                        ret.push(
                                                                            vlog::assign_stmt(
                                                                                vlog::assign_target_ident(stringify!(result)),
                                                                                vlog::unary_expr(
                                                                                    vlog::unary_not(),
                                                                                    vlog::ident_expr(stringify!(data)),
                                                                                ),
                                                                            ),
                                                                        );
                                                                        ret
                                                                    }),
                                                                    Some(
                                                                        vlog::block_stmt({
                                                                            let mut ret = Vec::with_capacity(1usize);
                                                                            ret.push(
                                                                                vlog::assign_stmt(
                                                                                    vlog::assign_target_ident(stringify!(result)),
                                                                                    vlog::binary_expr(
                                                                                        vlog::ident_expr(stringify!(data)),
                                                                                        vlog::binary_or(),
                                                                                        vlog::constant_expr(vlog::lit_verilog(8, stringify!(h0F))),
                                                                                    ),
                                                                                ),
                                                                            );
                                                                            ret
                                                                        }),
                                                                    ),
                                                                ),
                                                            );
                                                            ret
                                                        }),
                                                    ),
                                                );
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
