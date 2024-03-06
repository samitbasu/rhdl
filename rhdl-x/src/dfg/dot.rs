use std::array;
use std::io::Result;
use std::io::Write;

use rhdl_core::rhif::spec::Case;

use crate::dfg::components::ComponentKind;

use super::components::ArrayComponent;
use super::components::BinaryComponent;
use super::components::BufferComponent;
use super::components::CaseComponent;
use super::components::CastComponent;
use super::components::ConstantComponent;
use super::components::DiscriminantComponent;
use super::components::EnumComponent;
use super::components::ExecComponent;
use super::components::IndexComponent;
use super::components::RepeatComponent;
use super::components::SelectComponent;
use super::components::SpliceComponent;
use super::components::StructComponent;
use super::components::TupleComponent;
use super::components::UnaryComponent;
use super::{components::Component, schematic::Schematic};

struct DotWriter<'a, 'b, W: Write> {
    w: W,
    schematic: &'a Schematic,
    base_offset: usize,
    next_free: &'b mut usize,
}

pub fn write_dot(schematic: &Schematic, mut w: impl Write) -> Result<()> {
    writeln!(w, "digraph schematic {{")?;
    writeln!(w, "rankdir=\"LR\"")?;
    writeln!(w, "remincross=true;")?;
    let mut next_free = schematic.components.len();
    let mut dot = DotWriter {
        w: &mut w,
        schematic,
        base_offset: 0,
        next_free: &mut next_free,
    };
    dot.write_schematic()?;
    writeln!(w, "}}")
}

impl<'a, 'b, W: Write> DotWriter<'a, 'b, W> {
    fn write_schematic(&mut self) -> Result<()> {
        // Allocate the input ports for the schematic
        self.schematic
            .inputs
            .iter()
            .enumerate()
            .try_for_each(|(ix, pin)| {
                writeln!(
                    self.w,
                    "a{ix} [shape=octagon, label=\"{name}\", color=\"black\"];",
                    name = self.schematic.pin(*pin).name
                )
            })?;
        // Allocate the output ports for the schematic
        self.schematic
            .outputs
            .iter()
            .enumerate()
            .try_for_each(|(ix, pin)| {
                writeln!(
                    self.w,
                    "o{ix} [shape=octagon, label=\"{name}\", color=\"black\"];",
                    name = self.schematic.pin(*pin).name
                )
            })?;
        // Create nodes for each component in the schematic

        for (ndx, component) in self.schematic.components.iter().enumerate() {
            self.write_component(ndx + self.base_offset, component)?;
        }

        self.schematic.wires.iter().try_for_each(|wire| {
            let src = self.schematic.pin(wire.source);
            let dest = self.schematic.pin(wire.dest);
            writeln!(
                self.w,
                "{}:{}:e -> {}:{}:w;",
                src.parent, wire.source, dest.parent, wire.dest
            )
        })?;
        // Add wires from the input schematic ports to the input buffer pins
        self.schematic
            .inputs
            .iter()
            .enumerate()
            .try_for_each(|(ix, pin)| {
                let pin_data = self.schematic.pin(*pin);
                writeln!(
                    self.w,
                    "a{ix} -> {parent}:{pin};",
                    ix = ix,
                    parent = pin_data.parent,
                    pin = pin
                )
            })?;
        // Add wires from the output schematic ports to the output buffer pins
        self.schematic
            .outputs
            .iter()
            .enumerate()
            .try_for_each(|(ix, pin)| {
                let pin_data = self.schematic.pin(*pin);
                writeln!(
                    self.w,
                    "{parent}:{pin} -> o{ix};",
                    ix = ix,
                    parent = pin_data.parent,
                    pin = pin
                )
            })
    }

    fn write_component(&mut self, ndx: usize, component: &Component) -> Result<()> {
        match &component.kind {
            ComponentKind::Buffer(buf) => self.write_buffer(ndx, &component.name, buf),
            ComponentKind::Binary(bin) => self.write_binary(ndx, bin),
            ComponentKind::Unary(unary) => self.write_unary(ndx, unary),
            ComponentKind::Select(select) => self.write_select(ndx, select),
            ComponentKind::Index(index) => self.write_index(ndx, index),
            ComponentKind::Splice(splice) => self.write_splice(ndx, splice),
            ComponentKind::Repeat(repeat) => self.write_repeat(ndx, repeat),
            ComponentKind::Struct(structure) => {
                self.write_structure(ndx, &component.name, structure)
            }
            ComponentKind::Tuple(tuple) => self.write_tuple(ndx, tuple),
            ComponentKind::Case(case) => self.write_case(ndx, case),
            ComponentKind::Exec(exec) => self.write_exec(ndx, exec),
            ComponentKind::Array(array) => self.write_array(ndx, array),
            ComponentKind::Discriminant(disc) => self.write_discriminant(ndx, disc),
            ComponentKind::Enum(enm) => self.write_enum(ndx, &component.name, enm),
            ComponentKind::Constant(constant) => self.write_constant(ndx, constant),
            ComponentKind::Cast(cast) => self.write_cast(ndx, cast),
        }
    }

