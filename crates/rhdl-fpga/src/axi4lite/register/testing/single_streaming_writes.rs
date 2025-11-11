use rhdl::prelude::*;

use crate::{
    axi4lite::{
        core::controller::write::WriteController,
        register::single::AxiRegister,
        types::{ReadMOSI, StrobedData, WriteCommand, WriteResult},
    },
    core::dff::DFF,
    rng::xorshift::{XorShift, XorShift128},
    stream::testing::{sink_from_fn::SinkFromFn, source_from_fn::SourceFromFn, utils::stalling},
};

#[derive(Clone, Synchronous, SynchronousDQ)]
pub struct Fixture {
    write_source: SourceFromFn<WriteCommand>,
    write: WriteController,
    write_sink: SinkFromFn<WriteResult>,
    reg: AxiRegister,
    xor: XorShift,
    valid: DFF<bool>,
    prev_value: DFF<b32>,
}

impl Default for Fixture {
    fn default() -> Self {
        // get a set of write commands
        let cmd = XorShift128::default().map(|x| WriteCommand {
            addr: bits(0),
            strobed_data: StrobedData {
                data: bits(x as u128),
                strobe: bits(0b1111),
            },
        });
        let cmd = stalling(cmd, 0.23);
        // For the write sink, we expect all writes to succeed
        let acceptor = |x: Option<WriteResult>| {
            if let Some(res) = x {
                assert_eq!(res, Ok(()));
            }
            rand::random_bool(0.85)
        };
        Self {
            write_source: SourceFromFn::new(cmd),
            write: WriteController::default(),
            write_sink: SinkFromFn::new(acceptor),
            reg: AxiRegister::new(bits(0), bits(0)),
            xor: XorShift::default(),
            valid: DFF::new(true),
            prev_value: DFF::new(bits(0)),
        }
    }
}

impl SynchronousIO for Fixture {
    type I = ();
    type O = ();
    type Kernel = kernel;
}

#[kernel]
#[doc(hidden)]
pub fn kernel(_cr: ClockReset, _i: (), q: Q) -> ((), D) {
    let mut d = D::dont_care();
    // Pair the source interfaces
    d.write.req_data = q.write_source;
    d.write_source = q.write.req_ready;
    // Pair the sink interfaces
    d.write.resp_ready = q.write_sink;
    d.write_sink = q.write.resp_data;
    // Pair the AXI interface to the register
    d.reg.write_axi = q.write.axi;
    d.write.axi = q.reg.write_axi;
    // Nothing on the read interface in this test case
    d.reg.read_axi = ReadMOSI::default();
    // Nothing on the core write interface in this case
    d.reg.data = None;
    d.valid = q.valid;
    d.xor = false;
    d.prev_value = q.prev_value;
    if q.reg.data != q.prev_value {
        // Register value has changed
        d.prev_value = q.reg.data;
        // Update the valid flag comparing with the XOR sequence
        d.valid = q.valid & (q.reg.data == q.xor);
        // Advance the XOR generator
        d.xor = true;
    }
    ((), d)
}

#[cfg(test)]
mod tests {
    use std::iter::repeat_n;

    use super::*;

    #[test]
    fn synth_works() -> miette::Result<()> {
        let input = repeat_n((), 100).with_reset(1).clock_pos_edge(100);
        let uut = Fixture::default();
        let vcd = uut.run(input).collect::<Vcd>();
        vcd.dump_to_file("thing.vcd").unwrap();
        Ok(())
    }
}
