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
                            vlog::declaration_list(
                                vlog::wire(),
                                vlog::unsigned(0..=4),
                                {
                                    let mut ret = Vec::with_capacity(1usize);
                                    ret.push(vlog::decl_kind(stringify!(val1), None));
                                    ret
                                },
                            ),
                        ),
                    );
                    ret.push(
                        vlog::declaration_item(
                            vlog::declaration_list(
                                vlog::reg(),
                                vlog::signed(0..=3),
                                {
                                    let mut ret = Vec::with_capacity(1usize);
                                    ret.push(vlog::decl_kind(stringify!(val2), None));
                                    ret
                                },
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
