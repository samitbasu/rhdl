use anyhow::ensure;
use anyhow::Result;
use rhdl_core::root_descriptor;
use rhdl_core::schematic::components::ComponentKind;
use rhdl_core::schematic::components::ConstantComponent;
use rhdl_core::schematic::schematic_impl::Schematic;
use rhdl_core::Circuit;
use rhdl_core::CircuitDescriptor;
use rhdl_core::CircuitIO;
use rhdl_core::HDLKind;
use rhdl_core::Kind;
use rhdl_core::{as_verilog_literal, Digital, DigitalFn};
use rhdl_macro::Circuit;

use rhdl_core::circuit::hdl_descriptor::HDLDescriptor;

// Constant block
#[derive(Clone)]
pub struct Constant<T: Digital> {
    value: T,
}

impl<T: Digital> CircuitIO for Constant<T> {
    type I = ();
    type O = T;
}

impl<T: Digital> From<T> for Constant<T> {
    fn from(value: T) -> Self {
        Self { value }
    }
}

impl<T: Digital> DigitalFn for Constant<T> {
    fn kernel_fn() -> Option<rhdl_core::KernelFnKind> {
        None
    }
}

impl<T: Digital> Circuit for Constant<T> {
    type Q = ();

    type D = ();

    type S = ();

    type Z = ();

    type Update = Self;

    const UPDATE: fn(Self::I, Self::Q) -> (Self::O, Self::D) = { panic!() };

    fn sim(&self, _: Self::I, _: &mut Self::S, _: &mut Self::Z) -> Self::O {
        self.value
    }

    fn name(&self) -> &'static str {
        "Constant"
    }

    fn descriptor(&self) -> CircuitDescriptor {
        let mut desc = root_descriptor(self);
        // Build a schematic with no input pin, and one output pin driven
        // by a constant component.
        let mut schematic = Schematic::default();
        let out_pin = schematic.make_pin(desc.output_kind.clone(), "out".to_string(), None);
        let constant = schematic.make_component(
            ComponentKind::Constant(ConstantComponent {
                value: self.value.typed_bits(),
                output: out_pin,
            }),
            None,
        );
        schematic.pin_mut(out_pin).parent(constant);
        schematic.output = out_pin;
        desc.update_schematic = Some(schematic);
        desc
    }

    fn as_hdl(&self, kind: HDLKind) -> anyhow::Result<HDLDescriptor> {
        ensure!(kind == HDLKind::Verilog);
        Ok(self.as_verilog())
    }
}

impl<T: Digital> Constant<T> {
    fn as_verilog(&self) -> HDLDescriptor {
        let module_name = self.descriptor().unique_name;
        let output_bits = T::bits().saturating_sub(1);
        let value = as_verilog_literal(&self.value.typed_bits());
        let body = format!(
            "
module {module_name}(input wire[0:0] i, output wire[{output_bits}:0] o);
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
