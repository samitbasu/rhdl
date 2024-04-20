use rhdl_core::Kind;

#[derive(Clone, Debug)]
pub struct Descriptions {
    pub name: String,
    pub path: String,
    pub input_kind: Kind,
    pub output_kind: Kind,
    pub children: Vec<Descriptions>,
}

// Print out the descriptions using indentation for the
// children

impl Descriptions {
    pub fn print(&self, indent_level: usize, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let indent = "  ".repeat(indent_level * 3);
        writeln!(
            f,
            "{}{}/{}: {} -> {}",
            indent, self.path, self.name, self.input_kind, self.output_kind
        )?;
        for child in &self.children {
            child.print(indent_level + 1, f)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for Descriptions {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.print(0, f)
    }
}
