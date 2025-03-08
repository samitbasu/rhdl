use rhdl::prelude::*;

#[derive(Clone)]
struct U<T: Timed> {
    marker: std::marker::PhantomData<T>,
}

impl<T: Timed> CircuitIO for U<T> {
    type I = T;
    type O = ();
    type Kernel = NoKernel2<T, (), ((), ())>;
}

impl<T: Timed> CircuitDQ for U<T> {
    type D = ();
    type Q = ();
}

impl<T: Timed> Circuit for U<T> {
    type S = ();

    fn init(&self) -> Self::S {
        unimplemented!()
    }

    fn description(&self) -> String {
        format!("Output buffer for type {:?}", <T as Timed>::static_kind())
    }

    fn sim(&self, input: Self::I, state: &mut Self::S) -> Self::O {
        unimplemented!()
    }

    fn descriptor(&self, name: &str) -> Result<CircuitDescriptor, RHDLError> {
        unimplemented!()
    }

    fn hdl(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        todo!()
    }
}

/*
  Ultimately, the top level thing should look like this:

  struct Thing {
    ports: BTreeMap<String, Port>, // Direction, width
    io_cores: Vec<Drivers>
    circuit: impl Circuit,
    io_core_map: WiringPlan,
    constraints: Vec<Constraints>
  }

In this case, a Core is simply a fragment of Verilog.  Something
like:

OBUF #(
.DRIVE(12)
.IOSTANDARD("DEFAULT")
.SLEW("SLOW")
)
<name> (
.O(O)
.I(I)
);

These live at the top of the design hierarchy.  Which means
we need to define the top level globals _at this level_.
This could all be typed or untyped.

In Verilog form it will be:

// All ports are either input, output or inout
// There is a width, and each is a wire.
module top(input wire port_a, output wire port_b,..., port_c);

  wire circuit_inputs[many];
  wire circuit_outputs[many];

  bufg1 blah_0(.O(port_b[3]), I(circuit_outputs[67]));


  my_thing my_thing_inst(circuit_inputs, circuit_outputs);
endmodule

// We can check -
// All inputs are covered for my circuit
// No outputs are double-driven (two sources for one output)

// We can require that:
//  All references are of the form
//     name[bit]
//
//  That inputs can only come from output ports, and
//  that outputs can only go to input ports.
// Drivers should support templating (maybe via tiny template)
// The side of the driver that faces the circuit should be
// typed.
//
*/
