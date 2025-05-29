//! Test harness for Write Controller and Write Endpoint
//!

use std::iter::repeat_n;

use badascii_doc::badascii;
use rhdl::prelude::*;
use rhdl_fpga::{
    axi4lite::{
        native::{controller::write::WriteController, endpoint::write::WriteEndpoint},
        types::{AXI4Error, ExFlag, StrobedData, WriteCommand, WriteResult},
    },
    doc::write_svg_as_markdown,
    rng::xorshift::XorShift128,
    stream::testing::{sink_from_fn::SinkFromFn, source_from_fn::SourceFromFn, utils::stalling},
};

#[derive(Clone, Synchronous, SynchronousDQ)]
struct TestFixture {
    req_source: SourceFromFn<WriteCommand>,
    controller: WriteController,
    endpoint: WriteEndpoint,
    req_sink: SinkFromFn<WriteCommand>,
    reply_source: SourceFromFn<WriteResult>,
    reply_sink: SinkFromFn<WriteResult>,
}

impl SynchronousIO for TestFixture {
    type I = ();
    type O = ();
    type Kernel = kernel;
}

#[kernel]
pub fn kernel(_cr: ClockReset, _i: (), q: Q) -> ((), D) {
    let mut d = D::dont_care();
    d.controller.req_data = q.req_source; // 1
    d.req_source = q.controller.req_ready; // 2
    d.reply_sink = q.controller.resp_data; // 3
    d.controller.resp_ready = q.reply_sink; // 4
    d.controller.axi = q.endpoint.axi; // 5
    d.endpoint.axi = q.controller.axi; // 6
    d.req_sink = q.endpoint.req_data; // 7
    d.endpoint.req_ready = q.req_sink; // 8
    d.endpoint.resp_data = q.reply_source; // 9
    d.reply_source = q.endpoint.resp_ready; // 10
    ((), d)
}

fn write_commands() -> impl Iterator<Item = WriteCommand> {
    XorShift128::default().map(|x| {
        let addr = x >> 16;
        let data = x & 0xFFFF;
        WriteCommand {
            addr: bits(addr as u128),
            strobed_data: StrobedData {
                data: bits(data as u128),
                strobe: bits(0b0011),
            },
        }
    })
}

fn iter_results() -> impl Iterator<Item = WriteResult> {
    XorShift128::default().map(|x| {
        let x = x & 0b11;
        match x {
            0 => Ok(ExFlag::Normal),
            1 => Ok(ExFlag::Exclusive),
            2 => Err(AXI4Error::DECERR),
            _ => Err(AXI4Error::SLVERR),
        }
    })
}

fn main() -> Result<(), RHDLError> {
    let commands = write_commands();
    let commands_sink = write_commands();
    let commands = stalling(commands, 0.23);
    let results = iter_results();
    let results_sink = iter_results();
    let results = stalling(results, 0.23);
    let uut = TestFixture {
        req_source: SourceFromFn::new(commands),
        controller: WriteController::default(),
        endpoint: WriteEndpoint::default(),
        req_sink: SinkFromFn::new_from_iter(commands_sink, 0.1),
        reply_source: SourceFromFn::new(results),
        reply_sink: SinkFromFn::new_from_iter(results_sink, 0.1),
    };
    let input = repeat_n((), 250);
    let input = input
        .with_reset(1)
        .clock_pos_edge(100)
        .take_while(|t| t.time < 1500);
    let vcd = uut.run_without_synthesis(input)?.collect::<Vcd>();
    let options = SvgOptions::default().with_filter(".*controller.*axi.*");
    write_svg_as_markdown(vcd, "axi_write.md", options)?;
    Ok(())
}
