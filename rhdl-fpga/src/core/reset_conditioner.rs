use rhdl::{
    core::{
        flow_graph,
        hdl::ast::{index, unsigned_reg_decl, unsigned_wire_decl},
    },
    prelude::*,
};

// Given a reset signal in domain W that is asynchronous to
// the clock of domain R, generate a reset signal in domain R
// that is synchronous to the clock of domain R.
#[derive(Debug, Clone, Default)]
pub struct U<W: Domain, R: Domain> {
    _w: std::marker::PhantomData<W>,
    _r: std::marker::PhantomData<R>,
}

#[derive(Debug, Digital, Timed)]
pub struct I<W: Domain, R: Domain> {
    pub reset: Signal<Reset, W>,
    pub clock: Signal<Clock, R>,
}

impl<W: Domain, R: Domain> CircuitDQ for U<W, R> {
    type D = ();
    type Q = ();
}

impl<W: Domain, R: Domain> CircuitIO for U<W, R> {
    type I = I<W, R>;
    type O = Signal<Reset, R>;
    type Kernel = NoKernel2<Self::I, (), (Self::O, ())>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct S {
    clock: Clock,
    prev_reset: Reset,
    reg1_next: bool,
    reg1_current: bool,
    reg2_next: bool,
    reg2_current: bool,
}

impl<W: Domain, R: Domain> Circuit for U<W, R> {
    type S = S;

    fn init(&self) -> Self::S {
        S {
            prev_reset: Reset::dont_care(),
            clock: Clock::dont_care(),
            reg1_next: false,
            reg1_current: false,
            reg2_next: false,
            reg2_current: false,
        }
    }

    fn description(&self) -> String {
        format!("Reset synchronizer from {:?}->{:?}", W::color(), R::color())
    }

    fn sim(&self, input: Self::I, state: &mut Self::S) -> Self::O {
        const NO_RESET: bool = false;
        const RESET: bool = true;
        let clock = input.clock.val();
        let i_reset = input.reset.val();
        trace("clock", &clock);
        trace("reset", &i_reset);
        // if the clock is low, then chain the flops
        if !clock.raw() {
            state.reg1_next = NO_RESET;
            state.reg2_next = state.reg1_current;
        }
        // If there was an edge, then transition the values
        if (clock.raw() && !state.clock.raw()) || (i_reset.raw() && !state.prev_reset.raw()) {
            if !i_reset.raw() {
                state.reg1_current = state.reg1_next;
                state.reg2_current = state.reg2_next;
            } else {
                state.reg1_current = RESET;
                state.reg2_current = RESET;
                state.reg1_next = RESET;
                state.reg2_next = RESET;
            }
        }
        state.clock = clock;
        state.prev_reset = i_reset;
        trace("output", &state.reg2_current);
        signal(reset(state.reg2_current))
    }

    fn descriptor(&self, name: &str) -> Result<CircuitDescriptor, RHDLError> {
        let mut flow_graph = FlowGraph::default();
        let hdl = self.hdl(&format!("{name}_inner"))?;
        let (input, output) = flow_graph.circuit_black_box::<Self>(hdl);
        flow_graph.inputs = vec![input];
        flow_graph.output = output;
        Ok(CircuitDescriptor {
            unique_name: name.to_string(),
            flow_graph,
            input_kind: <Self::I as Timed>::static_kind(),
            output_kind: <Self::O as Digital>::static_kind(),
            d_kind: Kind::Empty,
            q_kind: Kind::Empty,
            children: Default::default(),
            rtl: None,
        })
    }

    fn hdl(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        let module_name = name.to_owned();
        let mut module = Module {
            name: module_name.clone(),
            ..Default::default()
        };
        let i_kind = <Self::I as Timed>::static_kind();
        module.ports = vec![
            port("i", Direction::Input, HDLKind::Wire, unsigned_width(2)),
            port("o", Direction::Output, HDLKind::Wire, unsigned_width(1)),
        ];
        module.declarations.extend([
            unsigned_wire_decl("i_reset", 1),
            unsigned_wire_decl("clock", 1),
            unsigned_reg_decl("reg1", 1),
            unsigned_reg_decl("reg2", 1),
        ]);
        let reassign = |name: &str, path: Path| {
            continuous_assignment(name, index("i", bit_range(i_kind, &path).unwrap().0))
        };
        module.statements.extend([
            reassign("i_reset", Path::default().field("reset").signal_value()),
            reassign("clock", Path::default().field("clock").signal_value()),
            continuous_assignment("o", id("reg2")),
        ]);
        /*
           The always block should be:
           always @(posedge clk, posedge reset) begin
               if (reset) begin
                   reg1 <= 1'b1;
                   reg2 <= 1'b1;
               end else begin
                   reg1 <= 1'b0;
                   reg2 <= reg1;
               end
           end
        */
        let reset_val = true.typed_bits().into();
        let normal_val = false.typed_bits().into();
        let reg1 = if_statement(
            id("i_reset"),
            vec![non_blocking_assignment("reg1", bit_string(&reset_val))],
            vec![non_blocking_assignment("reg1", bit_string(&normal_val))],
        );
        let reg2 = if_statement(
            id("i_reset"),
            vec![non_blocking_assignment("reg2", bit_string(&reset_val))],
            vec![non_blocking_assignment("reg2", id("reg1"))],
        );
        let events = vec![
            Events::Posedge("clock".into()),
            Events::Posedge("i_reset".into()),
        ];
        module.statements.push(always(events, vec![reg1, reg2]));
        Ok(HDLDescriptor {
            name: name.into(),
            body: module,
            children: Default::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::random;

    fn sync_stream() -> impl Iterator<Item = TimedSample<I<Red, Blue>>> {
        // Assume the red stuff comes on the edges of a clock
        let red = (0..)
            .map(|_| random::<u8>() > 200)
            .take(100)
            .stream_after_reset(1)
            .clock_pos_edge(100);
        let blue = std::iter::repeat(false)
            .stream_after_reset(1)
            .clock_pos_edge(79);
        red.merge(blue, |r, g| I {
            reset: signal(reset(r.1)),
            clock: signal(g.0.clock),
        })
    }

    #[test]
    fn test_hdl_generation() -> miette::Result<()> {
        let uut = U::<Red, Blue>::default();
        let input = sync_stream();
        let tb = uut.run(input)?.collect::<TestBench<_, _>>();
        let hdl = tb.rtl(&uut, &TestBenchOptions::default().skip(!0).vcd("rtl.vcd"))?;
        hdl.run_iverilog()?;
        let fg = tb.flow_graph(&uut, &TestBenchOptions::default())?;
        fg.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_reset_conditioner_function() -> miette::Result<()> {
        let uut = U::<Red, Blue>::default();
        let input = sync_stream();
        let output = uut.run(input)?.collect::<Vcd>();
        output
            .dump_to_file(&std::path::PathBuf::from("reset_conditioner.vcd"))
            .unwrap();
        Ok(())
    }
}
