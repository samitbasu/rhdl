use super::circuit_impl::Circuit;
use crate::Kind;
use crate::ntl::from_rtl::build_ntl_from_rtl;
use crate::rtl::Object;
use crate::types::digital::Digital;
use crate::types::path::{Path, bit_range};
use crate::{CompilationMode, RHDLError, Synchronous, compile_design};
use std::collections::BTreeMap;

/// A runtime description of a circuit.
///
/// # Notes
/// A few notes on the circuit descriptor struct
/// The idea here is to capture the details on the circuit in such
/// a way that it can be manipulated at run time.  This means that
/// information encoded in the type system must be lifted into the
/// runtime description.  And the repository for that information
/// is the CircuitDescriptor struct.  We cannot, for example, iterate
/// over the types that make up our children.  A reflection mechanism
/// like [facet](https://facet.rs/) would help here, but did not
/// exist when this was written.
#[derive(Clone)]
pub struct CircuitDescriptor {
    /// A unique name for the circuit type.
    pub unique_name: String,
    /// The [Kind] of the input to the circuit.
    pub input_kind: Kind,
    /// The [Kind] of the output from the circuit.
    pub output_kind: Kind,
    /// The [Kind] of the D (data) signals in the circuit.
    pub d_kind: Kind,
    /// The [Kind] of the Q (control) signals in the circuit.
    pub q_kind: Kind,
    /// The RTL representation of the circuit, if available.
    pub rtl: Option<Object>,
    /// The netlist representation of the circuit.
    pub ntl: crate::ntl::object::Object,
    /// The child circuits of this circuit, if any.
    pub children: BTreeMap<String, CircuitDescriptor>,
}

