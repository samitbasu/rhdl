use super::circuit_impl::Circuit;
use crate::rhdl_core::ntl::from_rtl::build_ntl_from_rtl;
use crate::rhdl_core::ntl::spec::Operand;
use crate::rhdl_core::rtl::object::RegisterKind;
use crate::rhdl_core::rtl::Object;
use crate::rhdl_core::types::digital::Digital;
use crate::rhdl_core::types::path::{bit_range, Path};
use crate::rhdl_core::Kind;
use crate::rhdl_core::{compile_design, CompilationMode, RHDLError, Synchronous};
use std::collections::BTreeMap;

// A few notes on the circuit descriptor struct
// The idea here is to capture the details on the circuit in such
// a way that it can be manipulated at run time.  This means that
// information encoded in the type system must be lifted into the
// runtime description.  And the repository for that information
// is the CircuitDescriptor struct.  We cannot, for example, iterate
// over the types that make up our children.
#[derive(Clone)]
pub struct CircuitDescriptor {
    pub unique_name: String,
    pub input_kind: Kind,
    pub output_kind: Kind,
    pub d_kind: Kind,
    pub q_kind: Kind,
    pub rtl: Option<Object>,
    pub ntl: crate::core::ntl::object::Object,
    pub children: BTreeMap<String, CircuitDescriptor>,
}

// Create a flow graph of an arbitrary circuit.  The model is
//
//          +-----------------+
//          |                 |
//   *in ---> In              Out >-----*out
//          |                 |
//     +--> Q                 D >-+
//     |    |                 |   |
//     |    +-----------------+   |
//     |                          |
//     +--< Out child 0      In <-+
//     |                          |
//     +--< Out child 1      In <-+
//
// Note - we don't want to build this in the proc-macro since the less logic we
// put there, the better.  Thus, we build the flow graph when we build the
// circuit descriptor.  Because of the need to have the children present, we rely
// on them being passed in as a map, with their descriptors already built.
pub fn build_descriptor<C: Circuit>(
    name: &str,
    children: BTreeMap<String, CircuitDescriptor>,
) -> Result<CircuitDescriptor, RHDLError> {
    use crate::core::ntl;
    let module = compile_design::<C::Kernel>(CompilationMode::Asynchronous)?;
    // Build the netlist
    // First construct the netlist for the update function
    let update_netlist = build_ntl_from_rtl(&module);
    // Create a manual builder for the top level netlist
    let mut builder = ntl::builder::Builder::new(name);
    let output_kind: RegisterKind = C::O::static_kind().into();
    if output_kind.is_empty() {
        return Err(RHDLError::NoOutputsError);
    }
    let input_kind: RegisterKind = C::I::static_kind().into();
    let top_i = builder.add_input(input_kind.len());
    let top_o = builder.allocate_outputs(output_kind.len());
    let update_register_offset = builder.link(&update_netlist);
    // Link the module input to the input of the update function
    for (&top_i_bit, &update_i_bit) in top_i.iter().zip(&update_netlist.inputs[0]) {
        builder.copy_from_to(top_i_bit, update_i_bit.offset(update_register_offset));
    }
    // Link up the output bits from the update_netlist
    for (&top_o_bit, &update_o_bit) in top_o.iter().zip(&update_netlist.outputs) {
        builder.copy_from_to(update_o_bit.offset(update_register_offset), top_o_bit);
    }
    // Get the "D" vector by skipping the first |O| bits, and pre-map them into their new addresses
    let d_vec = update_netlist
        .outputs
        .iter()
        .skip(output_kind.len())
        .map(|op| op.offset(update_register_offset))
        .collect::<Vec<_>>();
    // Get the "Q" vector by remapping the 2nd input to the update function.
    // Note that the update function signature for a synchronous function is (ClockReset, I, Q) -> (O, D)
    let q_vec = update_netlist.inputs[1]
        .iter()
        .map(|op| Operand::Register(op.offset(update_register_offset)))
        .collect::<Vec<_>>();
    // Create the inputs for the children by splitting bits off of the d_index
    for (child_name, child_descriptor) in &children {
        // Compute the bit range for this child's input based on its name
        let child_path = Path::default().field(child_name);
        let (output_bit_range, _) = bit_range(C::D::static_kind(), &child_path)?;
        let (input_bit_range, _) = bit_range(C::Q::static_kind(), &child_path)?;
        // Merge the child's netlist into ours
        let child_offset = builder.link(&child_descriptor.ntl);
        // Connect the child's input registers to the given bits of the D register
        for (&d_bit, &child_i) in d_vec[output_bit_range]
            .iter()
            .zip(&child_descriptor.ntl.inputs[0])
        {
            builder.copy_from_to(d_bit, child_i.offset(child_offset));
        }
        // Connect the childs output registers to the given bits of the Q register
        for (&q_bit, &child_o) in q_vec[input_bit_range]
            .iter()
            .zip(&child_descriptor.ntl.outputs)
        {
            builder.copy_from_to(child_o.offset(child_offset), q_bit);
        }
    }
    Ok(CircuitDescriptor {
        unique_name: name.into(),
        input_kind: C::I::static_kind(),
        output_kind: C::O::static_kind(),
        d_kind: C::D::static_kind(),
        q_kind: C::Q::static_kind(),
        ntl: builder.build(ntl::builder::BuilderMode::Asynchronous)?,
        rtl: Some(module),
        children,
    })
}

