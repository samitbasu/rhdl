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
                stringify!(test_mixed),
                {
                    let mut ret = Vec::with_capacity(4usize);
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
                                vlog::unsigned(0..=3),
                                stringify!(data_in),
                            ),
                        ),
                    );
                    ret.push(
                        vlog::port(
                            vlog::output(),
                            vlog::declaration(
                                vlog::reg(),
                                vlog::unsigned(0..=7),
                                stringify!(data_out),
                            ),
                        ),
                    );
                    ret.push(
                        vlog::port(
                            vlog::output(),
                            vlog::declaration(
                                vlog::wire(),
                                vlog::unsigned(0..=0),
                                stringify!(valid),
                            ),
                        ),
                    );
                    ret
                },
                {
                    let mut ret = Vec::with_capacity(5usize);
                    ret.push(
                        vlog::stmt_item(
                            vlog::local_param_stmt(
                                stringify!(INIT_VAL),
                                vlog::const_verilog(
                                    vlog::lit_verilog(8, stringify!(hAA).into()),
                                ),
                            ),
                        ),
                    );
                    ret.push(
                        vlog::stmt_item(
                            vlog::local_param_stmt(
                                stringify!(THRESHOLD),
                                vlog::const_verilog(
                                    vlog::lit_verilog(4, stringify!(b1010).into()),
                                ),
                            ),
                        ),
                    );
                    ret.push(
                        vlog::stmt_item(
                            vlog::local_param_stmt(
                                stringify!(NAME),
                                vlog::const_str("Test Module"),
                            ),
                        ),
                    );
                    ret.push(
                        vlog::stmt_item(
                            vlog::always_stmt(
                                {
                                    let mut ret = Vec::with_capacity(1usize);
                                    ret.push(vlog::pos_edge(stringify!(clk)));
                                    ret
                                },
                                vlog::block_stmt({
                                    let mut ret = Vec::with_capacity(1usize);
                                    ret.push(
                                        vlog::if_stmt(
                                            vlog::binary_expr(
                                                vlog::ident_expr(stringify!(data_in)),
                                                vlog::binary_ge(),
                                                vlog::ident_expr(stringify!(THRESHOLD)),
                                            ),
                                            vlog::block_stmt({
                                                let mut ret = Vec::with_capacity(1usize);
                                                ret.push(
                                                    vlog::nonblock_assign_stmt(
                                                        vlog::assign_target_ident(stringify!(data_out)),
                                                        vlog::concat_expr({
                                                            let mut ret = Vec::with_capacity(2usize);
                                                            ret.push(vlog::ident_expr(stringify!(data_in)));
                                                            ret.push(
                                                                vlog::constant_expr(vlog::lit_verilog(4, stringify!(b0000))),
                                                            );
                                                            ret
                                                        }),
                                                    ),
                                                );
                                                ret
                                            }),
                                            Some(
                                                vlog::block_stmt({
                                                    let mut ret = Vec::with_capacity(1usize);
                                                    ret.push(
                                                        vlog::nonblock_assign_stmt(
                                                            vlog::assign_target_ident(stringify!(data_out)),
                                                            vlog::ident_expr(stringify!(INIT_VAL)),
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
                    ret.push(
                        vlog::stmt_item(
                            vlog::continuous_assign_stmt(
                                vlog::assign_target_ident(stringify!(valid)),
                                vlog::paren_expr(
                                    vlog::binary_expr(
                                        vlog::ident_expr(stringify!(data_out)),
                                        vlog::binary_ne(),
                                        vlog::constant_expr(vlog::lit_verilog(8, stringify!(h00))),
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
    });
}
