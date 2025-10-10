use rhdl::prelude::*;

use std::ops::Add;

use rhdl_fpga::core::{
    constant, dff,
    option::unpack,
    slice::{lsbs, msbs},
};

// Start with a 2x Reducer
#[derive(Debug, PartialEq, Digital, Default, Clone)]
pub enum State {
    #[default]
    Empty,
    Load1,
    Load2,
}

#[derive(Debug, Clone, Synchronous, SynchronousDQ)]
pub struct U<DW: BitWidth, DN: BitWidth> {
    state: dff::DFF<State>,
    data_store: dff::DFF<Bits<DW>>,
    // TODO - make a marker?
    _unused: constant::Constant<Bits<DN>>,
}

impl<W, N> Default for U<W, N>
where
    W: BitWidth,
    N: Add<N>,
    W: IsEqualTo<Sum<N, N>>,
    N: BitWidth,
{
    fn default() -> Self {
        Self {
            state: dff::DFF::<State>::default(),
            data_store: dff::DFF::<Bits<W>>::default(),
            _unused: constant::Constant::<Bits<N>>::new(bits(0)),
        }
    }
}

#[derive(Debug, PartialEq, Digital, Clone)]
pub struct I<DW: BitWidth> {
    pub data: Option<Bits<DW>>,
    pub ready: bool,
}

#[derive(Debug, PartialEq, Digital, Clone)]
pub struct O<DN: BitWidth> {
    pub data: Option<Bits<DN>>,
    pub ready: bool,
}

impl<DW: BitWidth, DN: BitWidth> SynchronousIO for U<DW, DN> {
    type I = I<DW>;
    type O = O<DN>;
    type Kernel = kernel<DW, DN>;
}

#[kernel]
pub fn kernel<DW: BitWidth, DN: BitWidth>(
    _cr: ClockReset,
    i: I<DW>,
    q: Q<DW, DN>,
) -> (O<DN>, D<DW, DN>) {
    let mut d = D::<DW, DN>::dont_care();
    // Latch prevention
    d.state = q.state;
    d.data_store = q.data_store;
    let (in_valid, in_data) = unpack::<Bits<DW>>(i.data, bits(0));
    let stop_in = !i.ready;
    match q.state {
        State::Empty => {
            if in_valid {
                d.data_store = in_data;
                d.state = State::Load2;
            }
        }
        State::Load2 => {
            if !stop_in {
                d.state = State::Load1;
            }
        }
        State::Load1 => {
            if !stop_in && in_valid {
                d.data_store = in_data;
                d.state = State::Load2;
            } else if !stop_in && !in_valid {
                d.state = State::Empty;
            }
        }
    }
    // This is a combinatorial pathway between the output and input, so
    // a buffer is needed on the output to make this LID compliant.
    let ready_out = q.state == State::Empty || (q.state == State::Load1 && !stop_in);
    let mux = q.state == State::Load1;
    let output_valid = q.state != State::Empty;
    let mut o = O::<DN>::dont_care();
    let mux_output = if !mux {
        lsbs::<DN, DW>(q.data_store)
    } else {
        msbs::<DN, DW>(q.data_store)
    };
    o.data = if output_valid { Some(mux_output) } else { None };
    o.ready = ready_out;
    (o, d)
}

#[test]
fn test_no_combinatorial_paths() -> miette::Result<()> {
    let uut = U::<U16, U8>::default();
    let res = drc::no_combinatorial_paths(&uut);
    let Err(err) = res else {
        panic!("Expected this to fail");
    };
    let handler =
        miette::GraphicalReportHandler::new_themed(miette::GraphicalTheme::unicode_nocolor());
    let mut msg = String::new();
    handler.render_report(&mut msg, err.as_ref()).unwrap();
    expect_test::expect_file!["faulty_reducer_no_combinatorial_paths.expect"].assert_eq(&msg);
    Ok(())
}