// Create a flow graph of the circuit.  It is modified by adding
// a Q buffer and a D buffer.
//
//        +-----------------------------+
//        | +--------------------+      |
//        | |                    |      |
//   *rst +-> Reset              |      |
//          |                    |      |
//   *in ---> In                Out >------*out
//          |        update      |      |
//     +--> Q                   D >-+   |
//     |    |                    |  |   |
//     |    +--------------------+  |   |
//     |                            |   |
//     |                            |   |
//     |                      rst <-----+
//     +--< Out   child 0      In <-+   |
//     |                            |   |
//     |                      rst <-----+
//     +--< Out    child 1     In <-+
// Note - we don't want to build this in the proc-macro since the less logic we
// put there, the better.
pub fn build_synchronous_descriptor<C: Synchronous>(
    name: &str,
    children: BTreeMap<String, CircuitDescriptor>,
) -> Result<CircuitDescriptor, RHDLError> {
    use crate::core::ntl;
    let module = compile_design::<C::Kernel>(CompilationMode::Synchronous)?;
    // Build the netlist
    // First construct the netlist for the update function
    let update_netlist = build_ntl_from_rtl(&module);
    // Create a manual builder for the top level netlist
    let mut builder = ntl::builder::Builder::new(name);
    // This is the kind of output of the update kernel - it must be equal to
    // (Update::O, Update::D)
    // The update_fg will have 3 arguments (rst,i,q) and 2 outputs (o,d)
    let output_kind: RegisterKind = C::O::static_kind().into();
    if output_kind.is_empty() {
        return Err(RHDLError::NoOutputsError);
    }
    let input_kind: RegisterKind = C::I::static_kind().into();
    // The inputs to the circuit are [cr, I], the output is [O]
    // Allocate these as inputs to the netlist
    let top_cr = builder.add_input(2);
    let top_i = builder.add_input(input_kind.len());
    let top_o = builder.allocate_outputs(output_kind.len());
    // Link in the update code.
    let update_register_offset = builder.link(&update_netlist);
    // Link the ClockReset signal from the top down into the update code.
    for (&top_cr_bit, &update_cr_bit) in top_cr.iter().zip(&update_netlist.inputs[0]) {
        builder.copy_from_to(top_cr_bit, update_cr_bit.offset(update_register_offset));
    }
    // Link the module input to the input of the update function
    for (&top_i_bit, &update_i_bit) in top_i.iter().zip(&update_netlist.inputs[1]) {
        builder.copy_from_to(top_i_bit, update_i_bit.offset(update_register_offset));
    }
    // Link up the output bits from the update_netlist
    for (&top_o_bit, &update_o_bit) in top_o.iter().zip(&update_netlist.outputs) {
        builder.copy_from_to(update_o_bit.offset(update_register_offset), top_o_bit);
    }
    // Get the "D" vector by skipping the first |O| bits, and pre-map them into their new addresses
    let d_vec = update_netlist
        .outputs
        .iter()
        .skip(output_kind.len())
        .map(|op| op.offset(update_register_offset))
        .collect::<Vec<_>>();
    // Get the "Q" vector by remapping the 3rd input to the update function.
    // Note that the update function signature for a synchronous function is (ClockReset, I, Q) -> (O, D)
    let q_vec = update_netlist.inputs[2]
        .iter()
        .map(|op| Operand::Register(op.offset(update_register_offset)))
        .collect::<Vec<_>>();
    // Create the inputs for the children by splitting bits off of the d_index
    for (child_name, child_descriptor) in &children {
        // Compute the bit range for this child's input based on its name
        // The tuple index of .1 is to get the D element of the output from the kernel
        let child_path = Path::default().field(child_name);
        let (output_bit_range, _) = bit_range(C::D::static_kind(), &child_path)?;
        let (input_bit_range, _) = bit_range(C::Q::static_kind(), &child_path)?;
        // Merge the child's netlist into ours
        let child_offset = builder.link(&child_descriptor.ntl);
        // Connect the child's clock and reset to the top level clock and reset
        for (&top_cr, &child_cr) in top_cr.iter().zip(&child_descriptor.ntl.inputs[0]) {
            builder.copy_from_to(top_cr, child_cr.offset(child_offset));
        }
        // Connect the child's input registers to the given bits of the D register
        for (&d_bit, &child_i) in d_vec[output_bit_range]
            .iter()
            .zip(&child_descriptor.ntl.inputs[1])
        {
            builder.copy_from_to(d_bit, child_i.offset(child_offset));
        }
        // Connect the childs output registers to the given bits of the Q register
        for (&q_bit, &child_o) in q_vec[input_bit_range]
            .iter()
            .zip(&child_descriptor.ntl.outputs)
        {
            builder.copy_from_to(child_o.offset(child_offset), q_bit);
        }
    }
    Ok(CircuitDescriptor {
        unique_name: name.into(),
        input_kind: C::I::static_kind(),
        output_kind: C::O::static_kind(),
        d_kind: C::D::static_kind(),
        q_kind: C::Q::static_kind(),
        children,
        rtl: Some(module),
        ntl: builder.build(ntl::builder::BuilderMode::Synchronous)?,
    })
}
