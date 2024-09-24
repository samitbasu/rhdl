use std::collections::BTreeMap;

use crate::compiler::codegen::verilog::generate_verilog;
use crate::types::path::bit_range;
use crate::types::path::Path;
use crate::util::delim_list_optional_strings;
use crate::util::terminate_list_optional_strings;
use crate::Digital;
use crate::HDLKind;
use crate::Tristate;
use crate::{Circuit, HDLDescriptor, RHDLError, Synchronous};
enum Direction {
    Input,
    Output,
    Inout,
}

fn maybe_decl(dir: Option<Direction>, num_bits: usize, name: &str) -> Option<String> {
    let dir = match dir {
        Some(Direction::Input) => "input",
        Some(Direction::Output) => "output",
        Some(Direction::Inout) => "inout",
        None => "",
    };
    if num_bits == 0 {
        None
    } else {
        Some(format!(
            "{dir} wire[{NUM_BITS}:0] {NAME}",
            dir = dir,
            NUM_BITS = num_bits.saturating_sub(1),
            NAME = name
        ))
    }
}

pub fn build_hdl<C: Circuit>(
    circuit: &C,
    children: BTreeMap<String, HDLDescriptor>,
    _kind: HDLKind,
) -> Result<HDLDescriptor, RHDLError> {
    let descriptor = circuit.descriptor()?;
    let outputs = C::O::bits();

    let module_name = &descriptor.unique_name;
    let io_decl = maybe_decl(Some(Direction::Inout), C::Z::N, "io");
    let in_decl = maybe_decl(Some(Direction::Input), C::I::bits(), "i");
    let out_decl = maybe_decl(Some(Direction::Output), C::O::bits(), "o");

    let module_decl = format!(
        "module {module_name}({decls});",
        module_name = module_name,
        decls = delim_list_optional_strings(&[io_decl, in_decl.clone(), out_decl], ",")
    );

    let od_decls = maybe_decl(None, C::O::bits() + C::D::bits(), "od");
    let d_decls = maybe_decl(None, C::D::bits(), "d");
    let q_decls = maybe_decl(None, C::Q::bits(), "q");

    let d_kind = C::D::static_kind();
    let q_kind = C::Q::static_kind();

    let child_decls = descriptor
        .children
        .iter()
        .enumerate()
        .map(|(ndx, (local_name, descriptor))| {
            let child_path = Path::default().field(local_name);
            let (d_range, _) = bit_range(d_kind.clone(), &child_path)?;
            let (q_range, _) = bit_range(q_kind.clone(), &child_path)?;
            let input_binding = if d_range.is_empty() {
                None
            } else {
                Some(format!(
                    ".i(d[{d_msb}:{d_lsb}])",
                    d_msb = d_range.end.saturating_sub(1),
                    d_lsb = d_range.start
                ))
            };
            let output_binding = if q_range.is_empty() {
                None
            } else {
                Some(format!(
                    ".o(q[{q_msb}:{q_lsb}])",
                    q_msb = q_range.end.saturating_sub(1),
                    q_lsb = q_range.start
                ))
            };
            let component_name = &descriptor.unique_name;
            let args = delim_list_optional_strings(&[input_binding, output_binding], ",");
            Ok(format!("{component_name} c{ndx}({args});"))
        })
        .collect::<Result<Vec<String>, RHDLError>>()?;
    let verilog = generate_verilog(descriptor.rtl.as_ref().unwrap())?;
    // Call the verilog function with (i, q), if they exist.
    let i_bind = in_decl.clone().map(|_| "i".to_string());
    let q_bind = q_decls.clone().map(|_| "q".to_string());
    let iq_bind = delim_list_optional_strings(&[i_bind, q_bind], ",");
    let fn_call = format!("assign od = {fn_name}({iq_bind});", fn_name = verilog.name);
    let fn_body = &verilog.body;
    let o_bind = format!("assign o = od[{}:{}];", outputs.saturating_sub(1), 0);
    let d_bind = if C::D::bits() == 0 {
        None
    } else {
        Some(format!(
            "assign d = od[{}:{}];",
            C::D::bits().saturating_sub(1),
            outputs
        ))
    };
    let code = format!(
        "{module_decl}
{decls}
{o_bind}

{child_decls}

{fn_call}

{fn_body}

endmodule
        
        ",
        decls = terminate_list_optional_strings(&[od_decls, d_decls, q_decls, d_bind], ";"),
        child_decls = child_decls.join("\n"),
    );
    Ok(HDLDescriptor {
        name: module_name.into(),
        body: code,
        children,
    })
}

