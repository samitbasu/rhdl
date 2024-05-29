use std::collections::HashSet;
use std::io::Result;
use std::io::Write;
use std::iter::once;

use super::schematic_impl::Trace;
use super::{
    components::{
        ArrayComponent, BinaryComponent, BlackBoxComponent, BufferComponent, CaseComponent,
        CastComponent, Component, ComponentKind, ConstantComponent, EnumComponent, IndexComponent,
        KernelComponent, RepeatComponent, SelectComponent, SpliceComponent, StructComponent,
        TupleComponent, UnaryComponent,
    },
    schematic_impl::Schematic,
};

struct DotWriter<'a, 'b, W: Write> {
    w: &'b mut W,
    schematic: &'a Schematic,
}

pub fn write_dot(schematic: &Schematic, trace: Option<&Trace>, mut w: impl Write) -> Result<()> {
    writeln!(w, "digraph schematic {{")?;
    writeln!(w, "rankdir=\"LR\"")?;
    writeln!(w, "remincross=true;")?;
    let mut dot = DotWriter {
        w: &mut w,
        schematic,
    };
    dot.write_schematic(trace)?;
    writeln!(w, "}}")
}

impl<'a, 'b, W: Write> DotWriter<'a, 'b, W> {
    fn write_schematic(&mut self, trace: Option<&Trace>) -> Result<()> {
        // Allocate the input ports for the schematic
        self.schematic
            .inputs
            .iter()
            .enumerate()
            .try_for_each(|(ix, pin)| {
                writeln!(
                    self.w,
                    "a{ix} [shape=octagon, label=\"{name}\", color=\"black\"];",
                    ix = ix,
                    name = self.schematic.pin(*pin).name
                )
            })?;
        // Allocate the output ports for the schematic
        writeln!(
            self.w,
            "o{ix} [shape=octagon, label=\"{name}\", color=\"black\"];",
            ix = 0,
            name = self.schematic.pin(self.schematic.output).name
        )?;
        // Create nodes for each component in the schematic
        self.write_components(&[])?;

        self.schematic.wires.iter().try_for_each(|wire| {
            let src = self.schematic.pin(wire.source);
            let src_component: usize = src.parent.into();
            let dest = self.schematic.pin(wire.dest);
            let dest_component: usize = dest.parent.into();
            if self.schematic.components[src_component].is_noop()
                || self.schematic.components[dest_component].is_noop()
            {
                return Ok(());
            }
            let label = if let Some(t) = trace.as_ref().and_then(|t| {
                t.paths
                    .iter()
                    .find(|t| t.source == wire.source && t.dest == wire.dest)
            }) {
                format!("[label=\"{:?}\", color=\"red\"]", t.path)
            } else {
                "".to_string()
            };
            writeln!(
                self.w,
                "c{}:{:?}:e -> c{}:{:?}:w {};",
                src_component, wire.source, dest_component, wire.dest, label
            )
        })?;

        // Add wires from the input schematic ports to the input buffer pins
        self.schematic
            .inputs
            .iter()
            .enumerate()
            .try_for_each(|(ix, pin)| {
                let pin_data = self.schematic.pin(*pin);
                let parent_component: usize = pin_data.parent.into();
                let label = if let Some(t) = trace.as_ref().and_then(|t| {
                    t.sinks
                        .iter()
                        .chain(once(&t.source))
                        .find(|t| t.pin == *pin)
                }) {
                    format!("[label=\"{:?}\", color=\"red\"]", t.path)
                } else {
                    "".to_string()
                };
                writeln!(
                    self.w,
                    "a{ix}:e -> c{parent}:{pin:?}:w {label};",
                    ix = ix,
                    parent = parent_component,
                    pin = pin,
                    label = label
                )
            })?;
        // Add wires from the output schematic ports to the output buffer pins
        let pin_data = self.schematic.pin(self.schematic.output);
        let parent_component: usize = pin_data.parent.into();
        let label = if let Some(t) = trace.as_ref().and_then(|t| {
            t.sinks
                .iter()
                .chain(once(&t.source))
                .find(|t| t.pin == self.schematic.output)
        }) {
            format!("[label=\"{:?}\", color=\"red\"]", t.path)
        } else {
            "".to_string()
        };
        writeln!(
            self.w,
            "c{parent}:{pin:?}:e -> o{ix}:w {label};",
            ix = 0,
            parent = parent_component,
            pin = self.schematic.output,
            label = label
        )
    }

