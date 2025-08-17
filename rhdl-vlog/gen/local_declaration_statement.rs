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
                let elem0 = vlog::declaration_item(
                    vlog::declaration(
                        vlog::wire(),
                        vlog::unsigned(0..=4),
                        stringify!(val1),
                    ),
                );
                let elem1 = vlog::declaration_item(
                    vlog::declaration(vlog::reg(), vlog::signed(0..=3), stringify!(val2)),
                );
                vec![elem0, elem1]
            },
        );
        vec![elem0]
    });
}
