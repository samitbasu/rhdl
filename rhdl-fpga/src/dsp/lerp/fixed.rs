use std::ops::Add;

use rhdl::prelude::*;

use crate::cdc::synchronizer::S;

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


/*

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
*/

/// Linear interpolation as a function - for unsigned values
///
/// Given A: Bits<N>, B: Bits<N> and a factor: Bits<M>,
///
/// we want to compute
///
/// A * (1 - delta) + B * delta = Y
///
/// where delta = factor / 2^M
///
/// Substituting delta, we get
///
/// A * ( 1 - factor / 2^M) + B * factor / 2^M = Y
///
/// Multiplying out by 2^M, we get
///
/// A * 2^M - A * factor + B * factor = Y * 2^M
///
/// To get this into a single multiplication, we need
///
/// A * 2^M + (B - A) * factor = Y * 2^M
///
/// The  B - A term is signed, so we need to promote the factor to be signed as well
///
/// A * 2^M + Diff * signed_factor = Y * 2^M
///
/// Here signed_factor will be M+1 bits wide, and E will be N+1 bits wide
///
/// The product will thus be M+N+2 bits wide.  The factor A * 2^M will be N+M bits wide,
/// and is unsigned.  So we need to convert it to a signed value (which adds 1 bit) and
/// then extend it (signed) by a bit.
///
/// We can (after adding it), right shift by M bits to retrieve Y, and then
/// truncate the value to N bits, and cast as unsigned.
pub fn lerp_unsigned<N, M>(lower_value: Bits<N>, upper_value: Bits<N>, factor: Bits<M>) -> Bits<N>
where
    // We need to +be able to add 1 to the width of the interpolation factor
    N: BitWidth + Add<U1> + Add<M>,
    M: BitWidth + Add<U1>,
    op!(M + U1): BitWidth + Add<op!(N + U1)>,
    op!(N + U1): BitWidth,
    op!((M + U1) + (N + U1)): BitWidth + Add<op!(N + M + U1 + U1)>,
    op!(N + M): BitWidth + Add<U1>,
    op!(N + M + U1): BitWidth + Add<U1>,
    op!(N + M + U1 + U1): BitWidth,
    /*
        // We also need to be able to add 1 to the width of the values
        N: BitWidth + Add<U1> + Add<M>,
        op!(N + U1): BitWidth,
        // We also need M+U1 and N+U1 to be multiplicatively compatible
        op!(M + U1): Add<Sum<N, U1>>,
        Sum<Sum<M, U1>, Sum<N, U1>>: BitWidth,
        // We need N+M to be a thing
        Sum<N, M>: BitWidth + Add<U1>,
        // We need N+M+1 to be a thing also
        Sum<Sum<N, M>, U1>: BitWidth + Add<U1>,
        // As well as N+M+2
        Sum<Sum<Sum<N, M>, U1>, U1>: BitWidth,
    */
{
    // Signed factor is signed M+1 bits wide
    let signed_factor = factor.xsgn();
    // Compute B - A.  This will also be signed of width N+1
    let diff = upper_value.xsub(lower_value);
    // Compute signed_factor * B - A = correction.  This has size M+1 + N+1 = M+N+2
    let correction = signed_factor.xmul(diff);
    // Shift the lower value by M bits to the left
    let lower_value = lower_value.xshl::<M>();
    // Convert it to a signed value so we can add the correction (requires an additional bit)
    let lower_value = lower_value.xsgn().xext::<U1>();
    // Compute the correction - we do not need overflow on this, so a regular add (wrapping) is fine
    let y = lower_value + correction;

    /*
       // Calculate the product.  This will be signed of width N+M+2
       let product = signed_factor.xmul(diff);

       let b_frac = upper_value.xmul(factor);
       let a_frac = lower_value.xmul(factor);

       // This is now N+1 bits wide and signed
       let delta = upper_value.xsub(lower_value);
       // We need the lower value to also be N+1 bits wide and signed
       let lower_value = lower_value.xext::<U1>().as_signed();
       // Next, we will multiply delta times the factor.  The result is M+N+1 bits wide
       let delta_times_factor = delta.xmul(factor);
    */
    todo!()
}
