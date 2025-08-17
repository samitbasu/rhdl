mod rhdl {
    pub mod vlog {
        include!("../src/ast.rs");
    }
}
fn main() {
    let _ = rhdl::vlog::ModuleList(
        vec![
            { let args_vec = vec![]; let items_vec = vec![]; rhdl::vlog::ModuleDef { name
            : stringify!(foo) .into(), args : args_vec, items : items_vec, } },
        ],
    );
}
