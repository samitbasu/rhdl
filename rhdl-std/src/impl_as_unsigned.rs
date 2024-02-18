use rhdl_bits::bits;
use rhdl_bits::Bits;
use rhdl_bits::SignedBits;
use rhdl_core::kernel::ExternalKernelDef;
use rhdl_core::kernel::KernelFnKind;
use rhdl_core::DigitalFn;

pub fn as_unsigned<const N: usize>(x: SignedBits<N>) -> Bits<N> {
    bits((x.0 as u128) & (Bits::<N>::mask().0))
}

fn vm_as_unsigned(args: &[rhdl_core::TypedBits]) -> anyhow::Result<rhdl_core::TypedBits> {
    args[0].as_unsigned()
}

#[allow(non_camel_case_types)]
pub struct as_unsigned<const N: usize> {}

impl<const N: usize> DigitalFn for as_unsigned<N> {
    fn kernel_fn() -> Option<KernelFnKind> {
        Some(KernelFnKind::Extern(ExternalKernelDef {
            name: format!("unsigned_{N}"),
            body: format!(
                "function [{}:0] unsigned_{N}(input signed [{}:0] a); unsigned_{N} = $unsigned(a); endfunction",
                N - 1,
                N - 1,
            ),
            vm_stub: Some(vm_as_unsigned),
        }))
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

    #[test]
    fn test_iverilog() -> anyhow::Result<()> {
        let test_values = (-128..=127).map(SignedBits::<8>::from).map(|x| (x,));
        rhdl_core::test_with_iverilog(
            as_unsigned::<8>,
            as_unsigned::<8>::kernel_fn().unwrap().try_into()?,
            test_values,
        )
    }
}
