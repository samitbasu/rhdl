use miette::Diagnostic;
use rhdl_vlog::Pretty;
use thiserror::Error;

use crate::{HDLDescriptor, RHDLError};

#[derive(Error, Debug, Diagnostic)]
pub enum YosysSynthError {
    #[error("Synthesis failed: {stdout} {stderr}")]
    SynthesisFailed { stdout: String, stderr: String },
    #[error("Latching write to signals {0:?}")]
    LatchingWriteToSignal(Vec<String>),
    #[error("Implicitly declared signals {0:?}")]
    ImplicitlyDeclared(Vec<String>),
    #[error("Duplicate modules {0:?}")]
    DuplicateModule(Vec<String>),
    #[error("IOError {0}")]
    IOError(std::io::Error),
    #[error("Wire has no driver {0:?}")]
    WireHasNoDriver(Vec<String>),
    #[error("Missing module {0:?}")]
    MissingModule(Vec<String>),
}

pub fn run_yosys_synth(hdl: HDLDescriptor) -> Result<(), RHDLError> {
    let module = hdl.as_module();
    let verilog = module.pretty();
    let d = tempfile::tempdir()?;
    let d_path = d.path();
    std::fs::write(d_path.join("top.v"), &verilog)?;
    std::fs::write("top.v", verilog)?;
    let mut cmd = std::process::Command::new("yosys");
    cmd.current_dir(d_path);
    let arg = "-p read -vlog95 top.v; hierarchy -check -top top; proc";
    cmd.arg(arg);
    let output = cmd
        .output()
        .expect("yosys should be installed and in your PATH.");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    if !output.status.success() {
        return Err(YosysSynthError::SynthesisFailed { stdout, stderr }.into());
    }
    fn capture(stdout: &str, reg_exp: &str) -> Vec<String> {
        let regex = regex::Regex::new(reg_exp).unwrap();
        let mut signal_name = vec![];
        if regex.is_match(stdout) {
            for capture in regex.captures(stdout).unwrap().iter() {
                signal_name.push(capture.unwrap().as_str().to_string());
            }
        }
        signal_name
    }
    if stdout.contains("Re-definition of") {
        return Err(YosysSynthError::DuplicateModule(capture(
            &stdout,
            r#"Re-definition of module (\S*)"#,
        ))
        .into());
    }
    if stdout.contains("implicitly declared.") {
        return Err(YosysSynthError::ImplicitlyDeclared(capture(
            &stdout,
            r#"Identifier (\S*) is implicitly declared"#,
        ))
        .into());
    }
    if stdout.contains("Latch inferred for") {
        return Err(YosysSynthError::LatchingWriteToSignal(capture(
            &stdout,
            r#"Latch inferred for signal (\S*)"#,
        ))
        .into());
    }
    if stdout.contains("is used but has no driver") {
        return Err(YosysSynthError::WireHasNoDriver(capture(
            &stdout,
            r#"Wire (\S*) .*? is used but has no driver."#,
        ))
        .into());
    }
    if stderr.contains("is not part of the design") {
        return Err(YosysSynthError::MissingModule(capture(
            &stderr,
            r#"Module (\S*) .*? is not part of the design"#,
        ))
        .into());
    }
    if !stdout.contains("End of script.") {
        return Err(YosysSynthError::SynthesisFailed { stdout, stderr }.into());
    }
    Ok(())
}