/// Build a circuit descriptor for an asynchronous circuit.
///
/// Not a function you will typically call directly.  This function
/// is used by the `Circuit` proc-macro to build the descriptor
/// for a circuit.
pub fn build_descriptor<C: Circuit>(
    name: &str,
    children: BTreeMap<String, CircuitDescriptor>,
) -> Result<CircuitDescriptor, RHDLError> {
    use crate::ntl;
    let module = compile_design::<C::Kernel>(CompilationMode::Asynchronous)?;
    // Build the netlist
    // First construct the netlist for the update function
    let update_netlist = build_ntl_from_rtl(&module);
    // Create a manual builder for the top level netlist
    let mut builder = ntl::builder::Builder::new(name);
    let output_kind: Kind = C::O::static_kind();
    if output_kind.is_empty() {
        return Err(RHDLError::NoOutputsError);
    }
    let input_kind: Kind = C::I::static_kind();
    let top_i = builder.add_input(input_kind);
    let top_o = builder.allocate_outputs(output_kind);
    let update_register_offset = builder.import(&update_netlist);
    // Link the module input to the input of the update function
    for (&top_i_bit, &update_i_bit) in top_i.iter().zip(&update_netlist.inputs[0]) {
        builder.copy_from_to(top_i_bit, update_register_offset(update_i_bit.into()));
    }
    // Link up the output bits from the update_netlist
    for (&top_o_bit, &update_o_bit) in top_o.iter().zip(&update_netlist.outputs) {
        builder.copy_from_to(update_register_offset(update_o_bit), top_o_bit);
    }
    // Get the "D" vector by skipping the first |O| bits, and pre-map them into their new addresses
    let d_vec = update_netlist
        .outputs
        .iter()
        .skip(output_kind.bits())
        .map(|op| update_register_offset(*op))
        .collect::<Vec<_>>();
    // Get the "Q" vector by remapping the 2nd input to the update function.
    // Note that the update function signature for a synchronous function is (ClockReset, I, Q) -> (O, D)
    let q_vec = update_netlist.inputs[1]
        .iter()
        .map(|op| update_register_offset(op.into()))
        .collect::<Vec<_>>();
    // Create the inputs for the children by splitting bits off of the d_index
    for (child_name, child_descriptor) in &children {
        // Compute the bit range for this child's input based on its name
        let child_path = Path::default().field(child_name);
        let (output_bit_range, _) = bit_range(C::D::static_kind(), &child_path)?;
        let (input_bit_range, _) = bit_range(C::Q::static_kind(), &child_path)?;
        // Merge the child's netlist into ours
        let child_offset = builder.import(&child_descriptor.ntl);
        // Connect the child's input registers to the given bits of the D register
        for (&d_bit, child_i) in d_vec[output_bit_range.clone()]
            .iter()
            .zip(&child_descriptor.ntl.inputs[0])
        {
            builder.copy_from_to(d_bit, child_offset(child_i.into()));
        }
        // Connect the childs output registers to the given bits of the Q register
        for (&q_bit, &child_o) in q_vec[input_bit_range.clone()]
            .iter()
            .zip(&child_descriptor.ntl.outputs)
        {
            builder.copy_from_to(child_offset(child_o), q_bit);
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

/// Build a circuit descriptor for a synchronous circuit.
///
/// Not a function you will typically call directly.  This function
/// is used by the `Synchronous` proc-macro to build the descriptor
/// for a circuit.
pub fn build_synchronous_descriptor<C: Synchronous>(
    name: &str,
    children: BTreeMap<String, CircuitDescriptor>,
) -> Result<CircuitDescriptor, RHDLError> {
    use crate::ntl;
    let module = compile_design::<C::Kernel>(CompilationMode::Synchronous)?;
    // Build the netlist
    // First construct the netlist for the update function
    let update_netlist = build_ntl_from_rtl(&module);
    // Create a manual builder for the top level netlist
    let mut builder = ntl::builder::Builder::new(name);
    // This is the kind of output of the update kernel - it must be equal to
    // (Update::O, Update::D)
    // The update_fg will have 3 arguments (rst,i,q) and 2 outputs (o,d)
    let output_kind: Kind = C::O::static_kind();
    if output_kind.is_empty() {
        return Err(RHDLError::NoOutputsError);
    }
    let input_kind: Kind = C::I::static_kind();
    // The inputs to the circuit are [cr, I], the output is [O]
    // Allocate these as inputs to the netlist
    let top_cr = builder.add_input(crate::ClockReset::static_kind());
    let top_i = builder.add_input(input_kind);
    let top_o = builder.allocate_outputs(output_kind);
    // Link in the update code.
    let update_register_offset = builder.import(&update_netlist);
    // Link the ClockReset signal from the top down into the update code.
    for (&top_cr_bit, &update_cr_bit) in top_cr.iter().zip(&update_netlist.inputs[0]) {
        builder.copy_from_to(top_cr_bit, update_register_offset(update_cr_bit.into()));
    }
    // Link the module input to the input of the update function
    for (&top_i_bit, update_i_bit) in top_i.iter().zip(&update_netlist.inputs[1]) {
        builder.copy_from_to(top_i_bit, update_register_offset(update_i_bit.into()));
    }
    // Link up the output bits from the update_netlist
    for (&top_o_bit, &update_o_bit) in top_o.iter().zip(&update_netlist.outputs) {
        builder.copy_from_to(update_register_offset(update_o_bit), top_o_bit);
    }
    // Get the "D" vector by skipping the first |O| bits, and pre-map them into their new addresses
    let d_vec = update_netlist
        .outputs
        .iter()
        .skip(output_kind.bits())
        .map(|op| update_register_offset(*op))
        .collect::<Vec<_>>();
    // Get the "Q" vector by remapping the 3rd input to the update function.
    // Note that the update function signature for a synchronous function is (ClockReset, I, Q) -> (O, D)
    let q_vec = update_netlist.inputs[2]
        .iter()
        .map(|op| update_register_offset(op.into()))
        .collect::<Vec<_>>();
    // Create the inputs for the children by splitting bits off of the d_index
    for (child_name, child_descriptor) in &children {
        // Compute the bit range for this child's input based on its name
        // The tuple index of .1 is to get the D element of the output from the kernel
        let child_path = Path::default().field(child_name);
        let (output_bit_range, _) = bit_range(C::D::static_kind(), &child_path)?;
        let (input_bit_range, _) = bit_range(C::Q::static_kind(), &child_path)?;
        // Merge the child's netlist into ours
        let child_offset = builder.import(&child_descriptor.ntl);
        log::debug!("Link child {child_name} into descriptor for {name}");
        // Connect the child's clock and reset to the top level clock and reset
        for (&top_cr, &child_cr) in top_cr.iter().zip(&child_descriptor.ntl.inputs[0]) {
            builder.copy_from_to(top_cr, child_offset(child_cr.into()));
        }
        // Connect the child's input registers to the given bits of the D register
        for (&d_bit, &child_i) in d_vec[output_bit_range.clone()]
            .iter()
            .zip(&child_descriptor.ntl.inputs[1])
        {
            builder.copy_from_to(d_bit, child_offset(child_i.into()));
        }
        // Connect the childs output registers to the given bits of the Q register
        for (&q_bit, &child_o) in q_vec[input_bit_range.clone()]
            .iter()
            .zip(&child_descriptor.ntl.outputs)
        {
            builder.copy_from_to(child_offset(child_o), q_bit);
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
