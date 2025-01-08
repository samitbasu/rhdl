use rhdl::prelude::*;

/// A Linear Interpolation unit.  This unit takes a pair of values that and an
/// interpolation factor, and then produces the linear interpolation of the
/// two values.  The interpolation factor is a fixed-point number, with a
/// magnitude strictly less than 1.0.
///
/// Given two input samples A, and B, and a linear interpolation value of x
/// the output is:
///     A * (1 - x) + B * x
/// Or equivalently:
///    A + (B - A) * x
/// Because x is represented as a fixed point number it really represents
///    f * 2^N
/// So we need to calculate
///    A + (B - A) * f * 2^N

/*#[derive(Debug, Clone, Synchronous)]
pub struct U {}
*/

#[derive(PartialEq, Debug, Digital, Default)]
pub struct I<N: BitWidth, const M: usize> {
    /// The value taken when the interpolant is 0
    pub a: SignedBits<N>,
    /// The value taken when the interpolant is maximal
    pub b: SignedBits<N>,
    /// The interpolation value (0 <= x < 1)
    pub x: Bits<M>,
}

#[derive(PartialEq, Debug, Digital, Default)]
pub struct O<N: BitWidth> {
    /// The interpolated value
    pub y: SignedBits<N>,
}

//  Where A = M + 1, B = N + M + 1
//#[kernel]
pub fn lerp_kernel<N: BitWidth, const M: usize, const A: usize>(_cr: ClockReset, i: I<N, M>) -> O<N>
where
    SignedBits<A>: std::ops::Mul<SignedBits<N>, Output = SignedBits<M>>,
    SignedBits<N>: Pad,
{
    let a: SignedBits<N> = i.a.resize();
    let ax = a.pad();
    let b: SignedBits<N> = i.b.resize();
    let delt = b - a;
    let _a = a << (A as u128);
    let x: Bits<A> = i.x.resize();
    let x: SignedBits<A> = x.as_signed();
    let z = x * delt;
    //    let y = (a + delt * x) >> (A as u128);
    O::<N> { y: z.resize() }
}
