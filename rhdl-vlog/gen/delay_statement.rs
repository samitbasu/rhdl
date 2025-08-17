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
                    vlog::always_stmt(
                        {
                            let elem0 = vlog::pos_edge(stringify!(a));
                            let elem1 = vlog::neg_edge(stringify!(b));
                            let elem2 = vlog::star();
                            vec![elem0, elem1, elem2]
                        },
                        vlog::block_stmt({
                            let elem0 = vlog::nonblock_assign_stmt(
                                stringify!(b),
                                vlog::literal_expr(1),
                            );
                            let elem1 = vlog::delay_stmt(10);
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
