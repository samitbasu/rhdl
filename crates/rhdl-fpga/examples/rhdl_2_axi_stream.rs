use rhdl::prelude::*;
use rhdl_fpga::{
    axi4lite::stream::{axi_to_rhdl::Axi2Rhdl, rhdl_to_axi::Rhdl2Axi},
    rng::xorshift::XorShift128,
    stream::testing::{sink_from_fn::SinkFromFn, source_from_fn::SourceFromFn, utils::stalling},
};

// For demo, we convert a RHDL stream to AXI and back again.
// Just because.

#[derive(Clone, Synchronous, SynchronousDQ)]
struct TestFixture {
    source: SourceFromFn<b8>,
    rhdl_2_axi: Rhdl2Axi<b8>,
    axi_2_rhdl: Axi2Rhdl<b8>,
    sink: SinkFromFn<b8>,
}

impl SynchronousIO for TestFixture {
    type I = ();
    type O = ();
    type Kernel = kernel;
}

#[kernel]
#[doc(hidden)]
pub fn kernel(_cr: ClockReset, _i: (), q: Q) -> ((), D) {
    let mut d = D::dont_care();
    d.rhdl_2_axi.data = q.source;
    d.source = q.rhdl_2_axi.ready;
    d.axi_2_rhdl.tdata = q.rhdl_2_axi.tdata;
    d.axi_2_rhdl.tvalid = q.rhdl_2_axi.tvalid;
    d.rhdl_2_axi.tready = q.axi_2_rhdl.tready;
    d.sink = q.axi_2_rhdl.data;
    d.axi_2_rhdl.ready = q.sink;
    ((), d)
}

fn main() -> Result<(), RHDLError> {
    let a_rng = XorShift128::default().map(|x| b8((x & 0xFF) as u128));
    let b_rng = a_rng.clone();
    let a_rng = stalling(a_rng, 0.23);
    let uut = TestFixture {
        source: SourceFromFn::new(a_rng),
        rhdl_2_axi: Rhdl2Axi::default(),
        axi_2_rhdl: Axi2Rhdl::default(),
        sink: SinkFromFn::new_from_iter(b_rng, 0.2),
    };
    let input = std::iter::repeat_n((), 15)
        .with_reset(1)
        .clock_pos_edge(100);
    let vcd = uut.run_without_synthesis(input)?.collect::<Vcd>();
    let options = SvgOptions::default();
    rhdl_fpga::doc::write_svg_as_markdown(vcd, "rhdl_2_axi_stream.md", options)?;
    Ok(())
}
