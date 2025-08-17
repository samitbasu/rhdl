pub mod vlog {
    include!("../src/ast.rs");
}
fn main() {
    let _ = vlog::module_list({
        let elem0 = vlog::module_def(stringify!(foo), vec![], vec![]);
        vec![elem0]
    });
}
