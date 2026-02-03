use crate::{
    Color, Kind,
    types::path::{Path, sub_kind},
};

#[derive(Clone, Debug, Copy, PartialEq)]
pub(crate) enum TraceColor {
    Single(Color),
    MultiColor,
}

impl Default for TraceColor {
    fn default() -> Self {
        TraceColor::Single(Color::Green)
    }
}

impl TraceColor {
    fn merge(self, other: TraceColor) -> TraceColor {
        match (self, other) {
            (TraceColor::Single(s), TraceColor::Single(o)) if s == o => TraceColor::Single(s),
            _ => TraceColor::MultiColor,
        }
    }
}

fn color_merged(list: impl Iterator<Item = TraceColor>) -> Option<TraceColor> {
    let mut ret = None;
    for color in list {
        ret = match (ret, color) {
            (None, x) => Some(x),
            (Some(c), d) => Some(c.merge(d)),
        }
    }
    ret
}

fn compute_trace_color(kind: Kind) -> Option<TraceColor> {
    match kind {
        Kind::Signal(_, color) => Some(TraceColor::Single(color)),
        Kind::Bits(_)
        | Kind::Signed(_)
        | Kind::Empty
        | Kind::Enum(_)
        | Kind::Clock
        | Kind::Reset => None,
        Kind::Struct(inner) => color_merged(
            inner
                .fields
                .iter()
                .flat_map(|x| compute_trace_color(x.kind)),
        ),
        Kind::Tuple(inner) => {
            color_merged(inner.elements.iter().flat_map(|x| compute_trace_color(*x)))
        }
        Kind::Array(inner) => compute_trace_color(*inner.base),
    }
}

// Compute the color of a path applied to a TypedBits.  It may be None if
// no colors are encountered.  Otherwise, it is the color "closest" to the
// path in question (closest ancestor).
pub(crate) fn compute_trace_color_from_path(t: Kind, path: &Path) -> Option<TraceColor> {
    let mut my_color = None;
    let mut path = path.clone();
    while my_color.is_none() {
        let my_kind = sub_kind(t, &path).ok()?;
        my_color = compute_trace_color(my_kind);
        if path.is_empty() {
            break;
        }
        path.pop();
    }
    my_color
}
