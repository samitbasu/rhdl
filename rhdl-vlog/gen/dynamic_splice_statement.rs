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
                    vlog::declaration(vlog::wire(), vlog::unsigned(0..=2), stringify!(a)),
                );
                let elem1 = vlog::port(
                    vlog::output(),
                    vlog::declaration(vlog::reg(), vlog::unsigned(0..=1), stringify!(b)),
                );
                vec![elem0, elem1]
            },
            {
                let elem0 = vlog::stmt_item(
                    vlog::dynamic_splice_stmt(
                        stringify!(a),
                        vlog::ident_expr(stringify!(b)),
                        vlog::literal_expr(1),
                        vlog::literal_expr(1),
                    ),
                );
                vec![elem0]
            },
        );
        vec![elem0]
    });
}
