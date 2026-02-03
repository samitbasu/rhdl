//! Functions to slice bits
//!
//! This module provides some synthesizable functions to
//! extract either the MSBs or LSBs of a bitvector.  The
//! code may not look efficient, but it optimizes away
//! when generating HDL.
use rhdl::prelude::*;

#[kernel]
/// Return the `N` LSBs of a bitvector of length `M`.  If
/// `N >= M`, then the upper bits will be zero filled.
pub fn lsbs<const N: usize, const M: usize>(n: Bits<M>) -> Bits<N>
where
    rhdl::bits::W<N>: BitWidth,
    rhdl::bits::W<M>: BitWidth,
{
    let mut o = bits(0);
    for i in 0..N {
        if n & (1 << i) != 0 {
            o |= 1 << i
        }
    }
    o
}

#[kernel]
/// Return the `N` MSBs of a bitvector of length `M`.  If
/// `N >= M`, then the lower bits of the output will be
/// zero filled.
pub fn msbs<const N: usize, const M: usize>(n: Bits<M>) -> Bits<N>
where
    rhdl::bits::W<N>: BitWidth,
    rhdl::bits::W<M>: BitWidth,
{
    let mut o = bits(0);
    for i in 0..N {
        if n & (1 << (M - N + i)) != 0 {
            o |= 1 << i
        }
    }
    o
}

#[kernel]
/// Return the `N` bits of a bitvector of length `M` starting
/// at bit position `P`.  If `P + N > M`, then the upper bits
/// will be zero filled.
pub fn slice<const N: usize, const M: usize, const P: usize>(n: Bits<M>) -> Bits<N>
where
    rhdl::bits::W<N>: BitWidth,
    rhdl::bits::W<M>: BitWidth,
{
    let mut o = bits(0);
    for i in 0..N {
        if n & (1 << (P + i)) != 0 {
            o |= 1 << i
        }
    }
    o
}

#[cfg(test)]
mod tests {
    use rhdl::core::{compiler::optimize_ntl, ntl::from_rtl::build_ntl_from_rtl};

    use super::*;

    #[test]
    fn test_msbs_works() {
        let n = 0xDEAD_BEEF_u128;
        let n = b32(n);
        let h = msbs::<16, 32>(n);
        assert_eq!(h, 0xDEAD);
        let l = lsbs::<16, 32>(n);
        assert_eq!(l, 0xBEEF);
    }

    #[test]
    fn test_slice_works() {
        let n = 0xDEAD_BEEF_u128;
        let n = b32(n);
        let s = slice::<16, 32, 8>(n);
        assert_eq!(s, 0xADBE);
    }

    #[test]
    fn test_slice_generated_code() -> miette::Result<()> {
        let hdl = compile_design::<slice<2, 6, 2>>(CompilationMode::Asynchronous)?;
        eprintln!("{}", hdl.as_vlog()?.pretty());
        let ntl = build_ntl_from_rtl(&hdl);
        let ntl = optimize_ntl(ntl)?;
        let ntl = ntl.as_vlog("slice")?.modules.pretty();
        expect_test::expect![[r#"
            module slice(input wire [5:0] arg_0, output reg [1:0] out);
               reg  r0;
               reg  r1;
               reg  r2;
               reg  r3;
               reg  r4;
               reg  r5;
               always @(*) begin
                  r0 = arg_0[0];
                  r1 = arg_0[1];
                  r2 = arg_0[2];
                  r3 = arg_0[3];
                  r4 = arg_0[4];
                  r5 = arg_0[5];
                  out = {r3, r2};
               end
            endmodule
        "#]].assert_eq(&ntl);
        Ok(())
    }
}
