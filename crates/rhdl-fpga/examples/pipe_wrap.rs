use std::iter::repeat_n;

use badascii_doc::badascii;
use rhdl::prelude::*;

use delay::DelayLine;
use rhdl_fpga::{
    core::slice::lsbs,
    rng::xorshift::XorShift128,
    stream::{
        pipe_wrapper::PipeWrapper,
        testing::{sink_from_fn::SinkFromFn, source_from_fn::SourceFromFn, utils::stalling},
    },
};

pub mod delay {
    use rhdl_fpga::core::{
        dff::DFF,
        option::{pack, unpack},
        slice::lsbs,
    };

    use super::*;
    #[derive(Clone, Synchronous, SynchronousDQ, Default)]
    pub struct DelayLine {
        stage_0: DFF<Option<b6>>,
        stage_1: DFF<Option<b6>>,
        stage_2: DFF<Option<b4>>,
    }

    impl SynchronousIO for DelayLine {
        type I = Option<b6>;
        type O = Option<b4>;
        type Kernel = kernel;
    }

    #[kernel]
    pub fn kernel(_cr: ClockReset, i: Option<b6>, q: Q) -> (Option<b4>, D) {
        let mut d = D::dont_care();
        d.stage_0 = i;
        d.stage_1 = q.stage_0;
        let (tag, data) = unpack::<b6>(q.stage_1, bits(0));
        let data = lsbs::<U4, U6>(data);
        d.stage_2 = pack::<b4>(tag, data);
        (q.stage_2, d)
    }
}

///
/// Here is a sketch of the internals:
///
#[doc = badascii!(r"
+Source+-+    +Wrapper+-----+     +Sink+--+
|        | ?T |             | ?S  |       |
|    data+--->|data     data+---->|data   |
|        |    |             |     |       |
|   ready|<---+ready   ready|<----+ready  |
+--------+    +--+------+---+     +-------+
           +-----+      +----+             
         ?T|  +------------+ |?S           
           +->|in       out+-+             
              +------------+               
")]
#[derive(Clone, Synchronous, SynchronousDQ)]
struct TestFixture {
    source: SourceFromFn<b6>,
    delay: DelayLine,
    wrapper: PipeWrapper<b6, b4, U2>,
    sink: SinkFromFn<b4>,
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
    d.wrapper.data = q.source;
    d.source = q.wrapper.ready;
    d.sink = q.wrapper.data;
    d.wrapper.ready = q.sink;
    d.delay = q.wrapper.to_pipe;
    d.wrapper.from_pipe = q.delay;
    ((), d)
}

fn main() -> Result<(), RHDLError> {
    let b_rng = XorShift128::default().map(|x| b6(((x >> 8) & 0x3F) as u128));
    let mut c_rng = b_rng.clone();
    let b_rng = stalling(b_rng, 0.13);
    let consume = move |data| {
        if let Some(data) = data {
            let validation = lsbs::<U4, U6>(c_rng.next().unwrap());
            assert_eq!(data, validation);
        }
        rand::random::<f64>() > 0.2
    };
    let uut = TestFixture {
        source: SourceFromFn::new(b_rng),
        delay: DelayLine::default(),
        wrapper: PipeWrapper::default(),
        sink: SinkFromFn::new(consume),
    };
    // Run a few samples through
    let input = repeat_n((), 15).with_reset(1).clock_pos_edge(100);
    let vcd = uut.run_without_synthesis(input)?.collect::<Vcd>();
    rhdl_fpga::doc::write_svg_as_markdown(
        vcd,
        "pipe_wrap.md",
        SvgOptions::default().with_filter("(.*wrapper.input.data)|(.*wrapper.outputs.data)"),
    )?;
    Ok(())
}
