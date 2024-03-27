use crate::{
    crusty::{index::IndexedSchematic, upstream::follow_pin_upstream, utils::labels_for_trace},
    path::Path,
    schematic::{
        components::ComponentKind,
        schematic_impl::{pin_path, PinIx},
    },
};
use anyhow::Result;
use miette::{LabeledSpan, Report};

pub fn check_dff_is_clocked(is: &IndexedSchematic, clock_pin: PinIx) -> Result<Option<Report>> {
    eprintln!("dff clock pin: {:?}", clock_pin);
    let pin = is.schematic.pin(clock_pin);
    let component = is.schematic.component(pin.parent);
    eprintln!("Owning component: {:?}", component);
    let trace = follow_pin_upstream(is, pin_path(clock_pin, Path::default()))?;
    let Some(invalid_source) = trace
        .sinks
        .iter()
        .find(|sink| !is.schematic.inputs.contains(&sink.pin))
    else {
        return Ok(None);
    };
    let labels = labels_for_trace(is, &trace);
    let diagnostic = miette::MietteDiagnostic::new("Clock error")
        .with_help("Check that the clock is connected to a clock source")
        .with_code("clock_error")
        .with_severity(miette::Severity::Warning)
        .with_labels(labels);
    Ok(Some(
        miette::Report::new(diagnostic).with_source_code(is.pool.clone()),
    ))
}

pub fn check_dffs_are_clocked(is: &IndexedSchematic) -> Result<Vec<Report>> {
    is.schematic
        .components
        .iter()
        .filter_map(|c| match &c.kind {
            ComponentKind::DigitalFlipFlop(d) => Some(d.clock),
            _ => None,
        })
        .filter_map(|clock_pin| check_dff_is_clocked(is, clock_pin).transpose())
        .collect()
}
/*
let dff_source_path = rhdl_core::crusty::upstream::follow_pin_upstream(
&schematic.clone().into(),
PinPath {
    pin: dff_clock_pin,
    path: Path::default(),
},
)
.unwrap();
let report = trace_diagnostic(&schematic.clone().into(), &dff_source_path);
eprintln!("report is {:?}", report);
for segment in &dff_source_path.paths {
eprintln!("segment is {:?}", segment);
}
let mut dot = std::fs::File::create("strobe_inlined.dot").unwrap();
rhdl_core::schematic::dot::write_dot(&schematic, Some(&dff_source_path), &mut dot).unwrap();
}
*/
// Notes to think about:
// 1. Make schematic inlined always.
// 2. Generate trace items using the operators as well.
// So something like this:
//   source -> pin -> "from here"
//              -> op "via this op"
//   dest -> pin -> "to here"
