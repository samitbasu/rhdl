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

pub fn write_dot(schematic: &Schematic, mut w: impl Write) -> Result<()> {
    writeln!(w, "digraph schematic {{")?;
    writeln!(w, "rankdir=\"LR\"")?;
    writeln!(w, "remincross=true;")?;
    // Allocate the input ports for the schematic
    schematic
        .inputs
        .iter()
        .enumerate()
        .try_for_each(|(ix, pin)| {
            writeln!(
                w,
                "n{ix} [shape=octagon, label=\"{name}\", color=\"black\"];",
                name = schematic.pin(*pin).name
            )
        })?;
    // Allocate the output ports for the schematic
    schematic
        .outputs
        .iter()
        .enumerate()
        .try_for_each(|(ix, pin)| {
            writeln!(
                w,
                "n{ix} [shape=octagon, label=\"{name}\", color=\"black\"];",
                name = schematic.pin(*pin).name
            )
        })?;
    // Create nodes for each component in the schematic
    schematic
        .components
        .iter()
        .enumerate()
        .try_for_each(|(ndx, component)| write_component(ndx, component, &mut w))?;
    schematic.wires.iter().try_for_each(|wire| {
        let src = schematic.pin(wire.source);
        let dest = schematic.pin(wire.dest);
        writeln!(
            w,
            "{}:{}:e -> {}:{}:w;",
            src.parent, wire.source, dest.parent, wire.dest
        )
    })?;
    writeln!(w, "}}")
}

fn write_component(ndx: usize, component: &Component, w: impl Write) -> Result<()> {
    match &component.kind {
        ComponentKind::Buffer(buf) => write_buffer(ndx, &component.name, buf, w),
        ComponentKind::Binary(bin) => write_binary(ndx, bin, w),
        ComponentKind::Unary(unary) => write_unary(ndx, unary, w),
        ComponentKind::Select(select) => write_select(ndx, select, w),
        ComponentKind::Index(index) => write_index(ndx, index, w),
        ComponentKind::Splice(splice) => write_splice(ndx, splice, w),
        ComponentKind::Repeat(repeat) => write_repeat(ndx, repeat, w),
        ComponentKind::Struct(structure) => write_structure(ndx, &component.name, structure, w),
        ComponentKind::Tuple(tuple) => write_tuple(ndx, tuple, w),
        ComponentKind::Case(case) => write_case(ndx, case, w),
        ComponentKind::Exec(exec) => write_exec(ndx, &component.name, exec, w),
        ComponentKind::Array(array) => write_array(ndx, array, w),
        ComponentKind::Discriminant(disc) => write_discriminant(ndx, disc, w),
        ComponentKind::Enum(enm) => write_enum(ndx, &component.name, enm, w),
        ComponentKind::Constant(constant) => write_constant(ndx, constant, w),
        ComponentKind::Cast(cast) => write_cast(ndx, cast, w),
    }
}

fn write_cnode(
    ndx: usize,
    input_ports: &str,
    label: &str,
    output_ports: &str,
    mut w: impl Write,
) -> Result<()> {
    // Escape the backslashes
    let label = label.replace('\n', "\\n");
    writeln!(
        w,
        "c{ndx} [ shape=record, label=\"{{ {input_ports} | {label} | {output_ports} }}\"]",
    )
}

fn write_buffer(ndx: usize, name: &str, buf: &BufferComponent, w: impl Write) -> Result<()> {
    let input_ports = format!("{{<{}> A}}", buf.input);
    let output_ports = format!("{{<{}> Y}}", buf.output);
    let label = format!("{name}\nBUF");
    write_cnode(ndx, &input_ports, &label, &output_ports, w)
}

fn write_binary(ndx: usize, bin: &BinaryComponent, w: impl Write) -> Result<()> {
    let input_ports = format!("{{<{}> A | <{}> B}}", bin.input1, bin.input2);
    let output_ports = format!("{{<{}> Y}}", bin.output);
    let label = format!("{}", bin.op);
    write_cnode(ndx, &input_ports, &label, &output_ports, w)
}

fn write_unary(ndx: usize, unary: &UnaryComponent, w: impl Write) -> Result<()> {
    let input_ports = format!("{{<{}> A}}", unary.input);
    let output_ports = format!("{{<{}> Y}}", unary.output);
    let label = format!("{}", unary.op);
    write_cnode(ndx, &input_ports, &label, &output_ports, w)
}

fn write_select(ndx: usize, select: &SelectComponent, w: impl Write) -> Result<()> {
    let input_ports = format!(
        "{{<{}> C | <{}> T | <{}> F}}",
        select.cond, select.true_value, select.false_value
    );
    let output_ports = format!("{{<{}> Y}}", select.output);
    let label = "mux";
    write_cnode(ndx, &input_ports, label, &output_ports, w)
}

