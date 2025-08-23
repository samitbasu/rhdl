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
                stringify!(concat_test),
                {
                    let mut ret = Vec::with_capacity(4usize);
                    ret.push(
                        vlog::port(
                            vlog::input(),
                            vlog::declaration(
                                vlog::wire(),
                                vlog::unsigned(0..=3),
                                stringify!(a),
                            ),
                        ),
                    );
                    ret.push(
                        vlog::port(
                            vlog::input(),
                            vlog::declaration(
                                vlog::wire(),
                                vlog::unsigned(0..=3),
                                stringify!(b),
                            ),
                        ),
                    );
                    ret.push(
                        vlog::port(
                            vlog::input(),
                            vlog::declaration(
                                vlog::wire(),
                                vlog::unsigned(0..=1),
                                stringify!(sel),
                            ),
                        ),
                    );
                    ret.push(
                        vlog::port(
                            vlog::output(),
                            vlog::declaration(
                                vlog::reg(),
                                vlog::unsigned(0..=15),
                                stringify!(result),
                            ),
                        ),
                    );
                    ret
                },
                {
                    let mut ret = Vec::with_capacity(4usize);
                    ret.push(
                        vlog::declaration_item(
                            vlog::declaration_list(
                                vlog::wire(),
                                vlog::unsigned(0..=7),
                                {
                                    let mut ret = Vec::with_capacity(2usize);
                                    ret.push(vlog::decl_kind(stringify!(temp1), None));
                                    ret.push(vlog::decl_kind(stringify!(temp2), None));
                                    ret
                                },
                            ),
                        ),
                    );
                    ret.push(
                        vlog::stmt_item(
                            vlog::continuous_assign_stmt(
                                vlog::assign_target_ident(stringify!(temp1)),
                                vlog::replica_expr(
                                    2,
                                    {
                                        let mut ret = Vec::with_capacity(1usize);
                                        ret.push(vlog::ident_expr(stringify!(a)));
                                        ret
                                    },
                                ),
                            ),
                        ),
                    );
                    ret.push(
                        vlog::stmt_item(
                            vlog::continuous_assign_stmt(
                                vlog::assign_target_ident(stringify!(temp2)),
                                vlog::concat_expr({
                                    let mut ret = Vec::with_capacity(2usize);
                                    ret.push(vlog::ident_expr(stringify!(b)));
                                    ret.push(vlog::ident_expr(stringify!(a)));
                                    ret
                                }),
                            ),
                        ),
                    );
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
                                                let mut ret = Vec::with_capacity(4usize);
                                                ret.push(
                                                    vlog::case_line(
                                                        vlog::case_item_literal(
                                                            vlog::lit_verilog(2, stringify!(b00).into()),
                                                        ),
                                                        vlog::assign_stmt(
                                                            vlog::assign_target_ident(stringify!(result)),
                                                            vlog::replica_expr(
                                                                2,
                                                                {
                                                                    let mut ret = Vec::with_capacity(1usize);
                                                                    ret.push(vlog::ident_expr(stringify!(temp1)));
                                                                    ret
                                                                },
                                                            ),
                                                        ),
                                                    ),
                                                );
                                                ret.push(
                                                    vlog::case_line(
                                                        vlog::case_item_literal(
                                                            vlog::lit_verilog(2, stringify!(b01).into()),
                                                        ),
                                                        vlog::assign_stmt(
                                                            vlog::assign_target_ident(stringify!(result)),
                                                            vlog::concat_expr({
                                                                let mut ret = Vec::with_capacity(2usize);
                                                                ret.push(vlog::ident_expr(stringify!(temp2)));
                                                                ret.push(vlog::ident_expr(stringify!(temp1)));
                                                                ret
                                                            }),
                                                        ),
                                                    ),
                                                );
                                                ret.push(
                                                    vlog::case_line(
                                                        vlog::case_item_literal(
                                                            vlog::lit_verilog(2, stringify!(b10).into()),
                                                        ),
                                                        vlog::assign_stmt(
                                                            vlog::assign_target_ident(stringify!(result)),
                                                            vlog::concat_expr({
                                                                let mut ret = Vec::with_capacity(3usize);
                                                                ret.push(
                                                                    vlog::replica_expr(
                                                                        3,
                                                                        {
                                                                            let mut ret = Vec::with_capacity(1usize);
                                                                            ret.push(
                                                                                vlog::index_expr(stringify!(a), vlog::literal_expr(0), None),
                                                                            );
                                                                            ret
                                                                        },
                                                                    ),
                                                                );
                                                                ret.push(
                                                                    vlog::replica_expr(
                                                                        2,
                                                                        {
                                                                            let mut ret = Vec::with_capacity(1usize);
                                                                            ret.push(
                                                                                vlog::index_expr(
                                                                                    stringify!(b),
                                                                                    vlog::literal_expr(1),
                                                                                    Some(vlog::literal_expr(0)),
                                                                                ),
                                                                            );
                                                                            ret
                                                                        },
                                                                    ),
                                                                );
                                                                ret.push(
                                                                    vlog::index_expr(
                                                                        stringify!(temp1),
                                                                        vlog::literal_expr(6),
                                                                        Some(vlog::literal_expr(0)),
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
                                                            vlog::lit_verilog(2, stringify!(b11).into()),
                                                        ),
                                                        vlog::assign_stmt(
                                                            vlog::assign_target_ident(stringify!(result)),
                                                            vlog::binary_expr(
                                                                vlog::replica_expr(
                                                                    4,
                                                                    {
                                                                        let mut ret = Vec::with_capacity(1usize);
                                                                        ret.push(vlog::ident_expr(stringify!(a)));
                                                                        ret
                                                                    },
                                                                ),
                                                                vlog::binary_xor(),
                                                                vlog::replica_expr(
                                                                    4,
                                                                    {
                                                                        let mut ret = Vec::with_capacity(1usize);
                                                                        ret.push(vlog::ident_expr(stringify!(b)));
                                                                        ret
                                                                    },
                                                                ),
                                                            ),
                                                        ),
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
