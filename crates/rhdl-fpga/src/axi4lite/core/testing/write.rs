//! Test harness for Write Controller and Write Endpoint
//!
#![doc = badascii!(r"
+CmdSrc+-+         ++WriteController+   5/6    +-+WriteEndpoint+-+    7    +CmdSink++
|        |     1   |  sink          | awaddr   |   source        |?WriteCmd|        |
|        |?WriteCmd|                +--------->|       req.data  +-------->|        |
|        +-------->| req.data       | awvalid  |                 |    8    |        |
|        |     2   |                +--------->|       req.ready |<-------+|        |
|        |<--------+ req.ready      | awready  |                 |         |        |
|        |         |                |<---------+                 |         |        |
+--------+         |                | wdata    |                 |         +--------+
                   |                +--------->| axi             |                   
                   |                | wstrobe  |                 |                   
                   |                +--------->|                 |                   
                   |                | wvalid   |                 |                   
                   |                +--------->|                 |                   
                   |                | wready   |                 |                   
                   |  - - - - - -   |<---------+  - - - - - -    |                   
+------+       3   |  source        | bresp    |  sink           |     9     +------+
|      |?WriteReslt|                |<---------+                 |?WriteReslt|      |
|Write |<----------+ resp.data      | bvalid   |       resp.data |<----------+Write |
|Result|       4   |                |<---------+                 |     10    |Result|
|Sink  +---------->| resp.ready     | bready   |      resp.ready +---------->|Source|
|      |           |                +--------->|                 |           |      |
+------+           +----------------+          +-----------------+           +------+
")]

use badascii_doc::badascii;
use rhdl::prelude::*;

use crate::{
    axi4lite::{
        core::{controller::write::WriteController, endpoint::write::WriteEndpoint},
        types::{WriteCommand, WriteResult},
    },
    stream::testing::{sink_from_fn::SinkFromFn, source_from_fn::SourceFromFn},
};

#[derive(Clone, Synchronous, SynchronousDQ)]
#[rhdl(dq_no_prefix)]
pub struct TestFixture {
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

#[cfg(test)]
mod tests {
    use std::iter::repeat_n;

    use crate::{
        axi4lite::types::StrobedData, rng::xorshift::XorShift128, stream::testing::utils::stalling,
    };

    use super::*;

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
                0 => Ok(()),
                1 => Ok(()),
                2 => Err(crate::axi4lite::types::AXI4Error::DECERR),
                _ => Err(crate::axi4lite::types::AXI4Error::SLVERR),
            }
        })
    }

    #[test]
    fn test_controller_endpoint() -> Result<(), RHDLError> {
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
        let input = input.with_reset(1).clock_pos_edge(100);
        let vcd = uut.run(input).collect::<VcdFile>();
        vcd.dump_to_file("axi_write.vcd")?;
        Ok(())
    }
}
