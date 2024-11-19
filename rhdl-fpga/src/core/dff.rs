use rhdl::{
    core::{
        hdl::ast::{
            always, assign, bit_string, id, if_statement, index_bit, initial,
            non_blocking_assignment, port, signed_width, unsigned_width, Declaration, Direction,
            Events, HDLKind, Module,
        },
        types::bit_string::BitString,
    },
    prelude::*,
};

#[derive(Debug, Clone)]
pub struct U<T: Digital> {
    reset: T,
}

impl<T: Digital> U<T> {
    pub fn new(reset: T) -> Self {
        Self { reset }
    }
}

impl<T: Digital + Default> Default for U<T> {
    fn default() -> Self {
        Self {
            reset: T::default(),
        }
    }
}

impl<T: Digital> SynchronousIO for U<T> {
    type I = T;
    type O = T;
    type Kernel = NoKernel3<ClockReset, T, (), (T, ())>;
}

impl<T: Digital> SynchronousDQ for U<T> {
    type D = ();
    type Q = ();
}

#[derive(Debug, Clone, PartialEq, Copy, Digital)]
pub struct S<T: Digital> {
    cr: ClockReset,
    reset: Reset,
    current: T,
    next: T,
}

impl<T: Digital> Synchronous for U<T> {
    type S = S<T>;

    fn init(&self) -> Self::S {
        Self::S::init()
    }

    fn sim(&self, clock_reset: ClockReset, input: Self::I, state: &mut Self::S) -> Self::O {
        trace_push_path("dff");
        trace("input", &input);
        let clock = clock_reset.clock;
        let reset = clock_reset.reset;
        if !clock.raw() {
            state.next = input;
            state.reset = reset;
        }
        if clock.raw() && !state.cr.clock.raw() {
            if state.reset.raw() {
                state.current = self.reset;
            } else {
                state.current = state.next;
            }
        }
        state.cr = clock_reset;
        trace("output", &state.current);
        trace_pop_path();
        state.current
    }

    fn description(&self) -> String {
        format!(
            "Positive edge triggered DFF holding value of type {:?}, with reset value of {:?}",
            T::static_kind(),
            self.reset.typed_bits()
        )
    }

    fn hdl(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        self.as_verilog(name)
    }

    fn descriptor(&self, name: &str) -> Result<CircuitDescriptor, RHDLError> {
        let mut flow_graph = FlowGraph::default();
        let module = self.hdl(name)?;
        let (clock_reset, d, q) = flow_graph.synchronous_black_box::<Self>(module);
        flow_graph.inputs = vec![clock_reset, d];
        flow_graph.output = q;
        Ok(CircuitDescriptor {
            unique_name: name.to_string(),
            input_kind: Self::I::static_kind(),
            output_kind: Self::O::static_kind(),
            d_kind: Kind::Empty,
            q_kind: Kind::Empty,
            children: Default::default(),
            flow_graph,
            rtl: None,
        })
    }
}

impl<T: Digital> U<T> {
    fn as_verilog(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        let mut module = Module {
            name: name.into(),
            ..Default::default()
        };
        let output_bits = T::bits();
        let init: BitString = self.reset.typed_bits().into();
        let data_width = if T::static_kind().is_signed() {
            signed_width(output_bits)
        } else {
            unsigned_width(output_bits)
        };
        module.ports = vec![
            port(
                "clock_reset",
                Direction::Input,
                HDLKind::Wire,
                unsigned_width(2),
            ),
            port("i", Direction::Input, HDLKind::Wire, data_width),
            port("o", Direction::Output, HDLKind::Reg, data_width),
        ];
        module.declarations.push(Declaration {
            kind: HDLKind::Wire,
            name: "clock".into(),
            width: unsigned_width(1),
            alias: None,
        });
        module.declarations.push(Declaration {
            kind: HDLKind::Wire,
            name: "reset".into(),
            width: unsigned_width(1),
            alias: None,
        });
        module
            .statements
            .push(initial(vec![assign("o", bit_string(&init))]));
        module
            .statements
            .push(continuous_assignment("clock", index_bit("clock_reset", 0)));
        module
            .statements
            .push(continuous_assignment("reset", index_bit("clock_reset", 1)));
        let dff = if_statement(
            id("reset"),
            vec![non_blocking_assignment("o", bit_string(&init))],
            vec![non_blocking_assignment("o", id("i"))],
        );
        let events = vec![Events::Posedge("clock".into())];
        module.statements.push(always(events, vec![dff]));
        Ok(HDLDescriptor {
            name: name.into(),
            body: module,
            children: Default::default(),
        })
    }
}
