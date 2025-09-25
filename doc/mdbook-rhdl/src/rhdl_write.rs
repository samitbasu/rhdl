use std::path::PathBuf;

use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};

use crate::exec_shell::SHELL_DIR;

pub const WRITE_PREFIX: &str = "rust,write";

fn remove_hash_commented_lines(text: &str) -> String {
    text.lines()
        .filter(|line| !line.starts_with("# "))
        .collect::<Vec<_>>()
        .join("\n")
}

fn remove_hash_prefix(text: &str) -> String {
    text.lines()
        .map(|line| {
            if let Some(stripped) = line.strip_prefix("# ") {
                stripped.trim_start()
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn rhdl_write(tag: &str, text: &str) -> Vec<Event<'static>> {
    let text = text.to_string();
    let path = tag
        .strip_prefix(WRITE_PREFIX)
        .unwrap()
        .strip_prefix(":")
        .unwrap_or("");
    std::fs::write(
        PathBuf::from(SHELL_DIR).join(path),
        remove_hash_prefix(&text),
    )
    .unwrap();
    vec![
        Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced("rust".into()))).into(),
        Event::Text(remove_hash_commented_lines(&text).into()).into(),
        Event::End(TagEnd::CodeBlock).into(),
    ]
}
