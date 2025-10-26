use std::{
    path::{Path, PathBuf},
    process::Command,
};

use pulldown_cmark::{Event, Tag, TagEnd};
const PROMPT_STR: &str = "❯";
pub const SHELL_DIR: &str = "/tmp/rhdl/";
pub const SHELL_PREFIX: &str = "shell,rhdl";
pub const SHELL_SILENT_PREFIX: &str = "shell,rhdl-silent";

fn do_shell_thing(start_dir: &str, txt: &str) -> String {
    log::info!("Executing shell command: {}", txt);
    // Constant directory for now - you can modify this path as needed

    // Split input into individual commands
    let commands: Vec<&str> = txt
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();

    if commands.is_empty() {
        return String::new();
    }

    let mut result = String::new();
    let shell_dir = Path::new(SHELL_DIR);
    let mut current_dir = shell_dir.join(start_dir);

    // Initialize the shell directory
    let _ = Command::new("mkdir")
        .args(["-p", current_dir.to_str().unwrap()])
        .output();

    for command in commands {
        // Get the relative path from the shell directory
        let relative_path = current_dir.strip_prefix(shell_dir).unwrap().display();
        // Create and display the prompt
        let prompt =
            format!("\x1b[36m{relative_path} {PROMPT_STR}\x1b[0m \x1b[1m{command}\x1b[0m",);
        result.push_str(&prompt);
        result.push('\n');

        // Create the shell script for this individual command
        let script = format!(
            r#"#!/bin/zsh
export ROOT_DIR=`pwd`
cd "{}"
# Enable colors in output
export CLICOLOR=1
export CLICOLOR_FORCE=1
export TERM=xterm-256color

# Execute the command and capture output
{command}

# Print the current directory for tracking
echo "___CURRENT_DIR___"
pwd
"#,
            current_dir.display()
        );

        log::info!("Running script:\n{}", script);
        // Execute the command
        let output = Command::new("zsh").arg("-c").arg(&script).output();

        match output {
            Ok(output) => {
                // Always capture both stdout and stderr
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let full_output = format!("{}{}", stdout, stderr);

                // Split the output to extract the current directory
                if let Some(dir_marker_pos) = full_output.find("___CURRENT_DIR___\n") {
                    let command_output = &full_output[..dir_marker_pos];
                    let dir_output = &full_output[dir_marker_pos + "___CURRENT_DIR___\n".len()..];

                    // Add the command output (everything before the directory marker)
                    result.push_str(command_output);

                    // Update current directory from the output
                    let dir_lines: Vec<&str> = dir_output.lines().collect();
                    if let Some(new_dir) = dir_lines.first() {
                        current_dir = PathBuf::from(new_dir);
                    }

                    // Add any remaining output after the directory line (like stderr content)
                    if dir_lines.len() > 1 {
                        for line in &dir_lines[1..] {
                            result.push_str(line);
                            result.push('\n');
                        }
                    }
                } else {
                    // Fallback if the directory marker is not found
                    result.push_str(&full_output);
                }
            }
            Err(e) => {
                result.push_str(&format!("Error executing command: {}\n", e));
            }
        }
    }

    result
}

pub fn exec_shell(_block: usize, tag: &str, text: &str) -> Vec<Event<'static>> {
    let start_dir = tag
        .strip_prefix(SHELL_PREFIX)
        .unwrap_or("")
        .strip_prefix(":")
        .unwrap_or("");
    let result = do_shell_thing(start_dir, text);
    let converter = ansi_to_html::Converter::new()
        .skip_escape(true)
        .skip_optimize(true);
    vec![
        Event::Start(Tag::HtmlBlock),
        Event::Html(
            format!(
                r#"<pre><code class="hljs hide-boring">{}</code></pre>"#,
                converter.convert(&result).unwrap()
            )
            .into(),
        ),
        Event::End(TagEnd::HtmlBlock),
    ]
}

pub fn silent_shell(_block: usize, tag: &str, text: &str) -> Vec<Event<'static>> {
    let tag = tag
        .strip_prefix(SHELL_SILENT_PREFIX)
        .unwrap_or("")
        .strip_prefix(":")
        .unwrap_or("");
    let _ = do_shell_thing(tag, text);
    vec![]
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_shell_function() {
        let result = do_shell_thing("", "ls");
        println!("Shell output:\n{}", result);

        // Basic checks - the output should contain the prompt and some files
        assert!(result.contains(PROMPT_STR));
        assert!(result.contains("ls"));
    }

    #[test]
    fn test_shell_with_colors() {
        let result = do_shell_thing("", "ls -la --color=always");
        println!("Colored shell output:\n{}", result);

        // The output should contain ANSI escape sequences for colors
        assert!(result.contains(PROMPT_STR));
        assert!(result.contains("ls -la --color=always"));
    }

    #[test]
    fn test_stderr_capture() {
        // Test with a command that writes to stderr
        let result = do_shell_thing("", "echo 'stdout message'; echo 'stderr message' >&2");
        println!("Stderr test output:\n{}", result);

        // Should contain both stdout and stderr output
        assert!(result.contains("stdout message"));
        assert!(result.contains("stderr message"));
        assert!(result.contains(PROMPT_STR));
    }

    #[test]
    fn test_error_command_stderr() {
        // Test with a command that fails and writes to stderr
        let result = do_shell_thing("", "cat /nonexistent/file/path");
        println!("Error command output:\n{}", result);

        // Should contain stderr error message (varies by system, but typically contains "No such file")
        assert!(result.contains("No such file") || result.contains("cannot access"));
        assert!(result.contains(PROMPT_STR));
    }
}
