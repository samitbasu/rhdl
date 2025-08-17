pub mod vlog {
    include!("../src/ast.rs");
}
fn main() {
    let _ = vlog::module_list({
        let elem0 = vlog::module_def(
            stringify!(foo),
            {
                let elem0 = vlog::port(
                    vlog::input(),
                    vlog::declaration(vlog::wire(), vlog::unsigned(0..=1), stringify!(a)),
                );
                let elem1 = vlog::port(
                    vlog::output(),
                    vlog::declaration(vlog::reg(), vlog::unsigned(0..=1), stringify!(b)),
                );
                vec![elem0, elem1]
            },
            {
                let elem0 = vlog::stmt_item(
                    vlog::case_stmt(
                        vlog::ident_expr(stringify!(a)),
                        {
                            let elem0 = vlog::case_line(
                                vlog::case_item_literal(
                                    vlog::lit_verilog(2, stringify!(b00).into()),
                                ),
                                vlog::assign_stmt(stringify!(b), vlog::literal_expr(1)),
                            );
                            let elem1 = vlog::case_line(
                                vlog::case_item_literal(
                                    vlog::lit_verilog(2, stringify!(b01).into()),
                                ),
                                vlog::assign_stmt(stringify!(b), vlog::literal_expr(2)),
                            );
                            let elem2 = vlog::case_line(
                                vlog::case_item_literal(
                                    vlog::lit_verilog(2, stringify!(b10).into()),
                                ),
                                vlog::assign_stmt(stringify!(b), vlog::literal_expr(3)),
                            );
                            let elem3 = vlog::case_line(
                                vlog::case_item_wild(),
                                vlog::assign_stmt(stringify!(b), vlog::literal_expr(4)),
                            );
                            vec![elem0, elem1, elem2, elem3]
                        },
                    ),
                );
                vec![elem0]
            },
        );
        vec![elem0]
    });
}
