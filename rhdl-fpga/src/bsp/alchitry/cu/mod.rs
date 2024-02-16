use std::{
    fs::{create_dir_all, remove_dir_all},
    path::Path,
    process::Command,
};

use crate::{bga_pin, BGAPin, BGARow, ConstrainedVerilog, Constraint};
use anyhow::Result;

pub const LED_ARRAY_LOCATIONS: [BGAPin; 8] = [
    bga_pin(BGARow::J, 11),
    bga_pin(BGARow::K, 11),
    bga_pin(BGARow::K, 12),
    bga_pin(BGARow::K, 14),
    bga_pin(BGARow::L, 12),
    bga_pin(BGARow::L, 14),
    bga_pin(BGARow::M, 12),
    bga_pin(BGARow::N, 14),
];

pub const BASE_CLOCK_100MHZ_LOCATION: BGAPin = bga_pin(BGARow::P, 7);

pub fn synth_yosys_nextpnr_icepack(v: &ConstrainedVerilog, path: &Path) -> Result<()> {
    let _ = remove_dir_all(path);
    create_dir_all(path)?;
    std::fs::write(path.join("top.v"), &v.module)?;
    std::fs::write(path.join("top.pcf"), v.pcf()?)?;
    let command_arg = "-p synth_ice40 -top top -json top.json";
    let yosys = Command::new("yosys")
        .current_dir(path)
        .arg(command_arg)
        .arg("top.v")
        .output()
        .expect("Yosys should be installed and in your PATH.");

    std::fs::write(path.join("yosys.stdout"), &yosys.stdout)?;
    std::fs::write(path.join("yosys.stderr"), &yosys.stderr)?;

    if !yosys.status.success() {
        anyhow::bail!(
            "yosys failed with status {:?}:\n{}",
            yosys.status,
            String::from_utf8_lossy(&yosys.stderr)
        );
    }

    eprintln!("{}", String::from_utf8_lossy(&yosys.stdout));
    let mut nextpnr = Command::new("nextpnr-ice40");
    nextpnr
        .current_dir(path)
        .arg("--hx8k")
        .arg("--package")
        .arg("cb132")
        .arg("--json")
        .arg("top.json")
        .arg("--pcf")
        .arg("top.pcf")
        .arg("--asc")
        .arg("top.asc");
    if v.constraints
        .iter()
        .any(|x| matches!(x.constraint, Constraint::Unused))
    {
        nextpnr.arg("--pcf-allow-unconstrained");
    }
    let nextpnr = nextpnr.output()?;
    std::fs::write(path.join("nextpnr.stdout"), &nextpnr.stdout)?;
    std::fs::write(path.join("nextpnr.stderr"), &nextpnr.stderr)?;
    if !nextpnr.status.success() {
        anyhow::bail!(
            "nextpnr-ice40 failed with status {:?}:\n{}",
            nextpnr.status,
            String::from_utf8_lossy(&nextpnr.stderr)
        );
    }
    eprintln!("{}", String::from_utf8_lossy(&nextpnr.stdout));

    let icepack = Command::new("icepack")
        .current_dir(path)
        .arg("top.asc")
        .arg("top.bin")
        .output()?;
    std::fs::write(path.join("icepack.stdout"), &icepack.stdout)?;
    std::fs::write(path.join("icepack.stderr"), &icepack.stderr)?;
    if !icepack.status.success() {
        anyhow::bail!(
            "icepack failed with status {:?}:\n{}",
            icepack.status,
            String::from_utf8_lossy(&icepack.stderr)
        );
    }
    eprintln!("{}", String::from_utf8_lossy(&icepack.stdout));
    Ok(())
}
