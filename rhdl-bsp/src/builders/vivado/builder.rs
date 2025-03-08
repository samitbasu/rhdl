use super::tcl;
use std::io::Write;

pub struct Builder {
    project_name: String,
    part_name: String,
    root_path: camino::Utf8PathBuf,
    script: tcl::Script,
}

impl Builder {
    pub fn new(path: &str, project_name: &str, part_name: &str) -> Self {
        let mut script = tcl::Script::default();
        script.add(tcl::CreateProject {
            path: path.into(),
            part: part_name.into(),
            name: project_name.into(),
            force: true,
        });
        Self {
            project_name: project_name.into(),
            part_name: part_name.into(),
            script,
            root_path: path.into(),
        }
    }
    pub fn step<T: std::fmt::Display>(mut self, x: T) -> Self {
        self.script.add(x);
        self
    }
    pub fn build(self) -> std::io::Result<()> {
        let file = std::fs::File::create(self.root_path.join("run.tcl"))?;
        let mut buf = std::io::BufWriter::new(file);
        for cmd in self.script.commands {
            writeln!(buf, "{cmd}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::builders::vivado::tcl::{ConfigureIp, CreateIp, GenerateIp};

    use super::*;
    // Run with vivado -mode tcl -source run.tcl

    #[test]
    fn builder_test() {
        std::fs::create_dir_all("jnk").unwrap();
        let mig_prj_path = PathBuf::from("jnk").join("mig_a.prj");
        Builder::new("jnk", "demo", "xc7a50tfgg484-1")
            .step(CreateIp::xilinx("mig_7series", "4.2", "mig7"))
            .step(ConfigureIp::new("mig7", "BOARD_MIG_PARAM", "Custom"))
            .step(ConfigureIp::new("mig7", "MIG_DONT_TOUCH_PARAM", "Custom"))
            .step(ConfigureIp::new("mig7", "RESET_BOARD_INTERFACE", "Custom"))
            .step(ConfigureIp::new(
                "mig7",
                "XML_INPUT_FILE",
                "/home/samitbasu/Devel/rhdl/rhdl-bsp/jnk/mig_a.prj",
            ))
            .step(GenerateIp::new("mig7"))
            .build()
            .unwrap();
    }
}
