use mdbook::{
    BookItem,
    book::{Book, Chapter},
    errors::Error,
    preprocess::{Preprocessor, PreprocessorContext},
};
mod exec_shell;
mod rewrite_block;
mod rhdl_write;
use exec_shell::{exec_shell, silent_shell};
use rewrite_block::BlockRewriterExt;

use crate::{
    exec_shell::{SHELL_PREFIX, SHELL_SILENT_PREFIX},
    rhdl_write::WRITE_PREFIX,
};

// The Rhdl preprocessor.
pub struct Rhdl;

impl Rhdl {
    fn process_chapter(chapter: &mut Chapter) {
        let parser = pulldown_cmark::Parser::new(&chapter.content);
        let mut buf = String::with_capacity(chapter.content.len() + 128);
        let source_path = chapter.source_path.clone();
        let events = parser.into_iter();
        let events = events.rewrite_blocks(&source_path, silent_shell, SHELL_SILENT_PREFIX);
        let events = events.rewrite_blocks(&source_path, rhdl_write::rhdl_write, WRITE_PREFIX);
        let events = events.rewrite_blocks(&source_path, exec_shell, SHELL_PREFIX);
        pulldown_cmark_to_cmark::cmark(events, &mut buf).unwrap();
        chapter.content = buf;
    }
}

impl Preprocessor for Rhdl {
    fn name(&self) -> &str {
        "rhdl"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        // In testing we want to tell the preprocessor to blow up by setting a
        // particular config value
        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                log::info!("Processing chapter: {}", chapter.name);
                Self::process_chapter(chapter);
            }
        });

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer != "not-supported"
    }
}
