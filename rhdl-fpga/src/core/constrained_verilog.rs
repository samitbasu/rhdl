// Given a struct that impl Synchronous, and a set of constraints,
// generate a Verilog module and constraint file.  For now, let's just
// get the Alchitry Cu working.

use rhdl_core::compile_design;
use rhdl_core::generate_verilog;

use rhdl_core::Digital;
use rhdl_core::DigitalFn;
use rhdl_core::Synchronous;

use crate::Constraint;
use crate::PinConstraint;
use crate::PinConstraintKind;
use anyhow::Result;

pub struct ConstrainedVerilog {
    pub module: String,
    pub constraints: Vec<PinConstraint>,
}

pub fn make_constrained_verilog<M: Synchronous>(
    obj: M,
    mut constraints: Vec<PinConstraint>,
    clock_source: Constraint,
) -> Result<ConstrainedVerilog> {
    let verilog = generate_verilog(&compile_design(M::Update::kernel_fn().try_into()?)?)?;
    let module_code = format!("{}", verilog);
    let module = format!(
        "
module top(input wire clk, input wire[{INPUT_BITS}:0] top_in, output reg[{OUTPUT_BITS}:0] top_out);
    localparam config_value = {config};
    reg[{STATE_BITS}:0] state;
    wire [{STATE_AND_OUTPUT_BITS}:0] update_result;
    wire [{OUTPUT_BITS}:0] output_value;
    
    {module_code}

    assign update_result = {update_fn}(config_value, state, top_in);
    assign output_value = update_result[{OUTPUT_END}:{OUTPUT_START}];

    always @(posedge clk) begin 
        state <= update_result[{STATE_BITS}:0];
        top_out <= output_value;
    end

    // This may not work.
    initial begin
        state <= {initial_state};
    end
endmodule
    ",
        STATE_BITS = M::State::bits().saturating_sub(1),
        STATE_AND_OUTPUT_BITS = (M::State::bits() + M::Output::bits()).saturating_sub(1),
        INPUT_BITS = M::Input::bits().saturating_sub(1),
        OUTPUT_BITS = M::Output::bits().saturating_sub(1),
        update_fn = verilog.name,
        config = rhdl_core::as_verilog_literal(&obj.typed_bits()),
        initial_state = rhdl_core::as_verilog_literal(&M::INITIAL_STATE.typed_bits()),
        OUTPUT_START = M::State::bits(),
        OUTPUT_END = (M::State::bits() + M::Output::bits()).saturating_sub(1),
    );
    constraints.push(PinConstraint {
        kind: PinConstraintKind::Clock("clk".into()),
        index: 0,
        constraint: clock_source,
    });
    Ok(ConstrainedVerilog {
        module,
        constraints,
    })
}