    fn write_cnode(
        &mut self,
        ndx: usize,
        input_ports: &str,
        label: &str,
        output_ports: &str,
    ) -> Result<()> {
        // Escape the backslashes
        let label = label.replace('\n', "\\n");
        writeln!(
            self.w,
            "c{ndx} [ shape=record, label=\"{{ {input_ports} | {label} | {output_ports} }}\"]",
        )
    }

    fn write_buffer(&mut self, ndx: usize, name: &str, buf: &BufferComponent) -> Result<()> {
        let input_ports = format!("{{<{}> A}}", buf.input);
        let output_ports = format!("{{<{}> Y}}", buf.output);
        let label = format!("{name}\nBUF");
        self.write_cnode(ndx, &input_ports, &label, &output_ports)
    }

    fn write_binary(&mut self, ndx: usize, bin: &BinaryComponent) -> Result<()> {
        let input_ports = format!("{{<{}> A | <{}> B}}", bin.input1, bin.input2);
        let output_ports = format!("{{<{}> Y}}", bin.output);
        let label = format!("{}", bin.op);
        self.write_cnode(ndx, &input_ports, &label, &output_ports)
    }

    fn write_unary(&mut self, ndx: usize, unary: &UnaryComponent) -> Result<()> {
        let input_ports = format!("{{<{}> A}}", unary.input);
        let output_ports = format!("{{<{}> Y}}", unary.output);
        let label = format!("{}", unary.op);
        self.write_cnode(ndx, &input_ports, &label, &output_ports)
    }

    fn write_select(&mut self, ndx: usize, select: &SelectComponent) -> Result<()> {
        let input_ports = format!(
            "{{<{}> C | <{}> T | <{}> F}}",
            select.cond, select.true_value, select.false_value
        );
        let output_ports = format!("{{<{}> Y}}", select.output);
        let label = "mux";
        self.write_cnode(ndx, &input_ports, label, &output_ports)
    }

    fn write_index(&mut self, ndx: usize, index: &IndexComponent) -> Result<()> {
        let dyn_ports = index
            .dynamic
            .iter()
            .map(|pin| format!("| <{}> D", pin))
            .collect::<Vec<String>>()
            .join("");
        let input_ports = format!("{{<{}> A{dyn_ports}}}", index.arg);
        let output_ports = format!("{{<{}> Y}}", index.output);
        let label = format!("{}\nindex", index.path);
        self.write_cnode(ndx, &input_ports, &label, &output_ports)
    }

    fn write_splice(&mut self, ndx: usize, splice: &SpliceComponent) -> Result<()> {
        let dyn_ports = splice
            .dynamic
            .iter()
            .map(|pin| format!("| <{}> D", pin))
            .collect::<Vec<String>>()
            .join("");
        let input_ports = format!("{{<{}> A|<{}> S{dyn_ports}}}", splice.orig, splice.subst);
        let output_ports = format!("{{<{}> Y}}", splice.output);
        let label = format!("{}\nsplice", splice.path);
        self.write_cnode(ndx, &input_ports, &label, &output_ports)
    }

    fn write_repeat(&mut self, ndx: usize, repeat: &RepeatComponent) -> Result<()> {
        let input_ports = format!("{{<{}> A}}", repeat.value);
        let output_ports = format!("{{<{}> Y}}", repeat.output);
        let label = format!("repeat {}", repeat.len);
        self.write_cnode(ndx, &input_ports, &label, &output_ports)
    }

    fn write_structure(
        &mut self,
        ndx: usize,
        name: &str,
        structure: &StructComponent,
    ) -> Result<()> {
        let mut input_ports = structure
            .fields
            .iter()
            .map(|member| format!("<{}> {}", member.pin, member.member))
            .collect::<Vec<String>>()
            .join("|");
        if let Some(rest) = structure.rest {
            input_ports.push_str(&format!("| <{}> ...", rest));
        }
        let output_ports = format!("{{<{}> Y}}", structure.output);
        let label = format!("{}\nstruct", structure.kind.get_name());
        self.write_cnode(ndx, &format!("{{ {input_ports} }}"), &label, &output_ports)
    }

    fn write_tuple(&mut self, ndx: usize, tuple: &TupleComponent) -> Result<()> {
        let input_ports = tuple
            .fields
            .iter()
            .enumerate()
            .map(|(ndx, pin)| format!("<{}> .{ndx}", pin))
            .collect::<Vec<String>>()
            .join("|");
        let output_ports = format!("{{<{}> Y}}", tuple.output);
        let label = if tuple.fields.is_empty() {
            "()"
        } else {
            "tuple"
        };
        self.write_cnode(ndx, &format!("{{ {input_ports} }}"), label, &output_ports)
    }

