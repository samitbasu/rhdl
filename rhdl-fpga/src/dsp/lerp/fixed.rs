use std::ops::Add;

use rhdl::prelude::*;

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
#[kernel]
pub fn lerp_unsigned<N, M>(lower_value: Bits<N>, upper_value: Bits<N>, factor: Bits<M>) -> Bits<N>
where
    N: BitWidth,
    M: BitWidth + Add<U1>,
    op!(M + U1): BitWidth,
{
    // Convert them to DynBits so we can manipulate them
    let lower_value = lower_value.dyn_bits(); // Size N
    let upper_value = upper_value.dyn_bits(); // Size N
    let factor = factor.dyn_bits(); // Size M
    let signed_factor = factor.xsgn(); // Size M + 1
    let diff = upper_value.xsub(lower_value); // Size N + 1
    let correction = signed_factor.xmul(diff); // Size N + M + 2
    let lower_value = lower_value.xshl::<M>(); // Size N + M
    let lower_value = lower_value.xsgn(); // Size N + M + 1
    let lower_value = lower_value.xext::<U1>(); // Size N + M + 2
    let y = lower_value + correction; // Size N + M + 2 - we can use a wrapped sum here
    let y = y.xshr::<M>(); // Size N + 2
    let y = y.as_unsigned().resize::<N>(); // Size N
    y.as_bits()
}

#[cfg(test)]
mod tests {
    use rhdl::core::sim::testbench::kernel::test_kernel_vm_and_verilog_synchronous;

    use super::*;

    fn lerp_i32(a: i32, b: i32, f: i32, shift: u8) -> i32 {
        ((a << shift) + (b - a) * f) >> shift
    }

    #[test]
    fn test_lerp_exhaustive() {
        for a in 0..16 {
            for b in 0..16 {
                for factor in 0..32 {
                    let x = b4(a);
                    let y = b4(b);
                    let f = b5(factor);
                    // Compute the "right answer", but use integer arithmetic, not floating point.
                    let expected = lerp_i32(a as i32, b as i32, factor as i32, 5) as u128;
                    let expected = expected as u128;
                    assert_eq!(
                        lerp_unsigned(x, y, f).raw(),
                        expected,
                        "{} {} {}",
                        a,
                        b,
                        factor
                    );
                }
            }
        }
    }
    #[test]
    fn test_lerp_kernel() -> miette::Result<()> {
        let vals = (0..16)
            .map(b4)
            .flat_map(|x| (0..16).map(move |y| (x, b4(y))))
            .flat_map(|(x, y)| (0..32).map(move |f| (x, y, b5(f))))
            .collect::<Vec<_>>();
        test_kernel_vm_and_verilog_synchronous::<lerp_unsigned<U4, U5>, _, _, _>(
            lerp_unsigned,
            vals.into_iter(),
        )?;
        Ok(())
    }
}
