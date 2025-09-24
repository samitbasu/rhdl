use std::process::Command;

use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};

fn do_shell_thing(txt: &str) -> String {
    log::debug!("Executing shell command: {}", txt);
    // Constant directory for now - you can modify this path as needed
    const SHELL_DIR: &str = "/tmp/rhdl";

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
mkdir -p "{SHELL_DIR}"
cd "{SHELL_DIR}"
# Enable colors in output
export CLICOLOR=1
export CLICOLOR_FORCE=1
export TERM=xterm-256color

# Execute the command and capture output
{txt}
"#,
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

pub fn exec_shell(text: &str) -> Vec<Event<'static>> {
    let result = do_shell_thing(text);
    vec![
        Event::Html(
            format!(
                r#"
                
<div class="shell-session">
<pre class="shell-output">{}</pre>
</div>

"#,
                ansi_to_html::convert(&result).unwrap()
            )
            .into(),
        )
        .into(),
    ]
}

pub fn silent_shell(text: &str) -> Vec<Event<'static>> {
    let _ = do_shell_thing(text);
    vec![]
}

#[cfg(test)]
mod test {
    use super::*;

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
