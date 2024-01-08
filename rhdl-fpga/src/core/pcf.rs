use anyhow::anyhow;
use anyhow::Result;

use crate::{ConstrainedVerilog, Constraint, PinConstraintKind};

pub fn make_pcf_from_constrained_verilog(module: &ConstrainedVerilog) -> Result<String> {
    let pcf_lines = module
        .constraints
        .iter()
        .filter_map(|x| match &x.kind {
            PinConstraintKind::Input => pcf_pin_name("top_in", x.index, &x.constraint).transpose(),
            PinConstraintKind::Output => {
                pcf_pin_name("top_out", x.index, &x.constraint).transpose()
            }
            PinConstraintKind::Clock(name) => {
                if let Constraint::Location(pin) = &x.constraint {
                    Ok(Some(format!("set_io {name} {pin}"))).transpose()
                } else {
                    Some(Err(anyhow!("Clock constraint must have a location")))
                }
            }
        })
        .collect::<Result<Vec<_>>>()?
        .join("\n");
    // Temporary hack
    let pcf_lines = format!("set_frequency clk 50\n{pcf_lines}\n");
    Ok(pcf_lines)
}

fn pcf_pin_name(name: &str, index: usize, c: &Constraint) -> Result<Option<String>> {
    match c {
        Constraint::Location(pin) => Ok(Some(format!("set_io {name}[{index}] {pin}"))),
        Constraint::Custom(text) => Ok(Some(format!("set_io {name}[{index}] {text}"))),
        Constraint::Unused => Ok(None),
        _ => anyhow::bail!("Cannot convert constraint to PCF pin name: {:?}", c),
    }
}

impl ConstrainedVerilog {
    pub fn pcf(&self) -> Result<String> {
        make_pcf_from_constrained_verilog(self)
    }
}
