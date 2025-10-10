//! Functions for working with nextpnr-ice40.

pub struct NextpnrIce40Output {
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug)]
pub struct TimingInfo {
    pub routing_delay: f64,
    pub logic_delay: f64,
}

impl NextpnrIce40Output {
    pub fn extract_timing(&self) -> miette::Result<TimingInfo> {
        // Look for a line of the form: "Info: <number> ns logic, <number> ns routing"
        // and collect the two numbers
        let mut logic_delay = None;
        let mut routing_delay = None;
        for line in self.stderr.lines() {
            if line.contains("ns logic") && line.contains("ns routing") {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() == 2 {
                    let logic_part = parts[0];
                    let routing_part = parts[1];
                    if let Some(logic_str) = logic_part.split_whitespace().nth(1) {
                        if let Ok(val) = logic_str.parse::<f64>() {
                            logic_delay = Some(val);
                        }
                    }
                    if let Some(routing_str) = routing_part.split_whitespace().nth(0) {
                        if let Ok(val) = routing_str.parse::<f64>() {
                            routing_delay = Some(val);
                        }
                    }
                }
            }
        }
        let logic_delay = logic_delay.ok_or_else(|| {
            miette::miette!(
                "Could not find logic delay in nextpnr-ice40 output:\n{stderr}",
                stderr = self.stderr
            )
        })?;
        let routing_delay = routing_delay.ok_or_else(|| {
            miette::miette!(
                "Could not find routing delay in nextpnr-ice40 output:\n{stderr}",
                stderr = self.stderr
            )
        })?;
        Ok(TimingInfo {
            routing_delay,
            logic_delay,
        })
    }
}

pub fn place_and_route(
    part: &str,
    package: &str,
    constraints: Option<&str>,
    path: impl AsRef<camino::Utf8Path>,
) -> miette::Result<NextpnrIce40Output> {
    let path = path.as_ref();
    std::fs::create_dir_all(path).map_err(|e| miette::miette!(e.to_string()))?;
    let mut cmd = std::process::Command::new("nextpnr-ice40");
    cmd.current_dir(&path);
    cmd.arg(format!("--{part}"));
    cmd.arg("--package").arg(package);
    if let Some(constraints) = constraints {
        std::fs::write(path.join("rhdl.pcf"), constraints)
            .map_err(|e| miette::miette!(e.to_string()))?;
        cmd.arg("--pcf").arg("rhdl.pcf");
    }
    cmd.arg("--asc").arg("rhdl.asc");
    cmd.arg("--json").arg("rhdl.json");
    let output = cmd.output().map_err(|e| miette::miette!(e.to_string()))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        return Err(miette::miette!(format!(
            "nextpnr-ice40 place and route failed.\nSTDOUT:\n{stdout}\nSTDERR:\n{stderr}"
        )));
    }
    Ok(NextpnrIce40Output {
        stdout: stdout.to_string(),
        stderr: stderr.to_string(),
    })
}
