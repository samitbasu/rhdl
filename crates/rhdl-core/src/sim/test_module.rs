//! Lightweight test module for using Icarus Verilog
use crate::RHDLError;
use rhdl_vlog as vlog;

/// A simple test module that can be used to run Verilog simulations
/// using Icarus Verilog.
pub struct TestModule(vlog::ModuleList);

impl From<vlog::ModuleList> for TestModule {
    fn from(m: vlog::ModuleList) -> Self {
        Self(m)
    }
}

impl std::fmt::Display for TestModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TestModule {
    /// Run the test module using Icarus Verilog.
    /// The test module should include a test bench that
    /// prints "TESTBENCH OK" on success, and "FAILED" on failure.
    pub fn run_iverilog(&self) -> Result<(), RHDLError> {
        let d = tempfile::tempdir()?;
        // Write the test bench to a file
        let d_path = d.path();
        std::fs::write(d_path.join("testbench.v"), self.to_string())?;
        // Compile the test bench
        let mut cmd = std::process::Command::new("iverilog");
        cmd.arg("-o")
            .arg(d_path.join("testbench"))
            .arg(d_path.join("testbench.v"));
        let status = cmd
            .status()
            .expect("Icarus Verilog should be installed and in your PATH.");
        if !status.success() {
            return Err(anyhow::anyhow!("Failed to compile testbench with {}", status).into());
        }
        let mut cmd = std::process::Command::new("vvp");
        cmd.arg(d_path.join("testbench"));
        let output = cmd.output()?;
        let output_stdout = String::from_utf8_lossy(&output.stdout);
        for line in output_stdout.lines() {
            if line.contains("FAILED") {
                return Err(RHDLError::VerilogVerificationErrorString(line.into()));
            }
            if line.starts_with("TESTBENCH OK") {
                return Ok(());
            }
        }
        Err(RHDLError::VerilogVerificationErrorString(
            "No output".into(),
        ))
    }
}
