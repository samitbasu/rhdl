use anyhow::ensure;
use anyhow::Result;
use rhdl_core::clk;
use rhdl_core::constraint_input_synchronous;
use rhdl_core::constraint_must_clock;
use rhdl_core::constraint_not_constant_valued;
use rhdl_core::constraint_output_synchronous;
use rhdl_core::note;
use rhdl_core::path::Path;
use rhdl_core::root_descriptor;
use rhdl_core::schematic::components::BlackBoxComponent;
use rhdl_core::schematic::components::ComponentKind;
use rhdl_core::schematic::components::IndexComponent;
use rhdl_core::schematic::schematic_impl::pin_path;
use rhdl_core::schematic::schematic_impl::PinIx;
use rhdl_core::schematic::schematic_impl::Schematic;
use rhdl_core::types::timed::signal;
use rhdl_core::BlackBoxTrait;
use rhdl_core::Circuit;
use rhdl_core::CircuitDescriptor;
use rhdl_core::CircuitIO;
use rhdl_core::Clk;
use rhdl_core::Clock;
use rhdl_core::Constraint;
use rhdl_core::EdgeType;
use rhdl_core::HDLDescriptor;
use rhdl_core::HDLKind;
use rhdl_core::Kind;
use rhdl_core::Notable;
use rhdl_core::Sig;
use rhdl_core::Timed;
use rhdl_core::{as_verilog_literal, Digital, DigitalFn};
use rhdl_macro::Timed;

#[derive(Default, Clone)]
pub struct DFF<T: Digital, C: Clock> {
    init: T,
    clock: std::marker::PhantomData<C>,
}

#[derive(Clone, Debug)]
pub struct DigitalFlipFlopComponent {
    clock: PinIx,
    d: PinIx,
    q: PinIx,
}

impl BlackBoxTrait for DigitalFlipFlopComponent {
    fn name(&self) -> &'static str {
        "DFF"
    }

    fn args(&self) -> Vec<PinIx> {
        vec![self.clock, self.d]
    }

    fn output(&self) -> PinIx {
        self.q
    }

    fn offset(&self, shift: usize) -> BlackBoxComponent {
        BlackBoxComponent::new(DigitalFlipFlopComponent {
            clock: self.clock.offset(shift),
            d: self.d.offset(shift),
            q: self.q.offset(shift),
        })
    }

    fn constraints(&self) -> Vec<Constraint> {
        vec![
            constraint_must_clock(pin_path(self.clock, Path::default())),
            constraint_not_constant_valued(pin_path(self.d, Path::default())),
            constraint_output_synchronous(
                pin_path(self.q, Path::default()),
                pin_path(self.clock, Path::default()),
                EdgeType::Positive,
            ),
            constraint_input_synchronous(
                pin_path(self.d, Path::default()),
                pin_path(self.clock, Path::default()),
                EdgeType::Positive,
            ),
        ]
    }
}

#[derive(PartialEq, Clone, Copy, Debug, Timed)]
struct DFFI<T: Digital + Default, C: Clock> {
    data: Sig<T, C>,
    clock: Sig<Clk, C>,
}

impl<T: Digital + Default, C: Clock> Default for DFFI<T, C> {
    fn default() -> Self {
        Self {
            data: signal(Default::default()),
            clock: signal(clk(false)),
        }
    }
}

impl<T: Digital + Default, C: Clock> CircuitIO for DFF<T, C> {
    type I = DFFI<T, C>;
    type O = Sig<T, C>;
}

// TODO - remove this--v
impl<T: Digital + Default, C: Clock> Circuit for DFF<T, C> {
    type Q = ();

    type D = ();

    type Z = ();

    type Update = Self;

    const UPDATE: fn(Self::I, Self::Q) -> (Self::O, Self::D) = |i, _| (i.data, ());

    type S = DFFI<T, C>;

    fn init_state(&self) -> Self::S {
        Self::S::default()
    }

    fn sim(&self, input: Self::I, state: &mut Self::S, _io: &mut Self::Z) -> Self::O {
        note("input", input);
        let output = if input.clock.val().raw() && !state.clock.val().raw() {
            input.data
        } else {
            state.data
        };
        state.data = output;
        state.clock = input.clock;
        note("output", output);
        output
    }

    fn name(&self) -> &'static str {
        "DFF"
    }

    fn as_hdl(&self, kind: HDLKind) -> Result<HDLDescriptor> {
        ensure!(kind == HDLKind::Verilog);
        Ok(self.as_verilog())
    }

    fn descriptor(&self) -> CircuitDescriptor {
        let mut desc = root_descriptor(self);
        /*
        let mut schematic = Schematic::default();
        let (input_rx, input_tx) = schematic.make_buffer(DFFI::<T>::static_kind(), None);
        // The clock splitter
        let i = schematic.make_pin(DFFI::<T>::static_kind(), "i".to_string(), None);
        let clock = schematic.make_pin(Clock::static_kind(), "clock".to_string(), None);
        let index = schematic.make_component(
            ComponentKind::Index(IndexComponent {
                arg: i,
                path: Path::default().field("clock"),
                output: clock,
                kind: Clock::static_kind(),
                dynamic: vec![],
            }),
            None,
        );
        schematic.pin_mut(i).parent(index);
        schematic.pin_mut(clock).parent(index);
        schematic.wire(input_tx, i);
        // the D splitter
        let i = schematic.make_pin(DFFI::<T>::static_kind(), "i".to_string(), None);
        let d_pin = schematic.make_pin(T::static_kind(), "d".to_string(), None);
        let index = schematic.make_component(
            ComponentKind::Index(IndexComponent {
                arg: i,
                path: Path::default().field("data"),
                output: d_pin,
                kind: T::static_kind(),
                dynamic: vec![],
            }),
            None,
        );
        schematic.pin_mut(i).parent(index);
        schematic.pin_mut(d_pin).parent(index);
        schematic.wire(input_tx, i);
        // The DFF itself
        let c = schematic.make_pin(Clock::static_kind(), "clock".to_string(), None);
        let d = schematic.make_pin(T::static_kind(), "d".to_string(), None);
        let q = schematic.make_pin(T::static_kind(), "q".to_string(), None);
        let dff = schematic.make_component(
            ComponentKind::BlackBox(BlackBoxComponent::new(DigitalFlipFlopComponent {
                clock: c,
                d,
                q,
            })),
            None,
        );
        schematic.pin_mut(c).parent(dff);
        schematic.pin_mut(d).parent(dff);
        schematic.pin_mut(q).parent(dff);
        schematic.wire(clock, c);
        schematic.wire(d_pin, d);
        schematic.inputs = vec![input_rx];
        schematic.output = q;
        desc.update_schematic = Some(schematic);
        */
        desc
    }
}

impl<T: Digital, C: Clock> DigitalFn for DFF<T, C> {
    fn kernel_fn() -> Option<rhdl_core::KernelFnKind> {
        None
    }
}

impl<T: Digital + Default, C: Clock> DFF<T, C> {
    fn as_verilog(&self) -> HDLDescriptor {
        let module_name = self.descriptor().unique_name;
        let input_bits = T::bits();
        let output_bits = T::bits().saturating_sub(1);
        let init = as_verilog_literal(&self.init.typed_bits());
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
