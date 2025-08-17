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
                    vlog::declaration(vlog::wire(), vlog::unsigned(0..=1), stringify!(b)),
                );
                let elem2 = vlog::port(
                    vlog::inout(),
                    vlog::declaration(vlog::reg(), vlog::unsigned(0..=3), stringify!(c)),
                );
                vec![elem0, elem1, elem2]
            },
            vec![],
        );
        let elem1 = vlog::module_def(
            stringify!(bar),
            {
                let elem0 = vlog::port(
                    vlog::input(),
                    vlog::declaration(vlog::wire(), vlog::signed(0..=1), stringify!(c)),
                );
                let elem1 = vlog::port(
                    vlog::output(),
                    vlog::declaration(vlog::reg(), vlog::signed(0..=1), stringify!(d)),
                );
                vec![elem0, elem1]
            },
            vec![],
        );
        vec![elem0, elem1]
    });
}
