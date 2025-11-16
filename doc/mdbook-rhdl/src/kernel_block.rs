use std::path::PathBuf;

use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};

use crate::exec_shell::SHELL_DIR;

pub const KERNEL_PREFIX: &str = "rust,kernel";

pub const KERNEL_DIR: &str = "kernels/tests/";

const TEST_HARNESS: &str = r#"
use rhdl::prelude::*;

##

#[test]
fn test_kernel_block() {
    compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
}
"#;

pub fn rhdl_write(block: usize, tag: &str, text: &str) -> Vec<Event<'static>> {
    let prefix = tag
        .strip_prefix(KERNEL_PREFIX)
        .unwrap()
        .strip_prefix(":")
        .unwrap_or_default();
    let text = text.to_string();
    let path = PathBuf::from(SHELL_DIR).join(KERNEL_DIR);
    std::fs::create_dir_all(&path).unwrap();
    let form = path.join(format!("{prefix}_{}.rs", block));
    let full_text = TEST_HARNESS.replace("##", &text);
    std::fs::write(&form, &full_text).unwrap();
    vec![
        Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced("rust".into()))),
        Event::Text(text.into()),
        Event::End(TagEnd::CodeBlock),
    ]
}
