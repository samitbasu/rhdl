//! Asynchronous testbench generation
use quote::{format_ident, quote};
use rhdl_trace_type::RTT;
use rhdl_vlog::{self as vlog, LitVerilog, maybe_decl_reg, maybe_decl_wire};
use syn::parse_quote;

use crate::{
    Circuit, CircuitIO, Digital, RHDLError, TimedSample, sim::test_module::TestModule,
    trace::trace_sample::TracedSample,
};

use super::TestBenchOptions;

/// A test bench for asynchronous circuits
#[derive(Clone)]
pub struct TestBench<I: Digital, O: Digital> {
    samples: Vec<TimedSample<(I, O)>>,
}

impl<I, O> FromIterator<TracedSample<I, O>> for TestBench<I, O>
where
    I: Digital,
    O: Digital,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = TracedSample<I, O>>,
    {
        let samples = iter.into_iter().map(|ts| ts.to_timed_sample()).collect();
        TestBench { samples }
    }
}

impl<I, O> FromIterator<TimedSample<(I, O)>> for TestBench<I, O>
where
    I: Digital,
    O: Digital,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = TimedSample<(I, O)>>,
    {
        let samples = iter.into_iter().collect();
        TestBench { samples }
    }
}

impl<I: Digital, O: Digital> TestBench<I, O> {
    fn build_test_module(
        &self,
        hdl: &vlog::ModuleList,
        options: &TestBenchOptions,
    ) -> Result<TestModule, RHDLError> {
        // Assume the uut is the first entry in the list
        let uut = &hdl.modules[0];
        // Asynchronous modules may have either 1 or 2 ports.
        // If the module has 2 ports, the first port is the input
        let has_nonempty_input = uut.args.len() == 2;
        let output_port = if has_nonempty_input {
            &uut.args[1]
        } else {
            &uut.args[0]
        };
        if has_nonempty_input && I::BITS == 0 {
            return Err(RHDLError::TestbenchConstructionError(
                "Input port mismatch".into(),
            ));
        }
        let port_0_width = uut.args[0].width();
        if has_nonempty_input && I::BITS != port_0_width {
            return Err(RHDLError::TestbenchConstructionError(
                "Input port width mismatch".into(),
            ));
        }
        if !output_port.direction.is_output() || output_port.width() != O::BITS {
            return Err(RHDLError::TestbenchConstructionError(
                "Output port mismatch".into(),
            ));
        }
        let arg1_connection = (has_nonempty_input).then(|| {
            let name = format_ident!("{}", uut.args[0].decl.name);
            quote!(.#name(i))
        });
        let output_port_ident = format_ident!("{}", output_port.decl.name);
        let arg2_connection = Some(quote!(.#output_port_ident(o)));
        let connections = [arg1_connection, arg2_connection];
        let uut_name = format_ident!("{}", uut.name);
        let declarations = [
            maybe_decl_reg(I::BITS, "i"),
            maybe_decl_wire(O::BITS, "o"),
            maybe_decl_reg(O::BITS, "rust_out"),
        ];
        let preamble = if let Some(vcd_file) = &options.vcd_file {
            // Also write out an RTT file for this VCD that can be loaded
            // afterwards to provide type information for the VCD
            let rtt = RTT::TraceInfo(
                [
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
        let test_cases = self.samples.iter().enumerate().map(|(test_case_counter, timed_entry)| {
            let sample_time = timed_entry.time;
            let (sample_i, sample_o) = timed_entry.value;
            // First, we determine if at least the hold time has elapsed between the sample time and the previous recorded time
            // and ensure that we actually have an expected output and that we have passed the number of test cases to skip
            let preamble = if sample_time.saturating_sub(absolute_time) > options.hold_time
                && test_case_counter > 0
                && test_case_counter >= options.skip_first_cases
            {
                let message = format!("TESTBENCH FAILED: Expected %b, got %b -- Test {test_case_counter} at time {sample_time}");
                let hold_time = vlog::delay_stmt(options.hold_time);
                let fragment = quote! {
                    #hold_time;
                    if (o !== rust_out) begin
                        $display(#message, rust_out, o);
                        $finish;
                    end
                };
                absolute_time += options.hold_time;
                fragment
            } else {
                quote! {}
            };
            let delay = vlog::delay_stmt(sample_time.saturating_sub(absolute_time));
            absolute_time = sample_time;
            let input_update = if has_nonempty_input {
                let bin: LitVerilog = sample_i.typed_bits().into();
                quote! {
                    i = #bin;
                }
            } else {
                quote! {}
            };
            let output_bin: LitVerilog = sample_o.typed_bits().into();
            quote! {
                #preamble
                #delay;
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

    /// Generate a RTL testbench module for the given asynchronous UUT
    pub fn rtl<T>(&self, uut: &T, options: &TestBenchOptions) -> Result<TestModule, RHDLError>
    where
        T: Circuit,
        T: CircuitIO<I = I, O = O>,
    {
        let desc = uut.descriptor("uut".into())?;
        let hdl = desc.hdl()?;
        self.build_test_module(&hdl.modules, options)
    }
    /// Generate a NTL testbench module for the given asynchronous UUT
    pub fn ntl<T>(&self, uut: &T, options: &TestBenchOptions) -> Result<TestModule, RHDLError>
    where
        T: Circuit,
        T: CircuitIO<I = I, O = O>,
    {
        let module = uut.descriptor("uut".into())?.netlist()?.as_vlog("dut")?;
        self.build_test_module(&module.modules, options)
    }
}
