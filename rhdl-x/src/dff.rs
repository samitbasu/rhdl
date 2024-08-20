use rhdl::{
    core::{rtl::object::RegisterKind, util::hash_id},
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
    clock: Clock,
    state: T,
    state_next: T,
}

impl<T: Digital> Synchronous for U<T> {
    type Update = Self;

    type S = S<T>;

    type Z = ();

    fn sim(
        &self,
        clock: Clock,
        reset: Reset,
        input: Self::I,
        state: &mut Self::S,
        _io: &mut Self::Z,
    ) -> Self::O {
        note("input", input);
        // Calculate the new state on a rising edge
        let new_state = if clock.raw() && !state.clock.raw() {
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
        state.clock = clock;
        note("output", new_state);
        new_state
    }

    fn name(&self) -> &'static str {
        "DFF"
    }

    fn as_hdl(&self, _: HDLKind) -> Result<HDLDescriptor, RHDLError> {
        self.as_verilog()
    }

    fn descriptor(&self) -> Result<CircuitDescriptor, RHDLError> {
        let mut fg = FlowGraph::default();
        // Make the FG slightly nicer
        let rst = fg.sink(RegisterKind::Unsigned(1), "rst", None);
        let d = fg.sink(Self::I::static_kind().into(), "d", None);
        let q = fg.source(Self::O::static_kind().into(), "q", None);
        fg.inputs = vec![Some(rst), Some(d), None];
        fg.output = q;
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
            update_schematic: None,
            tristate_offset_in_parent: 0,
            children: Default::default(),
            update_flow_graph: fg,
        })
    }
}

fn as_verilog_decl(kind: &str, len: usize, name: &str) -> String {
    let msb = len.saturating_sub(1);
    format!("{kind} {name}[{msb}:0]")
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
