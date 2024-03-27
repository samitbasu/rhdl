use std::{iter::once, ops::Range};

use itertools::Itertools;
use miette::LabeledSpan;

use crate::{
    path::Path,
    rhif::spec::Member,
    schematic::schematic_impl::{PinIx, PinPath, Trace, WirePath},
};

use super::index::IndexedSchematic;

pub(crate) fn path_with_member(path: Path, member: &Member) -> Path {
    match member {
        Member::Unnamed(ix) => path.index(*ix as usize),
        Member::Named(f) => path.field(f),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct LabeledSourceLocation {
    location: Range<usize>,
    label: String,
}

impl LabeledSourceLocation {
    fn span(self) -> LabeledSpan {
        LabeledSpan::new(
            Some(self.label),
            self.location.start,
            self.location.end - self.location.start,
        )
    }
}

fn location_for_pin_path(
    is: &IndexedSchematic,
    pin: PinIx,
    path: &Path,
    trace: &Trace,
) -> Option<LabeledSourceLocation> {
    eprintln!("pin: {:?}, path: {:?}", pin, path);
    let label = if trace.source.pin == pin || trace.sinks.iter().any(|s| s.pin == pin) {
        "trace terminates here".to_string()
    } else if path.is_empty() {
        "via signal".to_string()
    } else {
        format!("via {}", path)
    };
    is.schematic
        .pin(pin)
        .location
        .inspect(|l| eprintln!("location: {:?}", l))
        .and_then(|l| is.pool.get_range_from_location(l))
        .map(|l| LabeledSourceLocation { location: l, label })
}

fn locations_for_wire(
    is: &IndexedSchematic,
    p: &WirePath,
    t: &Trace,
) -> impl Iterator<Item = LabeledSourceLocation> {
    once(location_for_pin_path(is, p.source, &p.path, t))
        .chain(once(location_for_pin_path(is, p.dest, &p.path, t)))
        .flatten()
}

pub(crate) fn labels_for_trace(is: &IndexedSchematic, trace: &Trace) -> Vec<LabeledSpan> {
    trace
        .paths
        .iter()
        .rev()
        .inspect(|w| eprintln!("{:?}", w))
        .flat_map(|p| locations_for_wire(is, p, trace))
        .inspect(|x| eprintln!("{:?}", x))
        .unique()
        .map(|x| x.span())
        .collect()
}
