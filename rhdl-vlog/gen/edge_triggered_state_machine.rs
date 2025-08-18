pub mod vlog {
    include!("../src/ast.rs");
}
fn main() {
    let _ = vlog::module_list({
        let mut ret = Vec::with_capacity(1usize);
        ret.push(
            vlog::module_def(
                stringify!(state_machine),
                {
                    let mut ret = Vec::with_capacity(5usize);
                    ret.push(
                        vlog::port(
                            vlog::input(),
                            vlog::declaration(
                                vlog::wire(),
                                vlog::unsigned(0..=0),
                                stringify!(clk),
                            ),
                        ),
                    );
                    ret.push(
                        vlog::port(
                            vlog::input(),
                            vlog::declaration(
                                vlog::wire(),
                                vlog::unsigned(0..=0),
                                stringify!(rst),
                            ),
                        ),
                    );
                    ret.push(
                        vlog::port(
                            vlog::input(),
                            vlog::declaration(
                                vlog::wire(),
                                vlog::unsigned(0..=0),
                                stringify!(start),
                            ),
                        ),
                    );
                    ret.push(
                        vlog::port(
                            vlog::output(),
                            vlog::declaration(
                                vlog::reg(),
                                vlog::unsigned(0..=1),
                                stringify!(state),
                            ),
                        ),
                    );
                    ret.push(
                        vlog::port(
                            vlog::output(),
                            vlog::declaration(
                                vlog::reg(),
                                vlog::unsigned(0..=0),
                                stringify!(done),
                            ),
                        ),
                    );
                    ret
                },
                {
                    let mut ret = Vec::with_capacity(6usize);
                    ret.push(
                        vlog::stmt_item(
                            vlog::local_param_stmt(
                                stringify!(IDLE),
                                vlog::const_verilog(
                                    vlog::lit_verilog(2, stringify!(b00).into()),
                                ),
                            ),
                        ),
                    );
                    ret.push(
                        vlog::stmt_item(
                            vlog::local_param_stmt(
                                stringify!(ACTIVE),
                                vlog::const_verilog(
                                    vlog::lit_verilog(2, stringify!(b01).into()),
                                ),
                            ),
                        ),
                    );
                    ret.push(
                        vlog::stmt_item(
                            vlog::local_param_stmt(
                                stringify!(WAIT),
                                vlog::const_verilog(
                                    vlog::lit_verilog(2, stringify!(b10).into()),
                                ),
                            ),
                        ),
                    );
                    ret.push(
                        vlog::stmt_item(
                            vlog::local_param_stmt(
                                stringify!(COMPLETE),
                                vlog::const_verilog(
                                    vlog::lit_verilog(2, stringify!(b11).into()),
                                ),
                            ),
                        ),
                    );
                    ret.push(
                        vlog::declaration_item(
                            vlog::declaration_list(
                                vlog::reg(),
                                vlog::unsigned(0..=3),
                                {
                                    let mut ret = Vec::with_capacity(1usize);
                                    ret.push(vlog::decl_kind(stringify!(counter), None));
                                    ret
                                },
                            ),
                        ),
                    );
                    ret.push(
                        vlog::stmt_item(
                            vlog::always_stmt(
                                {
                                    let mut ret = Vec::with_capacity(2usize);
                                    ret.push(vlog::pos_edge(stringify!(clk)));
                                    ret.push(vlog::pos_edge(stringify!(rst)));
                                    ret
                                },
                                vlog::block_stmt({
                                    let mut ret = Vec::with_capacity(1usize);
                                    ret.push(
                                        vlog::if_stmt(
                                            vlog::ident_expr(stringify!(rst)),
                                            vlog::block_stmt({
                                                let mut ret = Vec::with_capacity(3usize);
                                                ret.push(
                                                    vlog::nonblock_assign_stmt(
                                                        vlog::assign_target_ident(stringify!(state)),
                                                        vlog::ident_expr(stringify!(IDLE)),
                                                    ),
                                                );
                                                ret.push(
                                                    vlog::nonblock_assign_stmt(
                                                        vlog::assign_target_ident(stringify!(counter)),
                                                        vlog::constant_expr(vlog::lit_verilog(4, stringify!(b0000))),
                                                    ),
                                                );
                                                ret.push(
                                                    vlog::nonblock_assign_stmt(
                                                        vlog::assign_target_ident(stringify!(done)),
                                                        vlog::constant_expr(vlog::lit_verilog(1, stringify!(b0))),
                                                    ),
                                                );
                                                ret
                                            }),
                                            Some(
                                                vlog::block_stmt({
                                                    let mut ret = Vec::with_capacity(1usize);
                                                    ret.push(
                                                        vlog::case_stmt(
                                                            vlog::ident_expr(stringify!(state)),
                                                            {
                                                                let mut ret = Vec::with_capacity(4usize);
                                                                ret.push(
                                                                    vlog::case_line(
                                                                        vlog::case_item_ident(stringify!(IDLE)),
                                                                        vlog::block_stmt({
                                                                            let mut ret = Vec::with_capacity(2usize);
                                                                            ret.push(
                                                                                vlog::if_stmt(
                                                                                    vlog::ident_expr(stringify!(start)),
                                                                                    vlog::block_stmt({
                                                                                        let mut ret = Vec::with_capacity(2usize);
                                                                                        ret.push(
                                                                                            vlog::nonblock_assign_stmt(
                                                                                                vlog::assign_target_ident(stringify!(state)),
                                                                                                vlog::ident_expr(stringify!(ACTIVE)),
                                                                                            ),
                                                                                        );
                                                                                        ret.push(
                                                                                            vlog::nonblock_assign_stmt(
                                                                                                vlog::assign_target_ident(stringify!(counter)),
                                                                                                vlog::constant_expr(vlog::lit_verilog(4, stringify!(b0000))),
                                                                                            ),
                                                                                        );
                                                                                        ret
                                                                                    }),
                                                                                    None,
                                                                                ),
                                                                            );
                                                                            ret.push(
                                                                                vlog::nonblock_assign_stmt(
                                                                                    vlog::assign_target_ident(stringify!(done)),
                                                                                    vlog::constant_expr(vlog::lit_verilog(1, stringify!(b0))),
                                                                                ),
                                                                            );
                                                                            ret
                                                                        }),
                                                                    ),
                                                                );
                                                                ret.push(
                                                                    vlog::case_line(
                                                                        vlog::case_item_ident(stringify!(ACTIVE)),
                                                                        vlog::block_stmt({
                                                                            let mut ret = Vec::with_capacity(2usize);
                                                                            ret.push(
                                                                                vlog::nonblock_assign_stmt(
                                                                                    vlog::assign_target_ident(stringify!(counter)),
                                                                                    vlog::binary_expr(
                                                                                        vlog::ident_expr(stringify!(counter)),
                                                                                        vlog::binary_plus(),
                                                                                        vlog::literal_expr(1),
                                                                                    ),
                                                                                ),
                                                                            );
                                                                            ret.push(
                                                                                vlog::if_stmt(
                                                                                    vlog::binary_expr(
                                                                                        vlog::ident_expr(stringify!(counter)),
                                                                                        vlog::binary_eq(),
                                                                                        vlog::constant_expr(vlog::lit_verilog(4, stringify!(b1111))),
                                                                                    ),
                                                                                    vlog::block_stmt({
                                                                                        let mut ret = Vec::with_capacity(1usize);
                                                                                        ret.push(
                                                                                            vlog::nonblock_assign_stmt(
                                                                                                vlog::assign_target_ident(stringify!(state)),
                                                                                                vlog::ident_expr(stringify!(WAIT)),
                                                                                            ),
                                                                                        );
                                                                                        ret
                                                                                    }),
                                                                                    None,
                                                                                ),
                                                                            );
                                                                            ret
                                                                        }),
                                                                    ),
                                                                );
                                                                ret.push(
                                                                    vlog::case_line(
                                                                        vlog::case_item_ident(stringify!(WAIT)),
                                                                        vlog::block_stmt({
                                                                            let mut ret = Vec::with_capacity(1usize);
                                                                            ret.push(
                                                                                vlog::nonblock_assign_stmt(
                                                                                    vlog::assign_target_ident(stringify!(state)),
                                                                                    vlog::ident_expr(stringify!(COMPLETE)),
                                                                                ),
                                                                            );
                                                                            ret
                                                                        }),
                                                                    ),
                                                                );
                                                                ret.push(
                                                                    vlog::case_line(
                                                                        vlog::case_item_ident(stringify!(COMPLETE)),
                                                                        vlog::block_stmt({
                                                                            let mut ret = Vec::with_capacity(2usize);
                                                                            ret.push(
                                                                                vlog::nonblock_assign_stmt(
                                                                                    vlog::assign_target_ident(stringify!(done)),
                                                                                    vlog::constant_expr(vlog::lit_verilog(1, stringify!(b1))),
                                                                                ),
                                                                            );
                                                                            ret.push(
                                                                                vlog::nonblock_assign_stmt(
                                                                                    vlog::assign_target_ident(stringify!(state)),
                                                                                    vlog::ident_expr(stringify!(IDLE)),
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
