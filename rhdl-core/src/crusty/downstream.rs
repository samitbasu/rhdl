use crate::path::{bit_range, sub_kind, Path};
use crate::schematic::components::{
    BinaryComponent, BufferComponent, CaseComponent, CastComponent, ComponentKind,
    DigitalFlipFlopComponent, EnumComponent, IndexComponent, RepeatComponent, SelectComponent,
    SpliceComponent, StructComponent, TupleComponent, UnaryComponent,
};
use crate::schematic::dot::write_dot;
use crate::schematic::schematic_impl::{PinPath, Trace, WirePath};
use crate::schematic::{components::ArrayComponent, schematic_impl::Schematic};
use anyhow::{anyhow, ensure};
use anyhow::{bail, Result};

use super::index::IndexedSchematic;

fn downstream_array(array: &ArrayComponent, input: PinPath) -> Result<Vec<PinPath>> {
    let (ndx, ix) = array
        .elements
        .iter()
        .enumerate()
        .find(|(ndx, &ix)| ix == input.pin)
        .ok_or(anyhow!("ICE - input pin not found in array"))?;
    Ok(vec![PinPath {
        pin: array.output,
        path: Path::default().index(ndx).join(&input.path),
    }])
}

fn downstream_binary(array: &BinaryComponent, input: PinPath) -> Result<Vec<PinPath>> {
    Ok(vec![PinPath {
        pin: array.output,
        path: input.path.clone(),
    }])
}

fn downstream_buffer(buffer: &BufferComponent, input: PinPath) -> Result<Vec<PinPath>> {
    Ok(vec![PinPath {
        pin: buffer.output,
        path: input.path,
    }])
}

fn downstream_case(case: &CaseComponent, input: PinPath) -> Result<Vec<PinPath>> {
    Ok(if case.table.iter().any(|(_, ix)| *ix == input.pin) {
        vec![PinPath {
            pin: case.output,
            path: input.path.clone(),
        }]
    } else {
        vec![]
    })
}

fn downstream_dff(dff: &DigitalFlipFlopComponent, input: PinPath) -> Result<Vec<PinPath>> {
    Ok(vec![])
}

// An EnumComponent is fed an enum template value.
// So if we want the discriminant, then this block is a source, and the
// search ends.
//
// On the other hand, if we want the payload, then we first remove the
// payload part of the path.  If at that point, the path is empty, we have a
// problem, since we do not know which of the inputs to use.  On the other
// hand if the path

fn downstream_enum(e: &EnumComponent, input: PinPath) -> Result<Vec<PinPath>> {
    let field = e
        .fields
        .iter()
        .find(|field| field.pin == input.pin)
        .ok_or(anyhow!("ICE - input pin not found in enum component"))?;
    Ok(vec![PinPath {
        pin: e.output,
        path: Path::default()
            .payload_by_value(e.template.discriminant()?.as_i64()?)
            .field(&field.member.to_string()),
    }])
}

fn downstream_index(i: &IndexComponent, input: PinPath) -> Result<Vec<PinPath>> {
    if i.path.is_prefix_of(&input.path) {
        let path = input.path.strip_prefix(&i.path)?;
        Ok(vec![PinPath {
            pin: i.output,
            path,
        }])
    } else {
        Ok(vec![])
    }
}

fn downstream_repeat(r: &RepeatComponent, input: PinPath) -> Result<Vec<PinPath>> {
    Ok((0..r.len)
        .map(|ndx| PinPath {
            pin: r.output,
            path: Path::default().index(ndx).join(&input.path),
        })
        .collect())
}

fn downstream_select(s: &SelectComponent, input: PinPath) -> Result<Vec<PinPath>> {
    if s.cond != input.pin {
        Ok(vec![PinPath {
            pin: s.output,
            path: input.path,
        }])
    } else {
        Ok(vec![])
    }
}

fn downstream_splice(s: &SpliceComponent, input: PinPath) -> Result<Vec<PinPath>> {
    if input.pin != s.orig && input.pin != s.subst {
        return Ok(vec![]);
    }
    let (input_bit_range, _) = bit_range(s.kind.clone(), &input.path)?;
    ensure!(
        !input.path.any_dynamic(),
        "Unsupported - dynamic path in splice"
    );
    ensure!(
        !s.path.any_dynamic(),
        "Unsupported - dynamic path in splice"
    );
    let (replace_bit_range, _) = bit_range(s.kind.clone(), &s.path)?;
    let input_path_in_replacement = replace_bit_range.contains(&input_bit_range.start);
    if input_path_in_replacement && input.pin == s.subst {
        Ok(vec![PinPath {
            pin: s.output,
            path: s.path.clone().join(&input.path),
        }])
    } else if !input_path_in_replacement && input.pin == s.orig {
        Ok(vec![PinPath {
            pin: s.output,
            path: input.path,
        }])
    } else {
        Ok(vec![])
    }
}

