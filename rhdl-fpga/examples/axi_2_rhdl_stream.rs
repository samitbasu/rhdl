use rhdl::prelude::*;
use rhdl_fpga::{
    axi4lite::stream::axi_to_rhdl::Axi2Rhdl,
    core::option::unpack,
    rng::xorshift::XorShift128,
    stream::testing::{sink_from_fn::SinkFromFn, source_from_fn::SourceFromFn, utils::stalling},
};

#[derive(Clone, Synchronous, SynchronousDQ)]
struct TestFixture {
    source: SourceFromFn<b8>,
    axi_2_rhdl: Axi2Rhdl<b8>,
    sink: SinkFromFn<b8>,
}

impl SynchronousIO for TestFixture {
    type I = ();
    type O = ();
    type Kernel = kernel;
}

#[kernel]
pub fn kernel(_cr: ClockReset, _i: (), q: Q) -> ((), D) {
    let mut d = D::dont_care();
    let (valid, data) = unpack::<b8>(q.source, bits(0));
    d.axi_2_rhdl.tdata = data;
    d.axi_2_rhdl.tvalid = valid;
    d.sink = q.axi_2_rhdl.data;
    d.axi_2_rhdl.ready = q.sink;
    d.source = q.axi_2_rhdl.tready;
    ((), d)
}

fn main() -> Result<(), RHDLError> {
    let a_rng = XorShift128::default().map(|x| b8((x & 0xFF) as u128));
    let b_rng = a_rng.clone();
    let a_rng = stalling(a_rng, 0.23);
    let uut = TestFixture {
        source: SourceFromFn::new(a_rng),
        axi_2_rhdl: Axi2Rhdl::default(),
        sink: SinkFromFn::new_from_iter(b_rng, 0.2),
    };
    let input = std::iter::repeat_n((), 15)
        .with_reset(1)
        .clock_pos_edge(100);
    let vcd = uut.run_without_synthesis(input)?.collect::<Vcd>();
    let options = SvgOptions::default();
    rhdl_fpga::doc::write_svg_as_markdown(vcd, "axi_2_rhdl_stream.md", options)?;
    Ok(())
}
