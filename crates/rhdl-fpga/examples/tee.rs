use std::iter::repeat_n;

use rhdl::prelude::*;
use rhdl_fpga::{
    rng::xorshift::XorShift128,
    stream::{
        tee::Tee,
        testing::{sink_from_fn::SinkFromFn, source_from_fn::SourceFromFn, utils::stalling},
    },
};

#[derive(Clone, Synchronous, SynchronousDQ)]
struct TestFixture {
    source: SourceFromFn<(b4, b6)>,
    tee: Tee<b4, b6>,
    s_sink: SinkFromFn<b4>,
    t_sink: SinkFromFn<b6>,
}

impl SynchronousIO for TestFixture {
    type I = ();
    type O = ();
    type Kernel = kernel;
}

#[kernel]
pub fn kernel(_cr: ClockReset, _i: (), q: Q) -> ((), D) {
    let mut d = D::dont_care();
    d.tee.data = q.source;
    d.source = q.tee.ready;
    d.s_sink = q.tee.s_data;
    d.t_sink = q.tee.t_data;
    d.tee.s_ready = q.s_sink;
    d.tee.t_ready = q.t_sink;
    ((), d)
}

fn main() -> Result<(), RHDLError> {
    let a_rng = XorShift128::default().map(|x| {
        let s = b4((x & 0xF) as u128);
        let t = b6(((x >> 8) & 0x3F) as u128);
        (s, t)
    });
    let mut c_rng = a_rng.clone();
    let mut d_rng = a_rng.clone();
    let a_rng = stalling(a_rng, 0.23);
    let consume_s = move |data| {
        if let Some(data) = data {
            let validation = c_rng.next().unwrap();
            assert_eq!(data, validation.0);
        }
        rand::random::<f64>() > 0.2
    };
    let consume_t = move |data| {
        if let Some(data) = data {
            let validation = d_rng.next().unwrap();
            assert_eq!(data, validation.1);
        }
        rand::random::<f64>() > 0.2
    };
    let uut = TestFixture {
        source: SourceFromFn::new(a_rng),
        tee: Tee::default(),
        s_sink: SinkFromFn::new(consume_s),
        t_sink: SinkFromFn::new(consume_t),
    };
    // Run a few samples through
    let input = repeat_n((), 15).with_reset(1).clock_pos_edge(100);
    let vcd = uut.run(input).collect::<Svg>();
    rhdl_fpga::doc::write_svg_as_markdown(
        vcd,
        "tee.md",
        SvgOptions::default().with_filter("(.*tee.input.data)|(.*tee.outputs.data)"),
    )?;
    Ok(())
}