    fn write_case(&mut self, ndx: usize, case: &CaseComponent) -> Result<()> {
        let discriminant_port = format!("<{}> D", case.discriminant);
        let table_ports = case
            .table
            .iter()
            .map(|(arg, pin)| format!("| <{}> {}", pin, arg))
            .collect::<Vec<String>>()
            .join("");
        let input_ports = format!("{{{discriminant_port}{table_ports}}}");
        let output_ports = format!("{{<{}> Y}}", case.output);
        let label = "case";
        self.write_cnode(ndx, &input_ports, &label, &output_ports)
    }

    fn write_exec(&mut self, ndx: usize, exec: &ExecComponent) -> Result<()> {
        let input_ports = exec
            .args
            .iter()
            .enumerate()
            .map(|(ndx, pin)| format!("<{}> {}", pin, ndx))
            .collect::<Vec<String>>()
            .join("|");
        let output_ports = format!("{{<{}> Y}}", exec.output);
        let label = format!("{name}\nexec", name = exec.name);
        if let Some(schematic) = &exec.sub_schematic {
            let base = *self.next_free;
            *self.next_free += schematic.components.len();
            let mut dot = DotWriter {
                w: &mut self.w,
                schematic,
                base_offset: base,
                next_free: self.next_free,
            };
            dot.write_schematic()?;
        }
        self.write_cnode(ndx, &format!("{{ {input_ports} }}"), &label, &output_ports)
    }

    fn write_array(&mut self, ndx: usize, array: &ArrayComponent) -> Result<()> {
        let input_ports = array
            .elements
            .iter()
            .enumerate()
            .map(|(pin, ndx)| format!("| <{}> A{}", pin, ndx))
            .collect::<Vec<String>>()
            .join("");
        let output_ports = format!("{{<{}> Y}}", array.output);
        let label = "array";
        self.write_cnode(ndx, &input_ports, &label, &output_ports)
    }

    fn write_discriminant(&mut self, ndx: usize, disc: &DiscriminantComponent) -> Result<()> {
        let input_ports = format!("{{<{}> A}}", disc.arg);
        let output_ports = format!("{{<{}> Y}}", disc.output);
        let label = "discriminant";
        self.write_cnode(ndx, &input_ports, &label, &output_ports)
    }

    fn write_enum(&mut self, ndx: usize, name: &str, enm: &EnumComponent) -> Result<()> {
        let input_ports = enm
            .fields
            .iter()
            .map(|member| format!("| <{}> {}", member.pin, member.member))
            .collect::<Vec<String>>()
            .join("");
        let output_ports = format!("{{<{}> Y}}", enm.output);
        let label = format!("{name}\nenum");
        self.write_cnode(ndx, &input_ports, &label, &output_ports)
    }

    fn write_constant(&mut self, ndx: usize, constant: &ConstantComponent) -> Result<()> {
        let output_ports = format!("{{<{}> Y}}", constant.output);
        let value = escape_string(&elide_string(&constant.value.to_string()));
        let tooltip = escape_string(&indent_string(&constant.value.to_string()));
        let label = format!("{}\nconstant", value);
        // Escape the backslashes
        let label = escape_string(&label);
        writeln!(
        self.w,
        "c{ndx} [ shape=record, label=\"{{  | {label} | {output_ports} }}\", tooltip=\"{tooltip}\"]",
    )
    }

    fn write_cast(&mut self, ndx: usize, cast: &CastComponent) -> Result<()> {
        let input_ports = format!("{{<{}> A}}", cast.input);
        let output_ports = format!("{{<{}> Y}}", cast.output);
        let label = "cast";
        self.write_cnode(ndx, &input_ports, label, &output_ports)
    }
}

fn escape_string(s: &str) -> String {
    s.replace('{', "\\{")
        .replace('}', "\\}")
        .replace('<', "\\<")
        .replace('>', "\\>")
        .replace('\n', "\\n")
}

fn elide_string(s: &str) -> String {
    if s.len() < 15 {
        return s.to_string();
    }
    s.chars()
        .take(15)
        .chain(std::iter::repeat('.').take(3))
        .collect()
}

fn indent_string(s: &str) -> String {
    // Each left { increases the indent by 2 spaces
    // Each right } decreases the indent by 2 spaces
    let mut indent = 0;
    let mut result = String::new();
    for c in s.chars() {
        if c == '{' {
            indent += 2;
            result.push(c);
            result.push('\n');
            result.push_str(&" ".repeat(indent));
        } else if c == '}' {
            indent -= 2;
            result.push('\n');
            result.push_str(&" ".repeat(indent));
            result.push(c);
        } else if c == ',' {
            result.push(c);
            result.push('\n');
            result.push_str(&" ".repeat(indent));
        } else {
            result.push(c);
        }
    }
    result
}
