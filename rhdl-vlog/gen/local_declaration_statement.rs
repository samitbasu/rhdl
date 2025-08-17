pub mod vlog {
    include!("../src/ast.rs");
}
fn main() {
    let _ = vlog::module_list({
        let mut ret = Vec::with_capacity(1usize);
        ret.push(
            vlog::module_def(
                stringify!(foo),
                {
                    let mut ret = Vec::with_capacity(2usize);
                    ret.push(
                        vlog::port(
                            vlog::input(),
                            vlog::declaration(
                                vlog::wire(),
                                vlog::unsigned(0..=2),
                                stringify!(a),
                            ),
                        ),
                    );
                    ret.push(
                        vlog::port(
                            vlog::output(),
                            vlog::declaration(
                                vlog::reg(),
                                vlog::unsigned(0..=1),
                                stringify!(b),
                            ),
                        ),
                    );
                    ret
                },
                {
                    let mut ret = Vec::with_capacity(2usize);
                    ret.push(
                        vlog::declaration_item(
                            vlog::declaration(
                                vlog::wire(),
                                vlog::unsigned(0..=4),
                                stringify!(val1),
                            ),
                        ),
                    );
                    ret.push(
                        vlog::declaration_item(
                            vlog::declaration(
                                vlog::reg(),
                                vlog::signed(0..=3),
                                stringify!(val2),
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
