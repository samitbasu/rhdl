use crate::{
    types::path::{bit_range, leaf_paths, Path},
    Circuit, CircuitIO, Kind, RHDLError, Timed,
};

use super::ast::{
    component_instance, connection, declaration, id, port, unsigned_width, Direction, HDLKind,
    Module, Statement,
};

fn build_coverage_error(kind: Kind, coverage: &[bool]) -> String {
    let paths = leaf_paths(&kind, Path::default());
    let mut details = String::new();
    for path in paths {
        let (bits, _) = bit_range(kind, &path).unwrap();
        let covered = coverage[bits].iter().all(|b| *b);
        if !covered {
            details.push_str(&format!("Path {:?} is not covered\n", path));
        }
    }
    details
}

pub fn export_hdl_module<'a, T>(
    uut: &T,
    name: &str,
    description: &str,
    binds: impl IntoIterator<Item = (Direction, &'a str, Kind, Path)>,
) -> Result<Module, RHDLError>
where
    T: Circuit,
{
    let hdl = uut.hdl(&format!("{name}_inner"))?;
    let verilog = hdl.as_module();
    let binds = binds.into_iter().collect::<Vec<_>>();
    let ports = binds
        .iter()
        .map(|(dir, name, kind, path)| {
            let (range, _) = bit_range(*kind, path).unwrap();
            let width = unsigned_width(range.end - range.start);
            port(name, *dir, HDLKind::Wire, width)
        })
        .collect::<Vec<_>>();
    let i_kind = <<T as CircuitIO>::I as Timed>::static_kind();
    let o_kind = <<T as CircuitIO>::O as Timed>::static_kind();
    let i_kind_bits = i_kind.bits();
    let o_kind_bits = o_kind.bits();
    let mut i_cover = vec![false; i_kind_bits];
    let mut o_cover = vec![false; o_kind_bits];
    binds.iter().for_each(|(dir, _, kind, path)| {
        let (range, _) = bit_range(*kind, path).unwrap();
        match dir {
            Direction::Input => {
                for bit in range {
                    i_cover[bit] = true;
                }
            }
            Direction::Output => {
                for bit in range {
                    o_cover[bit] = true;
                }
            }
            Direction::Inout => todo!(),
        }
    });
    if i_cover.iter().any(|b| !b) {
        let coverage = build_coverage_error(i_kind, &i_cover);
        return Err(RHDLError::InputsNotCovered(coverage));
    }
    let declarations = vec![
        declaration(HDLKind::Wire, "i", unsigned_width(i_kind_bits), None),
        declaration(HDLKind::Wire, "o", unsigned_width(o_kind_bits), None),
    ];
    let mut statements = binds
        .iter()
        .map(|(dir, name, kind, path)| {
            let (range, _) = bit_range(*kind, path).unwrap();
            match dir {
                Direction::Input => Statement::Custom(format!(
                    "assign i[{}:{}] = {name};",
                    range.end.saturating_sub(1),
                    range.start
                )),
                Direction::Output => Statement::Custom(format!(
                    "assign {name} = o[{}:{}];",
                    range.end.saturating_sub(1),
                    range.start
                )),
                Direction::Inout => todo!(),
            }
        })
        .collect::<Vec<_>>();
    statements.push(component_instance(
        &verilog.name,
        "sub",
        vec![connection("i", id("i")), connection("o", id("o"))],
    ));
    Ok(Module {
        name: name.into(),
        description: description.into(),
        ports,
        declarations,
        statements,
        submodules: vec![verilog],
        ..Default::default()
    })
}
