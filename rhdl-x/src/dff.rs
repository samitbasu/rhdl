use rhdl::{
    core::{
        flow_graph::edge_kind::EdgeKind,
        hdl::ast::{
            always, assign, bit_string, id, if_statement, initial, non_blocking_assignment, port,
            signed_width, unsigned_width, Direction, Events, HDLKind, Module,
        },
        rtl::object::RegisterKind,
        types::bit_string::BitString,
        util::hash_id,
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
    cr: ClockReset,
    state: T,
    state_next: T,
}

impl<T: Digital> Synchronous for U<T> {
    type Update = Self;

    type S = S<T>;

    type Z = ();

    fn sim(
        &self,
        clock_reset: ClockReset,
        input: Self::I,
        state: &mut Self::S,
        _io: &mut Self::Z,
    ) -> Self::O {
        note("input", input);
        let clock = clock_reset.clock;
        let reset = clock_reset.reset;
        // Calculate the new state on a rising edge
        let new_state = if clock.raw() && !state.cr.clock.raw() {
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
        state.cr = clock_reset;
        note("output", new_state);
        new_state
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
        let (d, q) = flow_graph.dff(T::static_kind().into(), &self.reset.typed_bits().bits, None);
        let clock = flow_graph.buffer(RegisterKind::Unsigned(1), "clk", None);
        let reset = flow_graph.buffer(RegisterKind::Unsigned(1), "rst", None);
        d.iter().for_each(|d| {
            flow_graph.edge(reset[0], *d, EdgeKind::Reset);
            flow_graph.edge(clock[0], *d, EdgeKind::Clock);
        });
        q.iter().for_each(|q| {
            flow_graph.edge(clock[0], *q, EdgeKind::Clock);
            flow_graph.edge(reset[0], *q, EdgeKind::Reset);
        });
        flow_graph.inputs = vec![vec![clock[0], reset[0]], d, vec![]];
        flow_graph.output = q;
        Ok(CircuitDescriptor {
            unique_name: format!("{name}_dff"),
            input_kind: Self::I::static_kind(),
            output_kind: Self::O::static_kind(),
            d_kind: Kind::Empty,
            q_kind: Kind::Empty,
            num_tristate: 0,
            tristate_offset_in_parent: 0,
            children: Default::default(),
            flow_graph,
            rtl: None,
        })
    }
}

impl<T: Digital> U<T> {
    fn as_verilog(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        let module_name = self.descriptor(name)?.unique_name;
        let mut module = Module {
            name: module_name.clone(),
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
            port("clock", Direction::Input, HDLKind::Wire, unsigned_width(1)),
            port("reset", Direction::Input, HDLKind::Wire, unsigned_width(1)),
            port("i", Direction::Input, HDLKind::Wire, data_width),
            port("o", Direction::Output, HDLKind::Reg, data_width),
        ];
        module
            .statements
            .push(initial(vec![assign("o", bit_string(&init))]));
        let dff = if_statement(
            id("reset"),
            vec![non_blocking_assignment("o", bit_string(&init))],
            vec![non_blocking_assignment("o", id("i"))],
        );
        let events = vec![Events::Posedge("clock".into())];
        module.statements.push(always(events, vec![dff]));
        /*
                let input_wire = as_verilog_decl("wire", input_bits, "i");
                let output_reg = as_verilog_decl("reg", output_bits, "o");
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
         */
        Ok(HDLDescriptor {
            name: module_name,
            body: module,
            children: Default::default(),
        })
    }
}

impl<T: Digital> DigitalFn for U<T> {}
