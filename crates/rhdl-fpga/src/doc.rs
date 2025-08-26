use rhdl::prelude::*;
use std::path::PathBuf;

#[doc(hidden)]
/// Useful for testing, but otherwise, probably not for end users
pub fn write_svg_as_markdown(vcd: Vcd, name: &str, options: SvgOptions) -> anyhow::Result<()> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = path.join("doc");
    std::fs::create_dir_all(&path)?;
    let path = path.join(name);
    std::fs::write(path, format!("\n\n<p>\n{}\n</p>", vcd.dump_svg(&options)))?;
    Ok(())
}
