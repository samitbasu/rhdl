use crate::{
    Circuit, CircuitDQ, CircuitIO, ClockReset, Digital, DigitalFn, Domain, HDLDescriptor, Kind,
    RHDLError, Signal, Synchronous, Timed,
    bitx::BitX,
    circuit::{circuit_descriptor::CircuitType, descriptor::Descriptor, scoped_name::ScopedName},
    digital_fn::NoKernel2,
    ntl,
    types::{kind::Field, signal::signal},
};

use quote::{format_ident, quote};
use rhdl_vlog::{self as vlog, maybe_port_wire};
use syn::parse_quote;

// An adapter allows you to use a Synchronous circuit in an Asynchronous context.
#[derive(Clone)]
pub struct Adapter<C: Synchronous, D: Domain> {
    circuit: C,
    domain: std::marker::PhantomData<D>,
}

impl<C: Synchronous, D: Domain> Adapter<C, D> {
    pub fn new(circuit: C) -> Self {
        Self {
            circuit,
            domain: Default::default(),
        }
    }
}

impl<C: Synchronous + Default, D: Domain> Default for Adapter<C, D> {
    fn default() -> Self {
        Self::new(C::default())
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct AdapterInput<I: Digital, D: Domain> {
    pub clock_reset: Signal<ClockReset, D>,
    pub input: Signal<I, D>,
}

impl<I: Digital, D: Domain> Timed for AdapterInput<I, D> {}

impl<I: Digital, D: Domain> Digital for AdapterInput<I, D> {
    const BITS: usize = <Signal<ClockReset, D> as Digital>::BITS + <Signal<I, D> as Digital>::BITS;
    fn static_kind() -> crate::Kind {
        Kind::make_struct(
            "AdapterInput",
            vec![
                Field {
                    name: "clock_reset".to_string().into(),
                    kind: <Signal<ClockReset, D> as Digital>::static_kind(),
                },
                Field {
                    name: "input".to_string().into(),
                    kind: <Signal<I, D> as Digital>::static_kind(),
                },
            ]
            .into(),
        )
    }
    fn bin(self) -> Box<[BitX]> {
        let mut out = vec![];
        out.extend(self.clock_reset.bin());
        out.extend(self.input.bin());
        out.into()
    }
    fn dont_care() -> Self {
        Self {
            clock_reset: Signal::dont_care(),
            input: Signal::dont_care(),
        }
    }
}

impl<C: Synchronous, D: Domain> CircuitIO for Adapter<C, D> {
    type I = AdapterInput<C::I, D>;
    type O = Signal<C::O, D>;
    type Kernel = NoKernel2<Self::I, (), (Self::O, ())>;
}

impl<C: Synchronous, D: Domain> CircuitDQ for Adapter<C, D> {
    type D = ();
    type Q = ();
}

impl<C: Synchronous, D: Domain> Circuit for Adapter<C, D> {
    type S = C::S;

    fn init(&self) -> Self::S {
        self.circuit.init()
    }

    fn sim(&self, input: AdapterInput<C::I, D>, state: &mut C::S) -> Signal<C::O, D> {
        let clock_reset = input.clock_reset.val();
        let input = input.input.val();
        let result = self.circuit.sim(clock_reset, input, state);
        signal(result)
    }

    fn descriptor(&self, scoped_name: ScopedName) -> Result<Descriptor, RHDLError> {
        let child_descriptor = self.circuit.descriptor(scoped_name.with("inner"))?;
        let name = scoped_name.to_string();
        Ok(Descriptor {
            name: scoped_name,
            input_kind: <<Self as CircuitIO>::I as Digital>::static_kind(),
            output_kind: <<Self as CircuitIO>::O as Digital>::static_kind(),
            d_kind: <<Self as CircuitDQ>::D as Digital>::static_kind(),
            q_kind: <<Self as CircuitDQ>::Q as Digital>::static_kind(),
            kernel: None,
            hdl: Some(self.hdl(&name, &child_descriptor)?),
            circuit_type: CircuitType::Asynchronous,
            netlist: Some(self.netlist(&name, &child_descriptor)?),
        })
    }

    fn children(
        &self,
        scoped_name: &ScopedName,
    ) -> impl Iterator<Item = Result<Descriptor, RHDLError>> {
        std::iter::once(self.circuit.descriptor(scoped_name.with("inner")))
    }
}

impl<C: Synchronous, D: Domain> DigitalFn for Adapter<C, D> {}

impl<C: Synchronous, D: Domain> Adapter<C, D> {
    fn hdl(&self, name: &str, child_descriptor: &Descriptor) -> Result<HDLDescriptor, RHDLError> {
        if child_descriptor.circuit_type != CircuitType::Synchronous {
            return Err(RHDLError::CircuitTypeMismatch {
                expected: CircuitType::Synchronous,
                found: child_descriptor.circuit_type,
                context: format!("in Adapter circuit {}", name),
            });
        }
        let ports = [
            maybe_port_wire(vlog::Direction::Input, <Self as CircuitIO>::I::bits(), "i"),
            maybe_port_wire(vlog::Direction::Output, <Self as CircuitIO>::O::bits(), "o"),
        ];
        let child_hdl = child_descriptor.hdl()?;
        let name_ident = format_ident!("{name}");
        let input_connection = if !child_descriptor.input_kind.is_empty() {
            let lsb = syn::Index::from(2);
            let msb = syn::Index::from((2 + child_descriptor.input_kind.bits()).saturating_sub(1));
            quote! {, .i(i[#msb:#lsb])}
        } else {
            quote! {}
        };
        let child_unique_name_ident = format_ident!("{}", child_descriptor.name.to_string());
        let child_modules = &child_hdl.modules;
        let module_list: vlog::ModuleList = parse_quote! {
            module #name_ident(#(#ports),*);
                #child_unique_name_ident c(.clock_reset(i[1:0]) #input_connection, .o(o))
            endmodule
            #child_modules
        };
        Ok(HDLDescriptor {
            name: name.to_string(),
            modules: module_list,
        })
    }
    fn netlist(&self, name: &str, child_descriptor: &Descriptor) -> Result<ntl::Object, RHDLError> {
        let mut builder = ntl::Builder::new(name);
        let input_reg: Kind = <<Self as CircuitIO>::I as Digital>::static_kind();
        let output_reg: Kind = <<Self as CircuitIO>::O as Digital>::static_kind();
        let ti = builder.add_input(input_reg);
        let to = builder.allocate_outputs(output_reg);
        let child_netlist = child_descriptor.netlist()?;
        if child_descriptor.circuit_type != CircuitType::Synchronous {
            return Err(RHDLError::CircuitTypeMismatch {
                expected: CircuitType::Synchronous,
                found: child_descriptor.circuit_type,
                context: format!("in Adapter circuit {} (netlist)", name),
            });
        }
        let child_offset = builder.import(child_netlist);
        let child_inputs = child_netlist.inputs.iter().flatten();
        for (&t, c) in ti.iter().zip(child_inputs) {
            builder.copy_from_to(t, child_offset(c.into()));
        }
        for (&t, c) in to.iter().zip(&child_netlist.outputs) {
            builder.copy_from_to(child_offset(*c), t);
        }
        builder.build(ntl::builder::BuilderMode::Asynchronous)
    }
}
