use rhdl::core::fsm::analysis::Transition;
use rhdl::core::fsm::diagram::{build_fsm_diagram, render_fsm_svg};
use rhdl::prelude::*;
use std::path::PathBuf;

#[doc(hidden)]
/// Useful for testing, but otherwise, probably not for end users
pub fn write_svg_as_markdown(vcd: SvgFile, name: &str, options: SvgOptions) -> anyhow::Result<()> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = path.join("doc");
    std::fs::create_dir_all(&path)?;
    let path = path.join(name);
    std::fs::write(path, format!("\n\n<p>\n{}\n</p>", vcd.to_string(&options)?))?;
    Ok(())
}

/// Render the FSM diagram for a `#[derive(FsmWidget)]`-tagged widget
/// as a self-contained inline-SVG markdown fragment.  Required by
/// every FSM-tagged widget per CLAUDE.md §12 rule 14.
///
/// The caller passes a manually-curated transition list — until
/// the RHIF-extraction pass is wired into the rustdoc emission
/// pipeline, the widget author records the transitions in source
/// alongside the kernel.
pub fn render_fsm_diagram_markdown<W: FsmWidget>(transitions: &[Transition]) -> String {
    let desc = W::fsm_descriptor();
    let diagram = build_fsm_diagram(&desc, transitions);
    let svg = render_fsm_svg(&diagram);
    format!("\n\n<p>\n{svg}\n</p>\n")
}

/// Same as [`render_fsm_diagram_markdown`], but writes the result
/// directly to `doc/<filename>`.
///
/// Convention: widgets named `<name>` write their FSM diagram to
/// `doc/<name>_fsm.md`, and include it in their rustdoc with
/// `#![doc = include_str!("../../doc/<name>_fsm.md")]`.
pub fn write_fsm_diagram_as_markdown<W: FsmWidget>(
    transitions: &[Transition],
    filename: &str,
) -> std::io::Result<()> {
    let md = render_fsm_diagram_markdown::<W>(transitions);
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("doc")
        .join(filename);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, md)
}
