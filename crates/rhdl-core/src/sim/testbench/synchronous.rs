use quote::{format_ident, quote};
use rhdl_trace_type::RTT;
use rhdl_vlog::{self as vlog, maybe_decl_reg, maybe_decl_wire};
use syn::parse_quote;

use crate::{
    ClockReset, Digital, RHDLError, Synchronous, SynchronousIO, TimedSample, clock_reset,
    sim::test_module::TestModule,
};

use super::TestBenchOptions;

#[derive(Clone)]
pub struct SynchronousTestBench<I: Digital, O: Digital> {
    pub samples: Vec<TimedSample<(ClockReset, I, O)>>,
}

impl<I, O> FromIterator<TimedSample<(ClockReset, I, O)>> for SynchronousTestBench<I, O>
where
    I: Digital,
    O: Digital,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = TimedSample<(ClockReset, I, O)>>,
    {
        let samples = iter.into_iter().collect();
        SynchronousTestBench { samples }
    }
}

impl<I: Digital, O: Digital> SynchronousTestBench<I, O> {
    fn build_test_module(
        &self,
        hdl: &vlog::ModuleList,
        options: &TestBenchOptions,
    ) -> Result<TestModule, RHDLError> {
        let uut = &hdl.modules[0];
        // All synchronous modules must have at least 2
        // ports (the first is clock + reset, the last is
        // the output).  They may have 3 if the circuit takes
        // input signals.
        let has_nonempty_input = uut.args.len() == 3;
        let output_port = if has_nonempty_input {
            &uut.args[2]
        } else {
            &uut.args[1]
        };
        if !uut.args[0].direction.is_input() || uut.args[0].width() != 2 {
            return Err(RHDLError::TestbenchConstructionError(
                "First port must be an input with 2 bits width".into(),
            ));
        }
        if has_nonempty_input && (I::BITS == 0) {
            return Err(RHDLError::TestbenchConstructionError(
                "Input port mismatch".into(),
            ));
        }
        if has_nonempty_input && I::BITS != uut.args[1].width() {
            return Err(RHDLError::TestbenchConstructionError(
                "Input port width mismatch".into(),
            ));
        }
        if !output_port.direction.is_output() || output_port.width() != O::BITS {
            return Err(RHDLError::TestbenchConstructionError(format!(
                "Output port mismatch: direction {dir:?} width {width} expected width {expected}",
                dir = output_port.direction,
                width = output_port.width(),
                expected = O::BITS
            )));
        }
        let clock_reset_port_ident = format_ident!("{}", uut.args[0].decl.name);
        let arg0_connection = Some(quote! { .#clock_reset_port_ident(clock_reset) });
        let arg1_connection = (has_nonempty_input).then(|| {
            let name = format_ident!("{}", uut.args[1].decl.name);
            quote!(.#name(i))
        });
        let output_port_ident = format_ident!("{}", output_port.decl.name);
        let arg2_connection = Some(quote!(.#output_port_ident(o)));
        let uut_name = format_ident!("{}", uut.name);
        let declarations = [
            maybe_decl_reg(ClockReset::bits(), "clock_reset"),
            maybe_decl_reg(I::BITS, "i"),
            maybe_decl_wire(O::BITS, "o"),
            maybe_decl_reg(O::BITS, "rust_out"),
        ];
        let connections = [arg0_connection, arg1_connection, arg2_connection];
        let preamble = if let Some(vcd_file) = &options.vcd_file {
            // Also write out an RTT file for this VCD that can be loaded
            // afterwards to provide type information for the VCD
            let rtt = RTT::TraceInfo(
                [
                    (
                        "testbench.clock_reset".to_string(),
                        ClockReset::static_trace_type(),
                    ),
                    ("testbench.i".to_string(), I::static_trace_type()),
                    ("testbench.o".to_string(), O::static_trace_type()),
                    ("testbench.rust_out".to_string(), O::static_trace_type()),
                ]
                .into_iter()
                .collect(),
            );
            std::fs::write(
                vcd_file.clone() + ".rtt",
                ron::ser::to_string(&rtt).unwrap(),
            )?;
            quote! {
                $dumpfile(#vcd_file);
                $dumpvars(0);
            }
        } else {
            quote! {}
        };
        let mut absolute_time = 0;
        let test_cases = self
            .samples
            .iter()
            .enumerate()
            .map(|(test_case_counter, timed_entry)| {
                let sample_time = timed_entry.time;
                let (sample_cr, sample_i, sample_o) = timed_entry.value;
                // First, we determine if at least the hold time has elapsed between the sample time and the previous recorded time
                // and ensure that we actually have an expected output and that we have passed the number of test cases to skip
                let preamble = if sample_time.saturating_sub(absolute_time) > options.hold_time
                    && test_case_counter > 0
                    && test_case_counter >= options.skip_first_cases
                {
                    let message = format!("Test {test_case_counter} at time {absolute_time}");
                    let hold_time = syn::Index::from(options.hold_time as usize);
                    let fragment = quote! {
                        # #hold_time;
                        if (o !== rust_out) begin
                            $display("TESTBENCH FAILED: Expected %b, got %b -- " #message, rust_out, o);
                            $finish;
                        end
                    };
                    absolute_time += options.hold_time;
                    fragment
                } else {
                    quote! {}
                };
                let delay = syn::Index::from(sample_time.saturating_sub(absolute_time) as usize);
                absolute_time = sample_time;
                let cr : vlog::LitVerilog = clock_reset(sample_cr.clock, sample_cr.reset).typed_bits().into();
                let input_update = if has_nonempty_input {
                    let bin: vlog::LitVerilog = sample_i.typed_bits().into();
                    quote! {
                        i = #bin;
                    }
                } else {
                    quote! {}
                };
                let output_bin: vlog::LitVerilog = sample_o.typed_bits().into();
                quote! {
                    #preamble
                    # #delay;
                    clock_reset = #cr;
                    #input_update
                    rust_out = #output_bin;
                }
            });
        let module: vlog::ModuleList = parse_quote! {
            module testbench;
                #(#declarations;)*
                #uut_name t(#(#connections),*);
                initial begin
                    #preamble
                    #(#test_cases)*
                    $display("TESTBENCH OK");
                    $finish;
                end
            endmodule
            #hdl
        };
        Ok(module.into())
    }

    pub fn rtl<T>(&self, uut: &T, options: &TestBenchOptions) -> Result<TestModule, RHDLError>
    where
        T: Synchronous,
        T: SynchronousIO<I = I, O = O>,
    {
        let module = uut.hdl("uut")?.as_module();
        self.build_test_module(&module, options)
    }
    pub fn ntl<T>(&self, uut: &T, options: &TestBenchOptions) -> Result<TestModule, RHDLError>
    where
        T: Synchronous,
        T: SynchronousIO<I = I, O = O>,
    {
        let module = crate::ntl::hdl::generate_hdl("dut", &uut.descriptor("uut")?.ntl)?;
        self.build_test_module(&module, options)
    }
}
