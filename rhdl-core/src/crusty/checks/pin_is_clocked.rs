use crate::{
    crusty::{
        index::{ClockId, IndexedSchematic},
        upstream::follow_pin_upstream,
        utils::labels_for_trace,
    },
    schematic::{constraints::MustClockConstraint, schematic_impl::PinPath},
};
use anyhow::Result;
use itertools::Itertools;
use miette::{MietteDiagnostic, Report};

pub fn check_pin_is_clocked(
    is: &IndexedSchematic,
    constraint: &MustClockConstraint,
) -> Result<(), Report> {
    get_clock_id_for_pin(is, &constraint.pin_path).map(|x| ())
}

pub fn get_clock_id_for_pin(is: &IndexedSchematic, pin_path: &PinPath) -> Result<ClockId, Report> {
    let trace = follow_pin_upstream(is, pin_path.clone())
        .map_err(|err| Report::new(MietteDiagnostic::new(err.to_string())))?;
    let clock_ids = trace
        .sinks
        .iter()
        .map(|sink| is.clock_id(sink))
        .all_equal_value();
    match clock_ids {
        Ok(Some(id)) => Ok(id),
        _ => {
            let labels = labels_for_trace(is, &trace);
            let diagnostic =
                miette::MietteDiagnostic::new("Pin must be clocked with a single clock")
                    .with_help("Pin must be clocked with a single clock.")
                    .with_severity(miette::Severity::Error)
                    .with_labels(labels);
            Err(Report::new(diagnostic).with_source_code(is.pool.clone()))
        }
    }
}