fn write_index(ndx: usize, index: &IndexComponent, w: impl Write) -> Result<()> {
    let dyn_ports = index
        .dynamic
        .iter()
        .map(|pin| format!("| <{}> D", pin))
        .collect::<Vec<String>>()
        .join("");
    let input_ports = format!("{{<{}> A{dyn_ports}}}", index.arg);
    let output_ports = format!("{{<{}> Y}}", index.output);
    let label = format!("{}\nindex", index.path);
    write_cnode(ndx, &input_ports, &label, &output_ports, w)
}

fn write_splice(ndx: usize, splice: &SpliceComponent, w: impl Write) -> Result<()> {
    let dyn_ports = splice
        .dynamic
        .iter()
        .map(|pin| format!("| <{}> D", pin))
        .collect::<Vec<String>>()
        .join("");
    let input_ports = format!("{{<{}> A|<{}> S{dyn_ports}}}", splice.orig, splice.subst);
    let output_ports = format!("{{<{}> Y}}", splice.output);
    let label = format!("{}\nsplice", splice.path);
    write_cnode(ndx, &input_ports, &label, &output_ports, w)
}

fn write_repeat(ndx: usize, repeat: &RepeatComponent, w: impl Write) -> Result<()> {
    let input_ports = format!("{{<{}> A}}", repeat.value);
    let output_ports = format!("{{<{}> Y}}", repeat.output);
    let label = format!("repeat {}", repeat.len);
    write_cnode(ndx, &input_ports, &label, &output_ports, w)
}

fn write_structure(
    ndx: usize,
    name: &str,
    structure: &StructComponent,
    w: impl Write,
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
    write_cnode(
        ndx,
        &format!("{{ {input_ports} }}"),
        &label,
        &output_ports,
        w,
    )
}

fn write_tuple(ndx: usize, tuple: &TupleComponent, w: impl Write) -> Result<()> {
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
    write_cnode(
        ndx,
        &format!("{{ {input_ports} }}"),
        label,
        &output_ports,
        w,
    )
}

fn write_case(ndx: usize, case: &CaseComponent, w: impl Write) -> Result<()> {
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
    write_cnode(ndx, &input_ports, &label, &output_ports, w)
}

fn write_exec(ndx: usize, name: &str, exec: &ExecComponent, w: impl Write) -> Result<()> {
    let input_ports = exec
        .args
        .iter()
        .enumerate()
        .map(|(pin, ndx)| format!("| <{}> A{}", pin, ndx))
        .collect::<Vec<String>>()
        .join("");
    let output_ports = format!("{{<{}> Y}}", exec.output);
    let label = format!("{name}\nexec");
    write_cnode(ndx, &input_ports, &label, &output_ports, w)
}

fn write_array(ndx: usize, array: &ArrayComponent, w: impl Write) -> Result<()> {
    let input_ports = array
        .elements
        .iter()
        .enumerate()
        .map(|(pin, ndx)| format!("| <{}> A{}", pin, ndx))
        .collect::<Vec<String>>()
        .join("");
    let output_ports = format!("{{<{}> Y}}", array.output);
    let label = "array";
    write_cnode(ndx, &input_ports, &label, &output_ports, w)
}

fn write_discriminant(ndx: usize, disc: &DiscriminantComponent, w: impl Write) -> Result<()> {
    let input_ports = format!("{{<{}> A}}", disc.arg);
    let output_ports = format!("{{<{}> Y}}", disc.output);
    let label = "discriminant";
    write_cnode(ndx, &input_ports, &label, &output_ports, w)
}

fn write_enum(ndx: usize, name: &str, enm: &EnumComponent, w: impl Write) -> Result<()> {
    let input_ports = enm
        .fields
        .iter()
        .map(|member| format!("| <{}> {}", member.pin, member.member))
        .collect::<Vec<String>>()
        .join("");
    let output_ports = format!("{{<{}> Y}}", enm.output);
    let label = format!("{name}\nenum");
    write_cnode(ndx, &input_ports, &label, &output_ports, w)
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

fn write_constant(ndx: usize, constant: &ConstantComponent, mut w: impl Write) -> Result<()> {
    let output_ports = format!("{{<{}> Y}}", constant.output);
    let value = escape_string(&elide_string(&constant.value.to_string()));
    let tooltip = escape_string(&indent_string(&constant.value.to_string()));
    let label = format!("{}\nconstant", value);
    // Escape the backslashes
    let label = escape_string(&label);
    writeln!(
        w,
        "c{ndx} [ shape=record, label=\"{{  | {label} | {output_ports} }}\", tooltip=\"{tooltip}\"]",
    )
}

fn write_cast(ndx: usize, cast: &CastComponent, w: impl Write) -> Result<()> {
    let input_ports = format!("{{<{}> A}}", cast.input);
    let output_ports = format!("{{<{}> Y}}", cast.output);
    let label = "cast";
    write_cnode(ndx, &input_ports, label, &output_ports, w)
}
