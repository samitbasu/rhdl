use rhdl_bits::bits;
use rhdl_bits::Bits;
use rhdl_bits::SignedBits;
use rhdl_core::digital_fn::DigitalFn;
use rhdl_core::kernel::ExternalKernelDef;
use rhdl_core::kernel::KernelFnKind;

pub fn as_unsigned<const N: usize>(x: SignedBits<N>) -> Bits<N> {
    bits((x.0 as u128) & (Bits::<N>::mask().0))
}

#[allow(non_camel_case_types)]
pub struct as_unsigned<const N: usize> {}

impl<const N: usize> DigitalFn for as_unsigned<N> {
    fn kernel_fn() -> KernelFnKind {
        KernelFnKind::Extern(ExternalKernelDef {
            name: format!("unsigned_{N}"),
            body: format!(
                "function [{}:0] unsigned_{N}(input signed [{}:0] a); unsigned_{N} = $unsigned(a); endfunction",
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
    fn test_unsigned() {
        assert_eq!(as_unsigned(SignedBits::<8>(-1)), Bits::<8>(0b1111_1111));
        assert_eq!(as_unsigned(SignedBits::<8>(0)), Bits::<8>(0b0000_0000));
        assert_eq!(as_unsigned(SignedBits::<8>(1)), Bits::<8>(0b0000_0001));
    }
}
