use anyhow::bail;
use anyhow::Result;
use rhdl_core::{as_verilog_literal, kernel::ExternalKernelDef, Digital, DigitalFn};
use rhdl_macro::Digital;

use crate::{circuit::Circuit, clock::Clock, translator::Translator};

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

    type Q = ();

    type D = ();

    type Update = Self;

    const UPDATE: fn(Self::I, Self::Q) -> (Self::O, Self::D) = |i, _| (i.data, ());

    type S = DFFI<T>;

    fn sim(&self, input: Self::I, state: &mut Self::S) -> Self::O {
        let output = if input.clock.0 && !state.clock.0 {
            input.data
        } else {
            state.data
        };
        state.clock = input.clock;
        state.data = output;
        output
    }

    fn name(&self) -> &'static str {
        "DFF"
    }

    fn components(&self) -> impl Iterator<Item = (String, crate::circuit::CircuitDescriptor)> {
        std::iter::empty()
    }

    fn translate<F: Translator>(&self, translator: &mut F) -> Result<()> {
        if translator.kind() == crate::translator::TranslationKind::Verilog {
            translator.custom_code(&self.as_verilog())
        } else {
            bail!(
                "Unsupported translation language for DFF of {:?}",
                translator.kind()
            )
        }
    }
}

impl<T: Digital> DigitalFn for DFF<T> {
    fn kernel_fn() -> rhdl_core::KernelFnKind {
        todo!()
    }
}

impl<T: Digital> DFF<T> {
    fn as_verilog(&self) -> String {
        let module_name = self.descriptor().unique_name;
        let input_bits = T::bits();
        let output_bits = T::bits().saturating_sub(1);
        let init = as_verilog_literal(&T::default().typed_bits());
        format!(
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
        )
    }
}
