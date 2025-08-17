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
                    vlog::input(),
                    vlog::declaration(vlog::wire(), vlog::unsigned(0..=1), stringify!(c)),
                );
                let elem2 = vlog::port(
                    vlog::output(),
                    vlog::declaration(vlog::reg(), vlog::unsigned(0..=1), stringify!(b)),
                );
                vec![elem0, elem1, elem2]
            },
            {
                let elem0 = vlog::stmt_item(
                    vlog::concat_assign_stmt(
                        {
                            let elem0 = vlog::ident_expr(stringify!(a));
                            let elem1 = vlog::ident_expr(stringify!(c));
                            vec![elem0, elem1]
                        },
                        vlog::concat_expr({
                            let elem0 = vlog::constant_expr(
                                vlog::lit_verilog(1, stringify!(b0)),
                            );
                            let elem1 = vlog::ident_expr(stringify!(a));
                            vec![elem0, elem1]
                        }),
                    ),
                );
                vec![elem0]
            },
        );
        vec![elem0]
    });
}
