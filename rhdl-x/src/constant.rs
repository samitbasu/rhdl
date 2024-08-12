use anyhow::ensure;
use rhdl::prelude::*;

#[derive(Clone, Debug)]
pub struct U<T: Digital> {
    value: T,
}

impl<T: Digital> U<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T: Digital> SynchronousIO for U<T> {
    type I = ();
    type O = T;
}

impl<T: Digital> SynchronousDQ for U<T> {
    type D = ();
    type Q = ();
}

impl<T: Digital> Synchronous for U<T> {
    type Update = Self;

    type S = ();

    type Z = ();

    fn sim(
        &self,
        _clock: Clock,
        _reset: Reset,
        _input: Self::I,
        _state: &mut Self::S,
        _io: &mut Self::Z,
    ) -> Self::O {
        self.value
    }

    fn name(&self) -> &'static str {
        "Constant"
    }

    fn as_hdl(&self, kind: HDLKind) -> Result<HDLDescriptor, RHDLError> {
        Ok(self.as_verilog())
    }
}

impl<T: Digital> DigitalFn for U<T> {}

impl<T: Digital> U<T> {
    fn as_verilog(&self) -> HDLDescriptor {
        let module_name = self.descriptor().unique_name;
        let output_bits = T::bits().saturating_sub(1);
        let value = self.value.typed_bits().as_verilog_literal();
        let body = format!(
            "
module {module_name}(input clock, input reset, input wire[0:0] i, output wire[{output_bits}:0] o);
    assign o = {value};
endmodule
"
        );
        HDLDescriptor {
            name: module_name,
            body,
            children: Default::default(),
        }
    }
}
