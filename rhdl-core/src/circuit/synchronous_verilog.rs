use crate::{
    compile_design,
    compiler::{codegen::verilog::generate_verilog, driver::CompilationMode},
    error::RHDLError,
    types::{
        digital::Digital,
        path::{bit_range, Path},
    },
    CircuitDescriptor, HDLDescriptor, Synchronous,
};

type Result<T> = std::result::Result<T, RHDLError>;

pub fn root_synchronous_verilog<S: Synchronous>(t: &S) -> Result<HDLDescriptor> {
    // Start with the module declaration for the circuit.
    let descriptor = t.descriptor()?;
    let input_bits = S::I::bits();
    let outputs = S::O::bits();

    let module_name = &descriptor.unique_name;
    // module top(input wire clk, input wire[0:0] top_in, output reg[3:0] top_out);

    let module_decl = format!(
        "module {module_name}(input clock, input reset, input wire[{INPUT_BITS}:0] i, output wire[{OUTPUT_BITS}:0] o);",
        module_name = module_name,
        INPUT_BITS = input_bits.saturating_sub(1),
        OUTPUT_BITS = outputs.saturating_sub(1)
    );

    let o_d_bits = S::O::bits() + S::D::bits();
    // Next declare the D and Q wires
    let od_decl = format!(
        "wire[{OD_BITS}:0] od;",
        OD_BITS = o_d_bits.saturating_sub(1)
    );
    let d_decl = format!(
        "wire[{D_BITS}:0] d;",
        D_BITS = S::D::bits().saturating_sub(1)
    );
    let q_decl = format!(
        "wire[{Q_BITS}:0] q;",
        Q_BITS = S::Q::bits().saturating_sub(1)
    );

    // Next, for each sub-component, we need to determine it's input range from the Q and D types.
    // Loop over the components.
    let component_decls = descriptor
        .children
        .iter()
        .enumerate()
        .map(|(ndx, (local, desc))| component_decl::<S>(ndx, local, desc))
        .collect::<Result<Vec<_>>>()?
        .join("\n");
    let design = compile_design::<S::Update>(CompilationMode::Synchronous)?;
    let verilog = generate_verilog(&design)?;
    let fn_call = format!(
        "assign od = {fn_name}(reset, i, q);",
        fn_name = &verilog.name
    );
    let fn_body = &verilog.body;
    let o_bind = format!("assign o = od[{}:{}];", outputs.saturating_sub(1), 0);
    let d_bind = format!(
        "assign d = od[{}:{}];",
        S::D::bits().saturating_sub(1),
        outputs
    );
    let code = format!(
        "{module_decl}
{od_decl}
{d_decl}
{q_decl}
{o_bind}
{d_bind}

{component_decls}

{fn_call}

{fn_body}

endmodule

",
    );
    Ok(HDLDescriptor {
        name: module_name.into(),
        body: code,
        children: Default::default(),
    })
}

fn component_decl<S: Synchronous>(
    ndx: usize,
    local_name: &str,
    desc: &CircuitDescriptor,
) -> Result<String> {
    // instantiate the component with name components.name.
    // give it a unique instance name of c{ndx}
    // wire the inputs to the range of d that corresponds to the name
    // wire the outputs to the range of q that corresponds to the name
    let d_kind = S::D::static_kind();
    let q_kind = S::Q::static_kind();
    eprintln!("d_kind: {:?}", d_kind);
    eprintln!("q_kind: {:?}", q_kind);
    eprintln!("local_name: {local_name}");
    let (d_range, _) = bit_range(d_kind, &Path::default().field(local_name))?;
    let (q_range, _) = bit_range(q_kind, &Path::default().field(local_name))?;
    Ok(format!(
        "{component_name} c{ndx} (.clock(clock),.reset(reset),.i(d[{d_msb}:{d_lsb}]),.o(q[{q_msb}:{q_lsb}]));",
        component_name = desc.unique_name,
        ndx = ndx,
        d_msb = d_range.end.saturating_sub(1),
        d_lsb = d_range.start,
        q_msb = q_range.end.saturating_sub(1),
        q_lsb = q_range.start
    ))
}
