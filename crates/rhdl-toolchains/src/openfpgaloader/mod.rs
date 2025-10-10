//! Functions that are useful for working with openfpgaloader

pub fn ice40_generic(path: impl AsRef<camino::Utf8Path>) -> miette::Result<()> {
    let path = path.as_ref();
    let mut cmd = std::process::Command::new("openfpgaloader");
    cmd.current_dir(&path);
    cmd.arg("-b").arg("ice40_generic").arg("rhdl.bin");
    let output = cmd.output().map_err(|e| miette::miette!(e.to_string()))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        return Err(miette::miette!(format!(
            "openfpgaloader failed.\nSTDOUT:\n{stdout}\nSTDERR:\n{stderr}"
        )));
    }
    Ok(())
}