    fn write_component(&mut self, ndx: usize, component: &Component) -> Result<()> {
        match &component.kind {
            ComponentKind::Buffer(buf) => self.write_buffer(ndx, buf),
            ComponentKind::Binary(bin) => self.write_binary(ndx, bin),
            ComponentKind::Unary(unary) => self.write_unary(ndx, unary),
            ComponentKind::Select(select) => self.write_select(ndx, select),
            ComponentKind::Index(index) => self.write_index(ndx, index),
            ComponentKind::Splice(splice) => self.write_splice(ndx, splice),
            ComponentKind::Repeat(repeat) => self.write_repeat(ndx, repeat),
            ComponentKind::Struct(structure) => self.write_structure(ndx, structure),
            ComponentKind::Tuple(tuple) => self.write_tuple(ndx, tuple),
            ComponentKind::Case(case) => self.write_case(ndx, case),
            ComponentKind::BlackBox(exec) => self.write_black_box(ndx, exec),
            ComponentKind::Array(array) => self.write_array(ndx, array),
            ComponentKind::Enum(enm) => self.write_enum(ndx, enm),
            ComponentKind::Constant(constant) => self.write_constant(ndx, constant),
            ComponentKind::Cast(cast) => self.write_cast(ndx, cast),
            ComponentKind::Kernel(kernel) => self.write_kernel(ndx, kernel),
            ComponentKind::Noop => Ok(()),
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

    fn write_buffer(&mut self, ndx: usize, buf: &BufferComponent) -> Result<()> {
        let input_ports = format!("{{<{:?}> A}}", buf.input);
        let output_ports = format!("{{<{:?}> Y}}", buf.output);
        let label = format!("{kind:?}\nBuf", kind = &self.schematic.pin(buf.input).kind);
        self.write_cnode(ndx, &input_ports, &label, &output_ports)
    }

    fn write_binary(&mut self, ndx: usize, bin: &BinaryComponent) -> Result<()> {
        let input_ports = format!("{{<{:?}> A | <{:?}> B}}", bin.input1, bin.input2);
        let output_ports = format!("{{<{:?}> Y}}", bin.output);
        let label = format!("{:?}", bin.op);
        self.write_cnode(ndx, &input_ports, &label, &output_ports)
    }

    fn write_unary(&mut self, ndx: usize, unary: &UnaryComponent) -> Result<()> {
        let input_ports = format!("{{<{:?}> A}}", unary.input);
        let output_ports = format!("{{<{:?}> Y}}", unary.output);
        let label = format!("{:?}", unary.op);
        self.write_cnode(ndx, &input_ports, &label, &output_ports)
    }

    fn write_select(&mut self, ndx: usize, select: &SelectComponent) -> Result<()> {
        let input_ports = format!(
            "{{<{:?}> C | <{:?}> T | <{:?}> F}}",
            select.cond, select.true_value, select.false_value
        );
        let output_ports = format!("{{<{:?}> Y}}", select.output);
        let label = "mux";
        self.write_cnode(ndx, &input_ports, label, &output_ports)
    }

    fn write_index(&mut self, ndx: usize, index: &IndexComponent) -> Result<()> {
        let dyn_ports = index
            .dynamic
            .iter()
            .map(|pin| format!("| <{:?}> D", pin))
            .collect::<Vec<String>>()
            .join("");
        let input_ports = format!("{{<{:?}> A{dyn_ports}}}", index.arg);
        let output_ports = format!("{{<{:?}> Y}}", index.output);
        let label = format!("{:?}\nindex", index.path);
        self.write_cnode(ndx, &input_ports, &label, &output_ports)
    }

    fn write_splice(&mut self, ndx: usize, splice: &SpliceComponent) -> Result<()> {
        let dyn_ports = splice
            .dynamic
            .iter()
            .map(|pin| format!("| <{:?}> D", pin))
            .collect::<Vec<String>>()
            .join("");
        let input_ports = format!(
            "{{<{:?}> A|<{:?}> S{dyn_ports}}}",
            splice.orig, splice.subst
        );
        let output_ports = format!("{{<{:?}> Y}}", splice.output);
        let label = format!("{:?}\nsplice", splice.path);
        self.write_cnode(ndx, &input_ports, &label, &output_ports)
    }

    fn write_repeat(&mut self, ndx: usize, repeat: &RepeatComponent) -> Result<()> {
        let input_ports = format!("{{<{:?}> A}}", repeat.value);
        let output_ports = format!("{{<{:?}> Y}}", repeat.output);
        let label = format!("repeat {}", repeat.len);
        self.write_cnode(ndx, &input_ports, &label, &output_ports)
    }

    fn write_structure(&mut self, ndx: usize, structure: &StructComponent) -> Result<()> {
        let mut input_ports = structure
            .fields
            .iter()
            .map(|member| format!("<{:?}> {:?}", member.pin, member.member))
            .collect::<Vec<String>>()
            .join("|");
        if let Some(rest) = structure.rest {
            input_ports.push_str(&format!("| <{:?}> ...", rest));
        }
        let output_ports = format!("{{<{:?}> Y}}", structure.output);
        let label = format!("{}\nstruct", structure.kind.get_name());
        self.write_cnode(ndx, &format!("{{ {input_ports} }}"), &label, &output_ports)
    }

    fn write_tuple(&mut self, ndx: usize, tuple: &TupleComponent) -> Result<()> {
        let input_ports = tuple
            .fields
            .iter()
            .enumerate()
            .map(|(ndx, pin)| format!("<{:?}> .{ndx}", pin))
            .collect::<Vec<String>>()
            .join("|");
        let output_ports = format!("{{<{:?}> Y}}", tuple.output);
        let label = if tuple.fields.is_empty() {
            "()"
        } else {
            "tuple"
        };
        self.write_cnode(ndx, &format!("{{ {input_ports} }}"), label, &output_ports)
    }

    fn write_case(&mut self, ndx: usize, case: &CaseComponent) -> Result<()> {
        let discriminant_port = format!("<{:?}> D", case.discriminant);
        let table_ports = case
            .table
            .iter()
            .map(|(arg, pin)| format!("| <{:?}> {:?}", pin, arg))
            .collect::<Vec<String>>()
            .join("");
        let input_ports = format!("{{{discriminant_port}{table_ports}}}");
        let output_ports = format!("{{<{:?}> Y}}", case.output);
        let label = "case";
        self.write_cnode(ndx, &input_ports, label, &output_ports)
    }

    fn write_black_box(&mut self, ndx: usize, black_box: &BlackBoxComponent) -> Result<()> {
        let input_ports = black_box
            .0
            .args()
            .iter()
            .enumerate()
            .map(|(ndx, pin)| format!("<{:?}> {}", pin, ndx))
            .collect::<Vec<String>>()
            .join("|");
        let output_ports = format!("{{<{:?}> Y}}", black_box.0.output());
        let label = black_box.0.name();
        self.write_cnode(ndx, &format!("{{ {input_ports} }}"), label, &output_ports)
    }

    fn write_kernel(&mut self, ndx: usize, kernel: &KernelComponent) -> Result<()> {
        let input_ports = kernel
            .args
            .iter()
            .enumerate()
            .map(|(ndx, pin)| format!("<{:?}> {}", pin, ndx))
            .collect::<Vec<String>>()
            .join("|");
        let output_ports = format!("{{<{:?}> Y}}", kernel.output);
        let label = format!("{name}\nkernel", name = kernel.name);
        self.write_cnode(ndx, &format!("{{ {input_ports} }}"), &label, &output_ports)
    }

    fn write_array(&mut self, ndx: usize, array: &ArrayComponent) -> Result<()> {
        let input_ports = array
            .elements
            .iter()
            .enumerate()
            .map(|(pin, ndx)| format!("| <{:?}> A{:?}", pin, ndx))
            .collect::<Vec<String>>()
            .join("");
        let output_ports = format!("{{<{:?}> Y}}", array.output);
        let label = "array";
        self.write_cnode(ndx, &input_ports, label, &output_ports)
    }

    fn write_enum(&mut self, ndx: usize, enm: &EnumComponent) -> Result<()> {
        let input_ports = enm
            .fields
            .iter()
            .map(|member| format!("| <{:?}> {:?}", member.pin, member.member))
            .collect::<Vec<String>>()
            .join("");
        let output_ports = format!("{{<{:?}> Y}}", enm.output);
        let label = format!(
            "{kind:?}\nenum",
            kind = &self.schematic.pin(enm.output).kind
        );
        self.write_cnode(ndx, &input_ports, &label, &output_ports)
    }

    fn write_constant(&mut self, ndx: usize, constant: &ConstantComponent) -> Result<()> {
        let output_ports = format!("{{<{:?}> Y}}", constant.output);
        let value = escape_string(&elide_string(&format!("{:?}", constant.value)));
        let tooltip = escape_string(&indent_string(&format!("{:?}", constant.value)));
        let label = format!("{}\nconstant", value);
        // Escape the backslashes
        let label = escape_string(&label);
        writeln!(
        self.w,
        "c{ndx} [ shape=record, label=\"{{  | {label} | {output_ports} }}\", tooltip=\"{tooltip}\"]",
    )
    }

    fn write_cast(&mut self, ndx: usize, cast: &CastComponent) -> Result<()> {
        let input_ports = format!("{{<{:?}> A}}", cast.input);
        let output_ports = format!("{{<{:?}> Y}}", cast.output);
        let label = "cast";
        self.write_cnode(ndx, &input_ports, label, &output_ports)
    }

    // Write out the components in a hierarchical fashion.
    fn write_components(&mut self, path: &[String]) -> Result<()> {
        writeln!(self.w, "subgraph cluster_{path} {{", path = path.join("_"))?;
        for (ndx, component) in self.schematic.components.iter().enumerate() {
            if component.path == path {
                self.write_component(ndx, component)?;
            }
        }
        // Collect all immediate children of the current path
        let children: HashSet<String> = self
            .schematic
            .components
            .iter()
            .filter_map(|component| {
                if component.path.len() == path.len() + 1 && component.path.starts_with(path) {
                    Some(component.path.last().unwrap().clone())
                } else {
                    None
                }
            })
            .collect();
        for child in children {
            let child_path = &[path, &[child]].concat();
            self.write_components(child_path)?;
        }
        writeln!(self.w, "}}")?;
        Ok(())
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
