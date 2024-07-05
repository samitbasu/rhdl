use rhdl::prelude::*;

#[derive(Debug, Clone)]
pub struct U<T: Digital> {
    reset: T,
}

impl<T: Digital> U<T> {
    pub fn new(reset: T) -> Self {
        Self { reset }
    }
}

impl<T: Digital> SynchronousIO for U<T> {
    type I = T;
    type O = T;
}

impl<T: Digital> SynchronousDQ for U<T> {
    type D = ();
    type Q = ();
}

#[derive(Debug, Clone, PartialEq, Copy, Digital)]
pub struct S<T: Digital> {
    clock: Clock,
    state: T,
    state_next: T,
}

impl<T: Digital> Synchronous for U<T> {
    type Update = Self;

    type S = S<T>;

    type Z = ();

    fn sim(
        &self,
        clock: Clock,
        reset: Reset,
        input: Self::I,
        state: &mut Self::S,
        _io: &mut Self::Z,
    ) -> Self::O {
        note("input", input);
        // Calculate the new state on a rising edge
        let new_state = if clock.raw() && !state.clock.raw() {
            state.state_next
        } else {
            state.state
        };
        let new_state_next = if !clock.raw() {
            input
        } else {
            state.state_next
        };
        if reset.raw() {
            state.state = self.reset;
            state.state_next = self.reset;
        } else {
            state.state = new_state;
            state.state_next = new_state_next;
        }
        state.clock = clock;
        note("output", new_state);
        new_state
    }

    fn name(&self) -> &'static str {
        "DFF"
    }

    fn as_hdl(&self, _: HDLKind) -> Result<HDLDescriptor, RHDLError> {
        Ok(self.as_verilog())
    }
}

impl<T: Digital> U<T> {
    fn as_verilog(&self) -> HDLDescriptor {
        let module_name = self.descriptor().unique_name;
        let input_bits = T::bits();
        let output_bits = T::bits();
        let init = rhdl::core::as_verilog_literal(&self.reset.typed_bits());
        let input_wire = rhdl::core::codegen::verilog::as_verilog_decl("wire", input_bits, "i");
        let output_reg = rhdl::core::codegen::verilog::as_verilog_decl("reg", output_bits, "o");
        let code = format!(
            "
module {module_name}(input clock, input reset, input {input_wire}, output {output_reg});
   initial begin
        o = {init};
   end
   always @(posedge clock) begin 
        if (reset) begin
            o <= {init};
        end else begin
            o <= i;
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

impl<T: Digital> DigitalFn for U<T> {}
