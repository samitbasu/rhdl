use crate::types::digital::Digital;
use crate::{Circuit, ClockReset, HDLKind, RHDLError, Synchronous, TimedSample};
use std::io::Write;

pub fn write_testbench<C: Circuit>(
    uut: &C,
    inputs: impl Iterator<Item = TimedSample<C::I>>,
    v_filename: &str,
) -> Result<(), RHDLError> {
    let out_bits = C::O::bits();
    let in_bits = C::I::bits();
    let in_decl = if in_bits != 0 {
        Some(format!(
            "reg [{in_msb}:0] test_input",
            in_msb = in_bits.saturating_sub(1)
        ))
    } else {
        None
    };
    let out_decl = format!(
        "wire [{out_msb}:0] test_output",
        out_msb = out_bits.saturating_sub(1)
    );
    let file = std::fs::File::create(v_filename).unwrap();
    let mut writer = std::io::BufWriter::new(file);
    writeln!(writer, "module top;").unwrap();
    if let Some(decl) = in_decl {
        writeln!(writer, "{};", decl).unwrap();
    }
    writeln!(writer, "{};", out_decl).unwrap();
    writeln!(writer, "initial begin").unwrap();
    let mut prev_time = 0_u64;
    let mut input_prev = C::I::init();
    for sample in inputs {
        let time = sample.time;
        if sample.value != input_prev || prev_time == 0 {
            if time != prev_time {
                writeln!(writer, "#{};", time - prev_time).unwrap();
                prev_time = time;
            }
            writeln!(
                writer,
                "test_input = {};",
                sample.value.typed_bits().as_verilog_literal()
            )
            .unwrap();
            input_prev = sample.value;
        }
    }
    writeln!(writer, "end").unwrap();
    let hdl = uut.as_hdl(HDLKind::Verilog)?;
    if in_bits != 0 {
        writeln!(writer, "{} dut(.i(test_input), .o(test_output));", hdl.name).unwrap();
    } else {
        writeln!(writer, "{} dut(.o(test_output));", hdl.name).unwrap();
    }
    writeln!(writer, "initial begin").unwrap();
    writeln!(
        writer,
        "$dumpfile(\"{}.vcd\");",
        v_filename.replace(".", "_")
    )
    .unwrap();
    writeln!(writer, "$dumpvars(0);").unwrap();
    writeln!(writer, "#{};", prev_time).unwrap();
    writeln!(writer, "$finish;").unwrap();
    writeln!(writer, "end").unwrap();
    writeln!(writer, "endmodule").unwrap();
    writeln!(writer, "{:?}", hdl).unwrap();
    Ok(())
}

pub fn write_synchronous_testbench<S: Synchronous>(
    uut: &S,
    mut inputs: impl Iterator<Item = TimedSample<(ClockReset, S::I)>>,
    v_filename: &str,
) -> Result<(), RHDLError> {
    let out_bits = S::O::bits();
    let in_bits = S::I::bits();
    let in_decl = if in_bits != 0 {
        Some(format!(
            "reg [{in_msb}:0] test_input",
            in_msb = in_bits.saturating_sub(1)
        ))
    } else {
        None
    };
    let out_decl = format!(
        "wire [{out_msb}:0] test_output",
        out_msb = out_bits.saturating_sub(1)
    );
    let file = std::fs::File::create(v_filename).unwrap();
    let mut writer = std::io::BufWriter::new(file);
    writeln!(writer, "module top;").unwrap();
    writeln!(writer, "reg clock;").unwrap();
    writeln!(writer, "reg reset;").unwrap();
    if let Some(decl) = in_decl {
        writeln!(writer, "{};", decl).unwrap();
    }
    writeln!(writer, "{};", out_decl).unwrap();
    // Add a periodic clock.
    writeln!(writer, "initial begin").unwrap();
    writeln!(writer, "   clock = 1;").unwrap();
    writeln!(writer, "   forever #{period} clock = ~clock;",).unwrap();
    writeln!(writer, "end").unwrap();
    writeln!(writer, "initial begin").unwrap();
    let clock_stream = sim_clock(period);
    let reset_stream = sim_clock_reset(clock_stream);
    let mut input = S::I::init();
    let mut prev_time = 0_u64;
    let mut reset_prev = false;
    let mut input_prev = S::I::init();
    let hdl = uut.as_hdl(HDLKind::Verilog)?;
    for cr in reset_stream {
        if cr.value.clock.raw() && !cr.value.reset.raw() {
            if let Some(sample) = inputs.next() {
                input = sample;
            } else {
                break;
            }
        }
        let time = cr.time;
        if cr.value.reset.raw() != reset_prev || prev_time == 0 {
            if time != prev_time {
                writeln!(writer, "#{};", time - prev_time).unwrap();
                prev_time = time;
            }
            writeln!(
                writer,
                "reset = {};",
                if cr.value.reset.raw() { 1 } else { 0 }
            )
            .unwrap();
            reset_prev = cr.value.reset.raw();
        }
        if in_bits != 0 && input != input_prev || prev_time == 0 {
            if time != prev_time {
                writeln!(writer, "#{};", time - prev_time).unwrap();
                prev_time = time;
            }
            writeln!(
                writer,
                "test_input = {};",
                input.typed_bits().as_verilog_literal()
            )
            .unwrap();
            input_prev = input;
        }
    }
    writeln!(writer, "end").unwrap();
    if in_bits != 0 {
        writeln!(
            writer,
            "{} dut(.clock(clock), .reset(reset), .i(test_input), .o(test_output));",
            hdl.name
        )
        .unwrap();
    } else {
        writeln!(
            writer,
            "{} dut(.clock(clock), .reset(reset), .o(test_output));",
            hdl.name
        )
        .unwrap();
    }
    writeln!(writer, "initial begin").unwrap();
    writeln!(
        writer,
        "$dumpfile(\"{}.vcd\");",
        v_filename.replace(".", "_")
    )
    .unwrap();
    writeln!(writer, "$dumpvars(0);").unwrap();
    writeln!(writer, "#{};", prev_time).unwrap();
    writeln!(writer, "$finish;").unwrap();
    writeln!(writer, "end").unwrap();
    writeln!(writer, "endmodule").unwrap();
    writeln!(writer, "{:?}", hdl).unwrap();
    Ok(())
}
