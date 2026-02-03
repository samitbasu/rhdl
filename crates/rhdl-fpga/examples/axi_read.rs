//! Test harness for Read Controller and Read Endpoint
//!

use std::iter::repeat_n;

use rhdl::prelude::*;
use rhdl_fpga::{
    axi4lite::{
        core::{controller::read::ReadController, endpoint::read::ReadEndpoint},
        types::ReadResult,
    },
    doc::write_svg_as_markdown,
    rng::xorshift::XorShift128,
    stream::testing::{sink_from_fn::SinkFromFn, source_from_fn::SourceFromFn, utils::stalling},
};

#[derive(Clone, Synchronous, SynchronousDQ)]
#[rhdl(dq_no_prefix)]
struct TestFixture {
    req_source: SourceFromFn<b32>,
    controller: ReadController,
    endpoint: ReadEndpoint,
    req_sink: SinkFromFn<b32>,
    reply_source: SourceFromFn<ReadResult>,
    reply_sink: SinkFromFn<ReadResult>,
}

impl SynchronousIO for TestFixture {
    type I = ();
    type O = ();
    type Kernel = kernel;
}

#[kernel]
pub fn kernel(_cr: ClockReset, _i: (), q: Q) -> ((), D) {
    let mut d = D::dont_care();
    // Wire the request source to the read controller
    d.controller.req_data = q.req_source;
    d.req_source = q.controller.req_ready;
    // Wire the request sink to the read endpoint
    d.req_sink = q.endpoint.req_data;
    d.endpoint.req_ready = q.req_sink;
    // Wire the controller to the reply sink
    d.reply_sink = q.controller.resp_data;
    d.controller.resp_ready = q.reply_sink;
    // Wire the endpoint to the reply source
    d.endpoint.resp_data = q.reply_source;
    d.reply_source = q.endpoint.resp_ready;
    // Wire the AXI busses together.
    d.controller.axi = q.endpoint.axi;
    d.endpoint.axi = q.controller.axi;
    ((), d)
}

fn main() -> Result<(), RHDLError> {
    let rng = XorShift128::default().map(|x| bits(x as u128));
    let address_sink = rng.clone();
    let address = stalling(rng.clone(), 0.23);
    let reply = rng.clone().map(ReadResult::Ok);
    let reply_sink = reply.clone();
    let reply = stalling(reply, 0.23);
    let uut = TestFixture {
        req_source: SourceFromFn::new(address),
        controller: ReadController::default(),
        endpoint: ReadEndpoint::default(),
        req_sink: SinkFromFn::new_from_iter(address_sink, 0.3),
        reply_source: SourceFromFn::new(reply),
        reply_sink: SinkFromFn::new_from_iter(reply_sink, 0.1),
    };
    let input = repeat_n((), 250);
    let input = input
        .with_reset(1)
        .clock_pos_edge(100)
        .take_while(|t| t.time < 1500);

    let vcd = uut.run(input).collect::<SvgFile>();
    let options = SvgOptions::default().with_filter(".*controller.*axi.*");
    write_svg_as_markdown(vcd, "axi_read.md", options)?;
    Ok(())
}
