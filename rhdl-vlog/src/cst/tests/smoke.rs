use crate::cst::{ModuleList, tests::common::test_parse};
use quote::quote;

#[test]
fn test_vlog_files() -> miette::Result<()> {
    let dir = std::fs::read_dir("vlog").unwrap();
    for entry in dir {
        let Ok(entry) = entry else {
            continue;
        };
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_file() {
            continue;
        };
        eprintln!("Path: {:?}", entry.path());
        let path = entry.path();
        let Some(extension) = path.extension() else {
            continue;
        };
        if extension != "v" {
            continue;
        }
        let Ok(module) = std::fs::read(entry.path()) else {
            continue;
        };
        let text = String::from_utf8_lossy(&module);
        let module_list = test_parse::<ModuleList>(text)?;
        let requote = quote! {#module_list ;}.to_string();
        let _ = syn::parse_str::<syn::Stmt>(&requote)
            .map_err(|err| syn_miette::Error::new(err, requote))?;
    }
    Ok(())
}
