use rhdl_bits::Bits;
use rhdl_bits::SignedBits;
use rhdl_core::digital_fn::DigitalFn;
use rhdl_core::kernel::ExternalKernelDef;
use rhdl_core::kernel::KernelFnKind;

use crate::impl_get_bit::get_bit;

pub fn as_signed<const N: usize>(x: Bits<N>) -> SignedBits<N> {
    // Need a sign extension here.
    if get_bit(x, N as u128 - 1) {
        SignedBits((x.0 | !(Bits::<N>::mask().0)) as i128)
    } else {
        SignedBits(x.0 as i128)
    }
}

#[allow(non_camel_case_types)]
pub struct as_signed<const N: usize> {}

impl<const N: usize> DigitalFn for as_signed<N> {
    fn kernel_fn() -> KernelFnKind {
        KernelFnKind::Extern(ExternalKernelDef {
            name: format!("signed_{N}"),
            body: format!(
                "function signed [{}:0] signed_{N}(input [{}:0] a); signed_{N} = $signed(a); endfunction",
                N - 1,
                N - 1,
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signed() {
        assert_eq!(as_signed(Bits::<8>(0b1111_1111)), SignedBits::<8>(-1));
        assert_eq!(as_signed(Bits::<8>(0b0000_0000)), SignedBits::<8>(0));
        assert_eq!(as_signed(Bits::<8>(0b0000_0001)), SignedBits::<8>(1));
    }
}
