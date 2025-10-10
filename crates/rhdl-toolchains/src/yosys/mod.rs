//! Functions that are useful for working with Yosys.

pub struct YosysOutput(pub String);

pub fn synth_ice40(
    code: rhdl_vlog::ModuleList,
    top: &str,
    path: impl AsRef<camino::Utf8Path>,
) -> miette::Result<YosysOutput> {
    let path = path.as_ref();
    std::fs::create_dir_all(path).map_err(|e| miette::miette!(e.to_string()))?;
    std::fs::write((&path).join("top.v"), code.to_string())
        .map_err(|e| miette::miette!(e.to_string()))?;
    // First synthesize it
    let mut cmd = std::process::Command::new("yosys");
    cmd.current_dir(&path);
    let arg = format!("-p synth_ice40 -top {top} -json rhdl.json");
    cmd.arg(arg).arg("top.v");
    let output = cmd.output().map_err(|e| miette::miette!(e.to_string()))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        return Err(miette::miette!(format!(
            "Yosys synthesis failed.\nSTDOUT:\n{stdout}\nSTDERR:\n{stderr}"
        )));
    }
    Ok(YosysOutput(stdout.into_owned()))
}
