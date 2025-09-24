use mdbook::{
    BookItem,
    book::{Book, Chapter},
    errors::Error,
    preprocess::{Preprocessor, PreprocessorContext},
};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag, TagEnd};
use std::process::Command;

// The Rhdl preprocessor.
pub struct Rhdl;

fn exec_shell(tst: &str) -> String {
    // First, create a shell code block for the command (with syntax highlighting)
    let command_block = format!("\n```shell\n{}\n```\n", tst.trim());

    // Then, get the terminal output with styled prompt
    let terminal_output = do_shell_thing(tst);
    let html = ansi_to_html::convert(&terminal_output).unwrap();

    // Combine both: syntax-highlighted command + styled terminal output
    format!(
        r#"{}
<div class="shell-session">
<pre class="shell-output">{}</pre>
</div>
"#,
        command_block, html
    )
}

fn do_shell_thing(txt: &str) -> String {
    log::debug!("Executing shell command: {}", txt);
    // Constant directory for now - you can modify this path as needed
    const SHELL_DIR: &str = "/Users/samitbasu/Devel/rhdl/doc/mdbook-rhdl";

    // Get the current user and hostname for the prompt
    let user = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
    let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| {
        // Try to get hostname from the system
        std::process::Command::new("hostname")
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout).ok()
                } else {
                    None
                }
            })
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "testdir".to_string())
    });

    let short_hostname = hostname.split('.').next().unwrap_or(&hostname);

    // Create the shell script that will execute the commands
    let script = format!(
        r#"#!/bin/zsh
cd "{}"
# Enable colors in output
export CLICOLOR=1
export CLICOLOR_FORCE=1
export TERM=xterm-256color

# Execute the command and capture output
{}
"#,
        SHELL_DIR, txt
    );

    // Execute the script with zsh
    let output = Command::new("zsh")
        .arg("-c")
        .arg(&script)
        .current_dir(SHELL_DIR)
        .output();

    let command_output = match output {
        Ok(output) => {
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout).to_string()
            } else {
                // Include stderr if the command failed
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                format!("{}{}", stdout, stderr)
            }
        }
        Err(e) => {
            format!("Error executing command: {}", e)
        }
    };

    // Create a properly formatted prompt with ANSI colors for styling
    let prompt_with_command = format!(
        "\x1b[32m{}\x1b[0m@\x1b[33m{}\x1b[0m \x1b[36m%\x1b[0m \x1b[1m{}\x1b[0m\n{}",
        user,           // Green user
        short_hostname, // Yellow hostname
        txt,            // Bold command
        command_output  // Actual command output
    );

    prompt_with_command
}

impl Rhdl {
    fn process_chapter(chapter: &mut Chapter) {
        let parser = pulldown_cmark::Parser::new(&chapter.content);
        let mut buf = String::with_capacity(chapter.content.len() + 128);
        // Inspired by svgbob2 mdbook preprocessor.

        let mut in_block = false;
        let mut block_text = String::new();
        let events = parser.filter_map(|event| match (&event, in_block) {
            (
                Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(CowStr::Borrowed("rhdl-shell")))),
                false,
            ) => {
                in_block = true;
                block_text.clear();
                None
            }
            (Event::Text(content), true) => {
                block_text.push_str(content);
                None
            }
            (Event::End(TagEnd::CodeBlock), true) => {
                in_block = false;
                Some(Event::Html(exec_shell(&block_text).into()))
            }
            _ => Some(event),
        });
        pulldown_cmark_to_cmark::cmark(events, &mut buf).unwrap();
        chapter.content = buf;
    }
}

impl Preprocessor for Rhdl {
    fn name(&self) -> &str {
        "rhdl"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_formal_mode_works() {
        let md = r##"
# Chapter 1

Here is a diagram of the mascot.
```rhdl-shell
ls -la --color=always
```
        "##;
        let mut chapter = Chapter {
            name: "Test".into(),
            content: md.into(),
            number: None,
            sub_items: vec![],
            path: None,
            source_path: None,
            parent_names: vec![],
        };
        Rhdl::process_chapter(&mut chapter);
        let expect = expect_test::expect_file!["./test_formal.md"];
        expect.assert_eq(&chapter.content);
    }

    #[test]
    fn test_shell_function() {
        let result = do_shell_thing("ls");
        println!("Shell output:\n{}", result);

        // Basic checks - the output should contain the prompt and some files
        assert!(result.contains("@"));
        assert!(result.contains("%"));
        assert!(result.contains("ls"));
    }

    #[test]
    fn test_shell_with_colors() {
        let result = do_shell_thing("ls -la --color=always");
        println!("Colored shell output:\n{}", result);

        // The output should contain ANSI escape sequences for colors
        assert!(result.contains("@"));
        assert!(result.contains("%"));
        assert!(result.contains("ls -la --color=always"));
    }
}
