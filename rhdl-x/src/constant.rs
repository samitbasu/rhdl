use rhdl_core::{as_verilog_literal, kernel::ExternalKernelDef, Digital, DigitalFn};

use crate::circuit::{Circuit, CircuitDescriptor};

// Constant block
#[derive(Clone)]
pub struct Constant<T: Digital> {
    value: T,
}

impl<T: Digital> From<T> for Constant<T> {
    fn from(value: T) -> Self {
        Self { value }
    }
}

impl<T: Digital> DigitalFn for Constant<T> {
    fn kernel_fn() -> rhdl_core::KernelFnKind {
        rhdl_core::KernelFnKind::Extern(ExternalKernelDef {
            name: todo!(),
            body: todo!(),
            vm_stub: todo!(),
        })
    }
}

impl<T: Digital> Circuit for Constant<T> {
    type I = ();

    type O = T;

    type Q = ();

    type D = ();

    type S = ();

    type Update = Self;

    const UPDATE: fn(Self::I, Self::Q) -> (Self::O, Self::D) = |_, _| (T::default(), ());

    fn sim(&self, _: Self::I, _: &mut Self::S) -> Self::O {
        self.value
    }

    fn name(&self) -> &'static str {
        "Constant"
    }

    fn components(&self) -> impl Iterator<Item = (String, CircuitDescriptor)> {
        std::iter::empty()
    }

    fn verilog(self) -> Result<String, anyhow::Error> {
        let module_name = self.descriptor().unique_name;
        let output_bits = T::bits().saturating_sub(1);
        let value = as_verilog_literal(&self.value.typed_bits());
        Ok(format!(
            "module {module_name}(input wire[0:0] i, output wire[{output_bits}:0] o);
  assign o = {value};
  endmodule
  "
        ))
    }
}