fn downstream_struct(s: &StructComponent, input: PinPath) -> Result<Vec<PinPath>> {
    if let Some(field) = s.fields.iter().find(|field| field.pin == input.pin) {
        Ok(vec![PinPath {
            pin: s.output,
            path: Path::default()
                .field(&field.member.to_string())
                .join(&input.path),
        }])
    } else if let Some(rest) = &s.rest {
        // Check if our value is replaced
        let (input_bit_range, _) = bit_range(s.kind.clone(), &input.path)?;
        if s.fields.iter().any(|f| {
            let field_path = Path::default().field(&f.member.to_string());
            let (field_bit_range, _) = bit_range(s.kind.clone(), &field_path).unwrap();
            field_bit_range.contains(&input_bit_range.start)
        }) {
            return Ok(vec![]);
        }
        Ok(vec![PinPath {
            pin: s.output,
            path: input.path.clone(),
        }])
    } else {
        Ok(vec![])
    }
}

fn downstream_tuple(t: &TupleComponent, input: PinPath) -> Result<Vec<PinPath>> {
    let (ndx, _) = t
        .fields
        .iter()
        .enumerate()
        .find(|(_, &ix)| ix == input.pin)
        .ok_or(anyhow!("ICE - input pin not found in tuple component"))?;
    Ok(vec![PinPath {
        pin: t.output,
        path: Path::default().index(ndx).join(&input.path),
    }])
}

fn downstream_cast(c: &CastComponent, input: PinPath) -> Result<Vec<PinPath>> {
    Ok(vec![PinPath {
        pin: c.output,
        path: input.path,
    }])
}

fn downstream_unary(u: &UnaryComponent, input: PinPath) -> Result<Vec<PinPath>> {
    Ok(vec![PinPath {
        pin: u.output,
        path: input.path,
    }])
}

// Given a schematic, and a path on an input pin of some component, we need
// to return a list of output pins and paths that are affected by this input
// pin/path combo.  If the input is a sink, then we will return an empty set
fn get_downstream_pin_paths(is: &IndexedSchematic, input: PinPath) -> Result<Vec<PinPath>> {
    let pin = is.schematic.pin(input.pin);
    let cix = pin.parent;
    let component = is.schematic.component(cix);
    match &component.kind {
        ComponentKind::Array(array) => downstream_array(array, input),
        ComponentKind::Binary(binary) => downstream_binary(binary, input),
        ComponentKind::BlackBox(_) => Ok(vec![]),
        ComponentKind::Buffer(buffer) => downstream_buffer(buffer, input),
        ComponentKind::Case(case) => downstream_case(case, input),
        ComponentKind::Cast(c) => downstream_cast(c, input),
        ComponentKind::DigitalFlipFlop(dff) => downstream_dff(dff, input),
        ComponentKind::Enum(e) => downstream_enum(e, input),
        ComponentKind::Index(i) => downstream_index(i, input),
        ComponentKind::Kernel(_) => Ok(vec![]),
        ComponentKind::Noop => Ok(vec![]),
        ComponentKind::Repeat(r) => downstream_repeat(r, input),
        ComponentKind::Select(s) => downstream_select(s, input),
        ComponentKind::Splice(s) => downstream_splice(s, input),
        ComponentKind::Struct(s) => downstream_struct(s, input),
        ComponentKind::Tuple(t) => downstream_tuple(t, input),
        ComponentKind::Unary(u) => downstream_unary(u, input),
        ComponentKind::Constant(_) => Ok(vec![]),
    }
}

fn follow_downstream(is: &IndexedSchematic, source: PinPath, trace: &mut Trace) -> Result<()> {
    if is.schematic.output == source.pin {
        trace.sinks.push(source);
        return Ok(());
    }
    if let Some(children) = is.index.forward.get(&source.pin) {
        for child in children {
            trace.paths.push(WirePath {
                source: source.pin,
                dest: *child,
                path: source.path.clone(),
            });
            let child_pin_path = PinPath {
                pin: *child,
                path: source.path.clone(),
            };
            let downstreams = get_downstream_pin_paths(is, child_pin_path.clone())?;
            if downstreams.is_empty() {
                trace.sinks.push(child_pin_path);
            } else {
                for downstream in downstreams {
                    follow_downstream(is, downstream, trace)?;
                }
            }
        }
    } else {
        let downstreams = get_downstream_pin_paths(is, source.clone())?;
        if downstreams.is_empty() {
            trace.sinks.push(source);
            return Ok(());
        } else {
            for downstream in downstreams {
                follow_downstream(is, downstream, trace)?;
            }
        }
    }
    Ok(())
}

pub fn follow_pin_downstream(is: &IndexedSchematic, pin_path: PinPath) -> Result<Trace> {
    let pin_kind = is.schematic.pin(pin_path.pin).kind.clone();
    if let Err(err) = sub_kind(pin_kind.clone(), &pin_path.path) {
        bail!("Illegal path in query.  The specified path {} is not valid on the type of the given pin, which is {}. Error was {err}",
        pin_path.path, pin_kind);
    }
    let mut w = vec![];
    write_dot(&is.schematic, Default::default(), &mut w)?;
    eprintln!("{}", String::from_utf8_lossy(&w));
    let mut trace = pin_path.clone().into();
    follow_downstream(is, pin_path, &mut trace)?;
    Ok(trace)
}
