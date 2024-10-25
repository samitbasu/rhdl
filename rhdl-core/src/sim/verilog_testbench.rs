use crate::hdl::ast::{Direction, Module};
use crate::hdl::formatter::bit_string;
use crate::types::bit_string::BitString;
use crate::types::digital::Digital;
use crate::{Circuit, RHDLError, Synchronous, TimedSample, TypedBits};
use std::io::Write;

use super::stream::ResetData;

fn verilog_literal(t: TypedBits) -> String {
    let bs: BitString = t.into();
    bit_string(&bs)
}

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
                verilog_literal(sample.value.typed_bits())
            )
            .unwrap();
            input_prev = sample.value;
        }
    }
    writeln!(writer, "end").unwrap();
    let hdl = uut.hdl("uut")?;
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
    writeln!(writer, "{}", hdl.as_verilog()).unwrap();
    Ok(())
}

pub fn write_testbench_module<T: Digital>(
    hdl: &Module,
    inputs: impl Iterator<Item = TimedSample<T>>,
    v_filename: &str,
    out_bits: usize,
) -> Result<(), RHDLError> {
    let in_bits = T::bits();
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
    let mut input_prev = T::init();
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
                verilog_literal(sample.value.typed_bits())
            )
            .unwrap();
            input_prev = sample.value;
        }
    }
    writeln!(writer, "end").unwrap();
    let input_name = &hdl
        .ports
        .iter()
        .find(|p| p.direction == Direction::Input)
        .unwrap()
        .name;
    let output_name = &hdl
        .ports
        .iter()
        .find(|p| p.direction == Direction::Output)
        .unwrap()
        .name;
    if in_bits != 0 {
        writeln!(
            writer,
            "{} dut(.{input_name}(test_input), .{output_name}(test_output));",
            hdl.name
        )
        .unwrap();
    } else {
        writeln!(writer, "{} dut(.{output_name}(test_output));", hdl.name).unwrap();
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
    writeln!(writer, "{}", hdl.as_verilog()).unwrap();
    Ok(())
}

pub fn write_synchronous_testbench<S: Synchronous>(
    uut: &S,
    inputs: impl Iterator<Item = ResetData<S::I>>,
    clock_period: u64,
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
    writeln!(writer, "   forever #{clock_period} clock = ~clock;",).unwrap();
    writeln!(writer, "end").unwrap();
    writeln!(writer, "initial begin").unwrap();
    let has_input = in_bits != 0;
    let mut prev_time = 0_u64;
    let mut elem_prev = ResetData::Data(S::I::init());
    let hdl = uut.hdl("uut")?;
    for (ndx, elem) in inputs.enumerate() {
        let time = ndx as u64 * clock_period * 2;
        if elem != elem_prev || prev_time == 0 {
            if time != prev_time {
                writeln!(writer, "#{};", time - prev_time).unwrap();
                prev_time = time;
            }
            match elem {
                ResetData::Reset => {
                    writeln!(writer, "reset = 1;").unwrap();
                    if has_input {
                        writeln!(
                            writer,
                            "test_input = {}; ",
                            verilog_literal(S::I::init().typed_bits())
                        )
                        .unwrap();
                    }
                }
                ResetData::Data(data) => {
                    if elem_prev == ResetData::Reset {
                        writeln!(writer, "reset = 0;").unwrap();
                    }
                    if has_input {
                        writeln!(
                            writer,
                            "test_input = {};",
                            verilog_literal(data.typed_bits())
                        )
                        .unwrap();
                    }
                }
            }
            elem_prev = elem;
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
    writeln!(writer, "{}", hdl.as_verilog()).unwrap();
    Ok(())
}
