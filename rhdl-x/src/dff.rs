use anyhow::ensure;
use rhdl::prelude::*;

#[derive(PartialEq, Clone, Copy, Debug, Digital)]
pub struct I<T: Digital> {
    pub data: T,
    pub clock: Clock,
    pub reset: Reset,
}

// The DFF itself has a reset value, and a clock domain.
#[derive(Debug, Clone)]
pub struct U<T: Digital, C: Domain> {
    reset: T,
    clock: std::marker::PhantomData<C>,
}

impl<T: Digital, C: Domain> U<T, C> {
    pub fn new(reset: T) -> Self {
        Self {
            reset,
            clock: Default::default(),
        }
    }
}

impl<T: Digital, C: Domain> CircuitIO for U<T, C> {
    type I = Signal<I<T>, C>;
    type O = Signal<T, C>;
}

#[derive(Debug, Clone, PartialEq, Copy, Digital)]
pub struct S<T: Digital> {
    clock: Clock,
    state: T,
    state_next: T,
}

impl<T: Digital, C: Domain> Circuit for U<T, C> {
    type Q = ();
    type D = ();
    type Z = ();

    type Update = Self;

    type S = S<T>;

    fn sim(&self, input: Self::I, state: &mut Self::S, _io: &mut Self::Z) -> Self::O {
        note("input", input);
        // Calculate the new state on a rising edge
        let new_state = if input.val().clock.raw() && !state.clock.raw() {
            state.state_next
        } else {
            state.state
        };
        let new_state_next = if !input.val().clock.raw() {
            input.val().data
        } else {
            state.state_next
        };
        if input.val().reset.raw() {
            state.state = self.reset;
            state.state_next = self.reset;
        } else {
            state.state = new_state;
            state.state_next = new_state_next;
        }
        state.clock = input.val().clock;
        note("output", new_state);
        signal(new_state)
    }

    fn name(&self) -> &'static str {
        "DFF"
    }

    fn as_hdl(&self, kind: HDLKind) -> anyhow::Result<HDLDescriptor> {
        ensure!(kind == HDLKind::Verilog);
        Ok(self.as_verilog())
    }

    fn descriptor(&self) -> CircuitDescriptor {
        root_descriptor(self)
    }
}

impl<T: Digital, C: Domain> U<T, C> {
    fn as_verilog(&self) -> HDLDescriptor {
        let module_name = self.descriptor().unique_name;
        let input_bits = I::<T>::bits();
        let output_bits = T::bits();
        let init = rhdl::core::as_verilog_literal(&self.reset.typed_bits());
        let input_wire = rhdl::core::codegen::verilog::as_verilog_decl("wire", input_bits, "i");
        let output_reg = rhdl::core::codegen::verilog::as_verilog_decl("reg", output_bits, "o");
        let d = rhdl::core::codegen::verilog::as_verilog_decl("wire", T::bits(), "d");
        let code = format!(
            "
module {module_name}(input {input_wire}, output {output_reg});
   wire clk;
   wire rst;
   wire {d};
   assign {{rst, clk, d}} = i;
   initial begin
        o = {init};
   end
   always @(posedge clk) begin 
        if (rst) begin
            o <= {init};
        end else begin
            o <= {d};
        end
    end
endmodule
            "
        );
        HDLDescriptor {
            name: module_name,
            body: code,
            children: Default::default(),
        }
    }
}

impl<T: Digital, C: Domain> DigitalFn for U<T, C> {}
