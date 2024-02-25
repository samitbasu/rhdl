use anyhow::ensure;
use anyhow::Result;
use rhdl_core::diagnostic::dfg::Component;
use rhdl_core::diagnostic::dfg::ComponentKind;
use rhdl_core::diagnostic::dfg::DFG;
use rhdl_core::CircuitIO;
use rhdl_core::Kind;
use rhdl_core::{as_verilog_literal, Digital, DigitalFn};
use rhdl_macro::Circuit;

use crate::circuit::root_descriptor;
use crate::circuit::HDLDescriptor;
use crate::circuit::{Circuit, CircuitDescriptor};

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

    const UPDATE: fn(Self::I, Self::Q) -> (Self::O, Self::D) = |_, _| (T::default(), ());

    fn sim(&self, _: Self::I, _: &mut Self::S, _: &mut Self::Z) -> Self::O {
        self.value
    }

    fn name(&self) -> &'static str {
        "Constant"
    }

    fn descriptor(&self) -> CircuitDescriptor {
        let mut desc = root_descriptor(self);
        let mut dfg = DFG::default();
        let o = dfg.graph.add_node(Component {
            input: Self::I::static_kind(),
            output: Kind::make_tuple(vec![Self::O::static_kind(), Kind::Empty]),
            kind: ComponentKind::Constant,
            location: None,
        });
        dfg.ret = o;
        desc.update_dfg = Some(dfg);
        desc
    }

    fn as_hdl(&self, kind: crate::circuit::HDLKind) -> anyhow::Result<HDLDescriptor> {
        ensure!(kind == crate::circuit::HDLKind::Verilog);
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
