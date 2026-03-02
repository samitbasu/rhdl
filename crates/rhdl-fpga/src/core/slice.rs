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
        expect_test::expect![[r#"
            function [1:0] kernel_slice(input reg [5:0] arg_0);
                  reg [5:0] r0;
                  reg [5:0] r1;
                  reg [0:0] r2;
                  // o
                  reg [1:0] r3;
                  reg [5:0] r4;
                  reg [0:0] r5;
                  reg [1:0] r6;
                  // o
                  reg [1:0] r7;
                  localparam l0 = 6'b000100;
                  localparam l1 = 2'b01;
                  localparam l2 = 2'b00;
                  localparam l3 = 6'b001000;
                  localparam l4 = 2'b10;
                  begin
                     r1 = arg_0;
                     r0 = r1 & l0;
                     r2 = |r0;
                     r3 = r2 ? l1 : l2;
                     r4 = r1 & l3;
                     r5 = |r4;
                     r6 = r3 | l4;
                     r7 = r5 ? r6 : r3;
                     kernel_slice = r7;
                  end
            endfunction"#]].assert_eq(&hdl.as_vlog()?.pretty());
        Ok(())
    }
}