// There is a fair amount of overlap between this function and the previous one.  In principle,
// it should be possible to factor out the common bits and DRY up the code.
pub fn build_synchronous_hdl<C: Synchronous>(
    circuit: &C,
    children: BTreeMap<String, HDLDescriptor>,
) -> Result<HDLDescriptor, RHDLError> {
    let descriptor = circuit.descriptor()?;
    let outputs = C::O::bits();

    let module_name = &descriptor.unique_name;
    let io_decl = maybe_decl(Some(Direction::Inout), C::Z::N, "io");
    let in_decl = maybe_decl(Some(Direction::Input), C::I::bits(), "i");
    let out_decl = maybe_decl(Some(Direction::Output), C::O::bits(), "o");
    let clock = Some("input wire clock".to_string());
    let reset = Some("input wire reset".to_string());

    let module_decl = format!(
        "module {module_name}({decls});",
        module_name = module_name,
        decls =
            delim_list_optional_strings(&[clock, reset, io_decl, in_decl.clone(), out_decl], ",")
    );

    let od_decls = maybe_decl(None, C::O::bits() + C::D::bits(), "od");
    let d_decls = maybe_decl(None, C::D::bits(), "d");
    let q_decls = maybe_decl(None, C::Q::bits(), "q");

    let d_kind = C::D::static_kind();
    let q_kind = C::Q::static_kind();

    let child_decls = descriptor
        .children
        .iter()
        .enumerate()
        .map(|(ndx, (local_name, descriptor))| {
            let child_path = Path::default().field(local_name);
            let (d_range, _) = bit_range(d_kind.clone(), &child_path)?;
            let (q_range, _) = bit_range(q_kind.clone(), &child_path)?;
            let input_binding = if d_range.is_empty() {
                None
            } else {
                Some(format!(
                    ".i(d[{d_msb}:{d_lsb}])",
                    d_msb = d_range.end.saturating_sub(1),
                    d_lsb = d_range.start
                ))
            };
            let output_binding = if q_range.is_empty() {
                None
            } else {
                Some(format!(
                    ".o(q[{q_msb}:{q_lsb}])",
                    q_msb = q_range.end.saturating_sub(1),
                    q_lsb = q_range.start
                ))
            };
            let component_name = &descriptor.unique_name;
            let clock_binding = Some(".clock(clock)".to_string());
            let reset_binding = Some(".reset(reset)".to_string());
            let args = delim_list_optional_strings(
                &[clock_binding, reset_binding, input_binding, output_binding],
                ",",
            );
            Ok(format!("{component_name} c{ndx}({args});"))
        })
        .collect::<Result<Vec<String>, RHDLError>>()?;
    let verilog = generate_verilog(descriptor.rtl.as_ref().unwrap())?;
    // Call the verilog function with (reset, i, q), if they exist.
    let reset_bind = Some("{{reset,clock}}".to_string());
    let i_bind = in_decl.clone().map(|_| "i".to_string());
    let q_bind = q_decls.clone().map(|_| "q".to_string());
    let iq_bind = delim_list_optional_strings(&[reset_bind, i_bind, q_bind], ",");
    let fn_call = format!("assign od = {fn_name}({iq_bind});", fn_name = verilog.name);
    let fn_body = &verilog.body;
    let o_bind = format!("assign o = od[{}:{}];", outputs.saturating_sub(1), 0);
    let d_bind = if C::D::bits() == 0 {
        None
    } else {
        Some(format!(
            "assign d = od[{}:{}];",
            (C::D::bits() + outputs).saturating_sub(1),
            outputs,
        ))
    };
    let code = format!(
        "{module_decl}
{decls}
{o_bind}

{child_decls}

{fn_call}

{fn_body}

endmodule
        
        ",
        decls = terminate_list_optional_strings(&[od_decls, d_decls, q_decls, d_bind], ";\n"),
        child_decls = child_decls.join("\n"),
    );
    Ok(HDLDescriptor {
        name: module_name.into(),
        body: code,
        children,
    })
}
