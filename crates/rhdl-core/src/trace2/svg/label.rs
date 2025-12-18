use crate::{BitX, Kind, TypedBits, trace2::svg::waveform::Waveform, types::path::Path};

#[derive(Debug, Clone)]
struct IndentedLabel {
    text: String,
    indent: usize,
    full_text: String,
}

impl IndentedLabel {
    fn compute_label(&self) -> String {
        (0..self.indent.saturating_sub(1))
            .map(|_| "   ")
            .chain(std::iter::once(self.text.as_str()))
            .collect()
    }
}

fn format_as_label_inner(t: &TypedBits) -> Option<String> {
    match t.kind() {
        Kind::Array(inner) => {
            let vals = (0..inner.size)
                .flat_map(|i| t.path(&Path::default().index(i)).ok())
                .flat_map(|element| format_as_label_inner(&element))
                .collect::<Vec<_>>()
                .join(", ");
            Some(format!("[{vals}]"))
        }
        Kind::Tuple(inner) => {
            let vals = inner
                .elements
                .iter()
                .enumerate()
                .flat_map(|(i, _)| t.path(&Path::default().tuple_index(i)).ok())
                .flat_map(|element| format_as_label_inner(&element))
                .collect::<Vec<_>>()
                .join(", ");
            Some(format!("({vals})"))
        }
        Kind::Struct(inner) => {
            let vals = inner
                .fields
                .iter()
                .flat_map(|field| {
                    t.path(&Path::default().field(&field.name))
                        .map(|x| (field, x))
                        .ok()
                })
                .flat_map(|(name, field)| format_as_label_inner(&field).map(|x| (name, x)))
                .map(|(name, val)| format!("{}: {}", name.name, val))
                .collect::<Vec<_>>()
                .join(", ");
            Some(format!("{{{vals}}}"))
        }
        Kind::Enum(inner) => {
            let discriminant = t.discriminant().ok()?.as_i64().ok()?;
            let variant = inner
                .variants
                .iter()
                .find(|v| v.discriminant == discriminant)?;
            let payload = t
                .path(&Path::default().payload_by_value(discriminant))
                .ok()?;
            let payload = format_as_label_inner(&payload).unwrap_or_default();
            Some(format!("{}{}", variant.name, payload))
        }
        Kind::Bits(inner) => {
            let mut val: u128 = 0;
            for ndx in 0..inner {
                if t.bits()[ndx] == BitX::One {
                    // TODO - handle other BitX values
                    val |= 1 << ndx;
                }
            }
            let num_nibbles = inner / 4 + if inner % 4 == 0 { 0 } else { 1 };
            // Format the val as a hex number with the given
            // number of nibbles, with left padding of zeros
            Some(format!("{val:0num_nibbles$x}"))
        }
        Kind::Signed(inner) => {
            let mut val: i128 = 0;
            for ndx in 0..inner {
                if t.bits()[ndx] == BitX::One {
                    // TODO - handle other BitX values
                    val |= 1 << ndx;
                }
            }
            if val & (1 << (inner - 1)) != 0 {
                val |= !0 << inner;
            }
            Some(format!("{val}"))
        }
        Kind::Signal(_inner, color) => {
            let val = &t.val();
            let val = format_as_label_inner(val)?;
            Some(format!("{color:?}@({val})"))
        }
        Kind::Empty => None,
    }
}

pub(crate) fn format_as_label(t: &TypedBits) -> Option<String> {
    (t.len() != 1).then(|| format_as_label_inner(t))?
}

// Compute a map indicating which time series in the list
// is a "parent" of this one.  The parent is defined as
// a time series with a path that is a prefix of the current
// path.  We search backwards to find the closest anscestor.
fn build_parent_map(labels: &[&str]) -> Box<[usize]> {
    // Every node starts out parented to the root
    let mut parents = vec![0; labels.len()];
    // Work down the list
    for (ndx, label) in labels.iter().enumerate() {
        for i in 0..ndx {
            let break_char = labels[ndx - 1 - i].len();
            if label.starts_with(labels[ndx - 1 - i])
                && let Some(char) = label.chars().nth(break_char)
                && ['.', '#', '['].contains(&char)
            {
                parents[ndx] = ndx - 1 - i;
                break;
            }
        }
    }
    parents.into()
}

// Given a list of parents, compute the indentation for each
// label.  The indentation is defined as the number of
// ancestors of the label.  Because we scan the list forward,
// we can keep track of the indentation level for a parent
// and increment it for each child.
fn compute_indentation(parents: &[usize]) -> Box<[usize]> {
    let mut indentation = vec![0; parents.len()];
    for ndx in 1..parents.len() {
        indentation[ndx] = indentation[parents[ndx]] + 1;
    }
    // Fix up the first entry
    if !indentation.is_empty() {
        indentation[0] = 1;
    }
    indentation.into()
}

fn tree_view(root: &str, labels: &[&str]) -> Box<[IndentedLabel]> {
    let parent_map = build_parent_map(labels);
    let indentation = compute_indentation(&parent_map);
    labels
        .iter()
        .enumerate()
        .map(|(ndx, label)| {
            let mut text = label.to_string();
            let parent_text = if parent_map[ndx] == 0 {
                root
            } else {
                labels[parent_map[ndx]]
            };
            if text.starts_with(parent_text) {
                text = text.replacen(parent_text, "", 1);
            }
            IndentedLabel {
                text,
                indent: indentation[ndx],
                full_text: label.to_string(),
            }
        })
        .collect()
}

pub(crate) fn rewrite_trace_names_into_tree(traces: &mut [Waveform]) {
    let labels = traces.iter().map(|t| t.label.as_str()).collect::<Vec<_>>();
    let tree: Vec<IndentedLabel> = tree_view("top", &labels).into();
    traces.iter_mut().zip(tree).for_each(|(trace, label)| {
        trace.label = label.compute_label();
        trace.hint = label.full_text;
    });
}
