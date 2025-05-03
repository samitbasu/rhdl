use rhdl::prelude::*;
use rhdl_fpga::doc::write_svg_as_markdown;
use rhdl_fpga::dsp::lerp::fixed::lerp_unsigned;

// Because the `lerp` function is just a function, if we
// want to make a core out of it, we need a wrapper. In
// this example, we will use the [Func] wrapper.  We
// need to fit the type signature of that function, so the
// inputs of the `lerp` function need to be put into a single
// struct.
#[derive(PartialEq, Digital)]
pub struct LerpIn<N, M>
where
    N: BitWidth,
    M: BitWidth,
{
    pub lower_value: Bits<N>,
    pub upper_value: Bits<N>,
    pub factor: Bits<M>,
}

// An wrapper function to call the `lerp_unsigned`
#[kernel]
pub fn wrap_lerp<N, M>(_cr: ClockReset, i: LerpIn<N, M>) -> Bits<N>
where
    N: BitWidth,
    M: BitWidth,
{
    lerp_unsigned::<N, M>(i.lower_value, i.upper_value, i.factor)
}

fn main() -> Result<(), RHDLError> {
    // The [Func] wrapper gives us a core we can simulate
    let uut: Func<LerpIn<U8, U4>, Bits<U8>> = Func::new::<wrap_lerp<U8, U4>>()?;
    // Simulate a ramp
    let ramp = (0..15)
        .map(|x| LerpIn {
            upper_value: bits(255),
            lower_value: bits(0),
            factor: bits(x),
        })
        .stream()
        .clock_pos_edge(100);
    let vcd = uut.run(ramp)?.collect::<Vcd>();
    write_svg_as_markdown(vcd, "lerp.md", SvgOptions::default())?;
    Ok(())
}
