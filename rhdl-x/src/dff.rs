use rhdl::{
    core::{flow_graph::edge_kind::EdgeKind, rtl::object::RegisterKind, util::hash_id},
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

    fn name(&self) -> String {
        "DFF".into()
    }

    fn as_hdl(&self, _: HDLKind) -> Result<HDLDescriptor, RHDLError> {
        self.as_verilog()
    }

    fn descriptor(&self) -> Result<CircuitDescriptor, RHDLError> {
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
            unique_name: format!(
                "{}_{:x}",
                self.name(),
                hash_id(std::any::TypeId::of::<Self>())
            ),
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

fn as_verilog_decl(kind: &str, len: usize, name: &str) -> String {
    let msb = len.saturating_sub(1);
    format!("{kind}[{msb}:0] {name}")
}

impl<T: Digital> U<T> {
    fn as_verilog(&self) -> Result<HDLDescriptor, RHDLError> {
        let module_name = self.descriptor()?.unique_name;
        let input_bits = T::bits();
        let output_bits = T::bits();
        let init = self.reset.typed_bits().as_verilog_literal();
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
        Ok(HDLDescriptor {
            name: module_name,
            body: code,
            children: Default::default(),
        })
    }
}

impl<T: Digital> DigitalFn for U<T> {}
