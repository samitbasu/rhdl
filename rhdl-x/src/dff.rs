use anyhow::bail;
use anyhow::ensure;
use anyhow::Result;
use rhdl_core::{as_verilog_literal, Digital, DigitalFn};
use rhdl_macro::Digital;

use crate::circuit::root_descriptor;
use crate::circuit::BufZ;
use crate::circuit::HDLDescriptor;
use crate::{circuit::Circuit, clock::Clock};

#[derive(Default, Clone)]
pub struct DFF<T: Digital> {
    phantom: std::marker::PhantomData<T>,
}

#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct DFFI<T: Digital> {
    pub clock: Clock,
    pub data: T,
}

impl<T: Digital> Circuit for DFF<T> {
    type I = DFFI<T>;

    type O = T;

    type IO = ();

    type Q = ();

    type D = ();

    type Update = Self;

    const UPDATE: fn(Self::I, Self::Q) -> (Self::O, Self::D) = |i, _| (i.data, ());

    type S = DFFI<T>;

    fn sim(&self, input: Self::I, io: Self::IO, state: &mut Self::S) -> (Self::O, BufZ<()>) {
        let output = if input.clock.0 && !state.clock.0 {
            input.data
        } else {
            state.data
        };
        state.clock = input.clock;
        state.data = output;
        (output, Default::default())
    }

    fn name(&self) -> &'static str {
        "DFF"
    }

    fn as_hdl(&self, kind: crate::circuit::HDLKind) -> Result<HDLDescriptor> {
        ensure!(kind == crate::circuit::HDLKind::Verilog);
        Ok(self.as_verilog())
    }

    fn descriptor(&self) -> crate::circuit::CircuitDescriptor {
        root_descriptor(self)
    }
}

impl<T: Digital> DigitalFn for DFF<T> {
    fn kernel_fn() -> rhdl_core::KernelFnKind {
        todo!()
    }
}

impl<T: Digital> DFF<T> {
    fn as_verilog(&self) -> HDLDescriptor {
        let module_name = self.descriptor().unique_name;
        let input_bits = T::bits();
        let output_bits = T::bits().saturating_sub(1);
        let init = as_verilog_literal(&T::default().typed_bits());
        let code = format!(
            "
module {module_name}(input wire[{input_bits}:0] i, output reg[{output_bits}:0] o);
   wire clk;
   wire[{output_bits}:0] d;
   assign clk = i[0];
   assign d = i[{input_bits}:1];
   initial begin
      o = {init};
    end
    always @(posedge clk) begin
        o <= d;
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
