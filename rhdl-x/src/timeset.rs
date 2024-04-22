// An experiment in timing propagation and solution.

// First, we need to generate all possible leaf paths for a given kind.

use anyhow::{bail, Result};
use fnv::FnvHashMap;
use std::iter::once;

use rhdl_core::{
    path::Path,
    schematic::{
        components::{
            BinaryComponent, BufferComponent, CaseComponent, CastComponent, TupleComponent,
            UnaryComponent,
        },
        schematic_impl::{pin_path, PinPath, Schematic},
    },
    Kind,
};

// It seems like this should be doable with an impl Iterator type, but :shrug:
pub fn leaf_paths(kind: &Kind, base: Path) -> Vec<Path> {
    match kind {
        Kind::Array(array) => (0..array.size)
            .flat_map(|i| leaf_paths(&array.base, base.clone().index(i)))
            .collect(),
        Kind::Tuple(tuple) => tuple
            .elements
            .iter()
            .enumerate()
            .flat_map(|(i, k)| leaf_paths(k, base.clone().index(i)))
            .collect(),
        Kind::Struct(structure) => structure
            .fields
            .iter()
            .flat_map(|field| leaf_paths(&field.kind, base.clone().field(&field.name)))
            .collect(),
        Kind::Enum(enumeration) => enumeration
            .variants
            .iter()
            .flat_map(|variant| {
                leaf_paths(
                    &variant.kind,
                    base.clone().payload_by_value(variant.discriminant),
                )
            })
            .chain(once(base.clone().discriminant()))
            .collect(),
        Kind::Signal(root, _) => leaf_paths(root, base.clone())
            .into_iter()
            .map(|p| p.signal_value())
            .collect(),
        Kind::Bits(_) | Kind::Signed(_) | Kind::Empty => vec![base.clone()],
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct ClockId(usize);

impl From<usize> for ClockId {
    fn from(id: usize) -> Self {
        Self(id)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Timing {
    Const,
    Async,
    Sync(ClockId),
}

pub fn merge(t1: Timing, t2: Timing) -> Timing {
    match (t1, t2) {
        (Timing::Async, _) | (_, Timing::Async) => Timing::Async,
        (Timing::Const, other) | (other, Timing::Const) => other,
        (Timing::Sync(c1), Timing::Sync(c2)) if c1 == c2 => Timing::Sync(c1),
        _ => Timing::Async,
    }
}

#[derive(Default, Debug, Clone)]
struct TimingDB {
    db: FnvHashMap<PinPath, Timing>,
}

impl TimingDB {
    pub fn set_timing(&mut self, pin_path: &PinPath, timing: Timing) -> Result<()> {
        if let Some(t_old) = self.db.insert(pin_path.clone(), timing) {
            if t_old != timing {
                bail!("Timing conflict");
            }
        }
        Ok(())
    }
    pub fn merge_timing(&mut self, pin1: &PinPath, output: &PinPath) -> Result<()> {
        if let Some(&t1) = self.db.get(pin1) {
            if let Some(&t2) = self.db.get(output) {
                self.set_timing(output, merge(t1, t2))?;
            } else {
                self.set_timing(output, t1)?;
            }
        }
        Ok(())
    }
    pub fn get(&mut self, pin_path: &PinPath) -> Option<&Timing> {
        self.db.get(pin_path)
    }
}

fn time_binary(schematic: &Schematic, binary: &BinaryComponent, db: &mut TimingDB) -> Result<()> {
    let kind = &schematic.pin(binary.output).kind;
    for p in leaf_paths(kind, Path::default()) {
        db.merge_timing(
            &pin_path(binary.input1, p.clone()),
            &pin_path(binary.output, p.clone()),
        )?;
        db.merge_timing(
            &pin_path(binary.input2, p.clone()),
            &pin_path(binary.output, p.clone()),
        )?;
    }
    Ok(())
}

fn time_unary(schematic: &Schematic, unary: &UnaryComponent, db: &mut TimingDB) -> Result<()> {
    let kind = &schematic.pin(unary.input).kind;
    for p in leaf_paths(kind, Path::default()) {
        if let Some(&timing) = db.get(&pin_path(unary.input, p.clone())) {
            db.set_timing(&pin_path(unary.output, p.clone()), timing)?;
        }
    }
    Ok(())
}

fn time_cast(cast: &CastComponent, db: &mut TimingDB) -> Result<()> {
    db.merge_timing(
        &pin_path(cast.input, Path::default()),
        &pin_path(cast.output, Path::default()),
    )
}

fn time_buffer(schematic: &Schematic, buffer: &BufferComponent, db: &mut TimingDB) -> Result<()> {
    let kind = &schematic.pin(buffer.input).kind;
    for p in leaf_paths(kind, Path::default()) {
        db.merge_timing(
            &pin_path(buffer.input, p.clone()),
            &pin_path(buffer.output, p.clone()),
        )?;
    }
    Ok(())
}

fn time_tuple(schematic: &Schematic, tuple: &TupleComponent, db: &mut TimingDB) -> Result<()> {
    for (i, &input) in tuple.fields.iter().enumerate() {
        let kind = &schematic.pin(input).kind;
        for p in leaf_paths(kind, Path::default()) {
            db.merge_timing(
                &pin_path(input, p.clone()),
                &pin_path(tuple.output, Path::default().index(i).join(&p)),
            )?;
        }
    }
    Ok(())
}

fn time_case(schematic: &Schematic, case: &CaseComponent, db: &mut TimingDB) -> Result<()> {
    let discriminant_kind = &schematic.pin(case.discriminant).kind;
    let output_kind = &schematic.pin(case.output).kind;
    for p in leaf_paths(discriminant_kind, Path::default()) {
        for q in leaf_paths(output_kind, Path::default()) {
            db.merge_timing(
                &pin_path(case.discriminant, p.clone()),
                &pin_path(case.output, q.clone()),
            )?;
        }
    }
    for (_, arg_pin) in &case.table {
        let arg_kind = &schematic.pin(*arg_pin).kind;
        for p in leaf_paths(arg_kind, Path::default()) {
            db.merge_timing(
                &pin_path(*arg_pin, p.clone()),
                &pin_path(case.output, p.clone()),
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use rhdl_bits::alias::*;
    use rhdl_core::{
        path::{bit_range, Path},
        Digital,
    };
    use rhdl_macro::Digital;

    use crate::timeset::{leaf_paths, merge, Timing};

    #[test]
    fn test_leaf_paths_for_kind() {
        #[derive(Copy, Clone, PartialEq, Digital, Default)]
        pub struct Foo {
            a: bool,
            b: (b4, b8),
            c: [b4; 3],
        }

        #[derive(Copy, Clone, PartialEq, Digital, Default)]
        pub struct Bar {
            a: [(Foo, Foo); 2],
            b: b4,
        }

        #[derive(Copy, Clone, PartialEq, Digital, Default)]
        pub enum Baz {
            #[default]
            O,
            A(b4),
            B(Bar),
            C {
                x: b4,
                y: b4,
            },
        }

        let kind = Baz::static_kind();
        let paths = leaf_paths(&kind, Path::default());
        let mut bit_mask = vec![false; kind.bits()];
        for path in paths {
            eprintln!("{}", path);
            let (bits, _) = bit_range(kind.clone(), &path).unwrap();
            for i in bits {
                bit_mask[i] = true;
            }
        }
        assert!(bit_mask.iter().all(|&b| b));
    }

    #[test]
    fn test_merge() {
        assert_eq!(merge(Timing::Async, Timing::Async), Timing::Async);
        assert_eq!(merge(Timing::Const, Timing::Async), Timing::Async);
        assert_eq!(merge(Timing::Async, Timing::Const), Timing::Async);
        assert_eq!(merge(Timing::Const, Timing::Const), Timing::Const);
        assert_eq!(merge(Timing::Sync(0.into()), Timing::Async), Timing::Async);
        assert_eq!(merge(Timing::Async, Timing::Sync(0.into())), Timing::Async);
        assert_eq!(
            merge(Timing::Sync(0.into()), Timing::Const),
            Timing::Sync(0.into())
        );
        assert_eq!(
            merge(Timing::Const, Timing::Sync(0.into())),
            Timing::Sync(0.into())
        );
        assert_eq!(
            merge(Timing::Sync(0.into()), Timing::Sync(0.into())),
            Timing::Sync(0.into())
        );
        assert_eq!(
            merge(Timing::Sync(0.into()), Timing::Sync(1.into())),
            Timing::Async
        );
        assert_eq!(
            merge(Timing::Sync(1.into()), Timing::Sync(0.into())),
            Timing::Async
        );
    }
}
