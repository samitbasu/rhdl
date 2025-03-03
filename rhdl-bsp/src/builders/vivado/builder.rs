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
    pub fn build(self) -> anyhow::Result<()> {
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
    use super::*;

    #[test]
    fn builder_test() {
        std::fs::create_dir_all("jnk").unwrap();
        let builder = Builder::new("jnk", "demo", "xc7a50tfgg484-1")
            .build()
            .unwrap();
    }
}
