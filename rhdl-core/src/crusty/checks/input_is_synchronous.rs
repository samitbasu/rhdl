use itertools::Itertools;

use crate::crusty::{index::Timing, upstream::follow_pin_upstream};

use super::pin_is_clocked::get_clock_id_for_pin;

pub(crate) fn check_input_is_synchronous(
    is: &mut crate::crusty::index::IndexedSchematic,
    c: &crate::schematic::constraints::InputSynchronousConstraint,
) -> Result<(), miette::Report> {
    let clock_id = get_clock_id_for_pin(is, &c.clock)?;
    let trace = follow_pin_upstream(is, c.input.clone())
        .map_err(|err| miette::Report::new(miette::MietteDiagnostic::new(err.to_string())))?;
    if trace
        .all_pin_paths()
        .try_for_each(|pin_path| is.set_timing(pin_path, Timing::Synchronous(clock_id)))
        .is_ok()
    {
        return Ok(());
    }
    let labels = crate::crusty::utils::labels_for_trace(is, &trace);
    let diagnostic = miette::MietteDiagnostic::new("Input must be synchronous with a single clock")
        .with_help("Input must be synchronous with a single clock.")
        .with_severity(miette::Severity::Error)
        .with_labels(labels);
    Err(miette::Report::new(diagnostic).with_source_code(is.pool.clone()))
}
