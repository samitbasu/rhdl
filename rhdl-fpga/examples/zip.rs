use std::iter::repeat_n;

use rhdl::prelude::*;
use rhdl_fpga::{
    rng::xorshift::XorShift128,
    stream::{
        testing::{sink_from_fn::SinkFromFn, source_from_fn::SourceFromFn, utils::stalling},
        zip::Zip,
    },
};

#[derive(Clone, Synchronous, SynchronousDQ)]
/// The test fixture has 2 sources feeding
///  a [Zip] core and a sink to consume the output.
struct TestFixture {
    a_source: SourceFromFn<b4>,
    b_source: SourceFromFn<b6>,
    zip: Zip<b4, b6>,
    sink: SinkFromFn<(b4, b6)>,
}

impl SynchronousIO for TestFixture {
    type I = ();
    type O = ();
    type Kernel = kernel;
}

#[kernel]
pub fn kernel(_cr: ClockReset, _i: (), q: Q) -> ((), D) {
    let mut d = D::dont_care();
    d.zip.a_data = q.a_source;
    d.zip.b_data = q.b_source;
    d.sink = q.zip.data;
    d.zip.ready = q.sink;
    d.a_source = q.zip.a_ready;
    d.b_source = q.zip.b_ready;
    ((), d)
}

fn main() -> Result<(), RHDLError> {
    let a_rng = XorShift128::default().map(|x| b4((x & 0xF) as u128));
    let b_rng = XorShift128::default().map(|x| b6(((x >> 8) & 0x3F) as u128));
    let mut c_rng = a_rng.clone().zip(b_rng.clone());
    let a_rng = stalling(a_rng, 0.23);
    let b_rng = stalling(b_rng, 0.15);
    let consume = move |data| {
        if let Some(data) = data {
            let validation = c_rng.next().unwrap();
            assert_eq!(data, validation);
        }
        rand::random::<f64>() > 0.2
    };
    let uut = TestFixture {
        a_source: SourceFromFn::new(a_rng),
        b_source: SourceFromFn::new(b_rng),
        zip: Zip::default(),
        sink: SinkFromFn::new(consume),
    };
    // Run a few samples through
    let input = repeat_n((), 15).with_reset(1).clock_pos_edge(100);
    let vcd = uut.run_without_synthesis(input)?.collect::<Vcd>();
    rhdl_fpga::doc::write_svg_as_markdown(
        vcd,
        "zip.md",
        SvgOptions::default().with_filter("(.*zip.input.data)|(.*zip.outputs.data)"),
    )?;
    Ok(())
}
