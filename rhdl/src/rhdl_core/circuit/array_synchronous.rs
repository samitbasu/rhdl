use std::collections::BTreeMap;

use crate::rhdl_core::{
    digital_fn::NoKernel3,
    hdl::ast::{component_instance, connection, id, index, Direction, Module, Statement},
    ntl,
    rtl::object::RegisterKind,
    trace_pop_path, trace_push_path,
    types::path::{bit_range, Path},
    CircuitDescriptor, ClockReset, Digital, HDLDescriptor, Kind, RHDLError, Synchronous,
    SynchronousDQ, SynchronousIO,
};

use super::hdl_backend::maybe_port_wire;

// Blanket implementation for an array of synchronous circuits.
impl<T: SynchronousIO, const N: usize> SynchronousIO for [T; N] {
    type I = [T::I; N];
    type O = [T::O; N];
    type Kernel = NoKernel3<ClockReset, Self::I, Self::Q, (Self::O, Self::D)>;
}

impl<T: SynchronousIO, const N: usize> SynchronousDQ for [T; N] {
    type D = ();
    type Q = ();
}

const ARRAY_ENTRIES: &[&str] = &[
    "[0]", "[1]", "[2]", "[3]", "[4]", "[5]", "[6]", "[7]", "[8]", "[9]", "[10]", "[11]", "[12]",
    "[13]", "[14]", "[15]", "[XX]",
];

impl<T: Synchronous, const N: usize> Synchronous for [T; N] {
    type S = [T::S; N];

    fn init(&self) -> Self::S {
        array_init::array_init(|i| self[i].init())
    }

    fn sim(&self, clock_reset: ClockReset, input: Self::I, state: &mut Self::S) -> Self::O {
        trace_push_path("array");
        let mut output = [T::O::dont_care(); N];
        for i in 0..N {
            trace_push_path(ARRAY_ENTRIES[i.min(16)]);
            output[i] = self[i].sim(clock_reset, input[i], &mut state[i]);
            trace_pop_path();
        }
        trace_pop_path();
        output
    }

    fn description(&self) -> String {
        format!("array of {} x {}", N, self[0].description())
    }

    // This requires a custom implementation because the default implementation
    // assumes that the children of the current circuit are named with field names
    // as part of a struct.
    fn descriptor(
        &self,
        name: &str,
    ) -> Result<crate::rhdl_core::CircuitDescriptor, crate::rhdl_core::RHDLError> {
        let mut builder = ntl::Builder::new(name);
        let cr_kind: RegisterKind = ClockReset::static_kind().into();
        let input_kind: RegisterKind = Self::I::static_kind().into();
        let output_kind: RegisterKind = Self::O::static_kind().into();
        let tcr = builder.add_input(cr_kind.len());
        let ti = builder.add_input(input_kind.len());
        let to = builder.allocate_outputs(output_kind.len());
        let mut children = std::collections::BTreeMap::default();
        for i in 0..N {
            let child_path = Path::default().index(i);
            let (output_bit_range, _) = bit_range(Self::O::static_kind(), &child_path)?;
            let (input_bit_range, _) = bit_range(Self::I::static_kind(), &child_path)?;
            let child_name = format!("{}_{}", name, i);
            let child_desc = self[i].descriptor(&child_name)?;
            let offset = builder.import(&child_desc.ntl);
            for (&t, c) in tcr.iter().zip(&child_desc.ntl.inputs[0]) {
                builder.copy_from_to(t, c.offset(offset));
            }
            for (&t, c) in ti[input_bit_range].iter().zip(&child_desc.ntl.inputs[1]) {
                builder.copy_from_to(t, c.offset(offset));
            }
            for (&t, c) in to[output_bit_range].iter().zip(&child_desc.ntl.outputs) {
                builder.copy_from_to(c.offset(offset), t);
            }
            children.insert(child_name, child_desc);
        }
        Ok(CircuitDescriptor {
            unique_name: name.into(),
            input_kind: Self::I::static_kind(),
            output_kind: Self::O::static_kind(),
            d_kind: Kind::Empty,
            q_kind: Kind::Empty,
            ntl: builder.build(ntl::builder::BuilderMode::Synchronous)?,
            rtl: None,
            children,
        })
    }

    fn hdl(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        let descriptor = self.descriptor(name)?;
        let module_name = &descriptor.unique_name;
        let mut module = Module {
            name: module_name.clone(),
            description: self.description(),
            ..Default::default()
        };

        let children = (0..N)
            .map(|ndx| {
                let name = format!("{}_{}", name, ndx);
                let hdl = self[ndx].hdl(&name)?;
                Ok((name, hdl))
            })
            .collect::<Result<BTreeMap<String, HDLDescriptor>, RHDLError>>()?;
        module.ports = [
            maybe_port_wire(Direction::Input, 2, "clock_reset"),
            maybe_port_wire(Direction::Input, Self::I::bits(), "i"),
            maybe_port_wire(Direction::Output, Self::O::bits(), "o"),
        ]
        .into_iter()
        .flatten()
        .collect();
        let i_kind = Self::I::static_kind();
        let o_kind = Self::O::static_kind();
        let child_decls = descriptor
            .children
            .iter()
            .enumerate()
            .map(|(ndx, (_, descriptor))| {
                let child_path = Path::default().index(ndx);
                let (i_range, _) = bit_range(i_kind, &child_path)?;
                let (o_range, _) = bit_range(o_kind, &child_path)?;
                let cr_binding = Some(connection("clock_reset", id("clock_reset")));
                let input_binding =
                    (!i_range.is_empty()).then(|| connection("i", index("i", i_range.clone())));
                let output_binding =
                    (!o_range.is_empty()).then(|| connection("o", index("o", o_range.clone())));
                let component_name = &descriptor.unique_name;
                Ok(component_instance(
                    component_name,
                    &format!("c{ndx}"),
                    [cr_binding, input_binding, output_binding]
                        .into_iter()
                        .flatten()
                        .collect(),
                ))
            })
            .collect::<Result<Vec<Statement>, RHDLError>>()?;
        module.statements.extend(child_decls);
        Ok(HDLDescriptor {
            name: module_name.into(),
            body: module,
            children,
        })
    }
}
