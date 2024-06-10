use rhdl_bits::Bits;
use rhdl_core::error::RHDLError;
use rhdl_core::kernel::ExternalKernelDef;
use rhdl_core::kernel::KernelFnKind;
use rhdl_core::DigitalFn;

pub fn any<const N: usize>(x: Bits<N>) -> bool {
    (x.0 & Bits::<N>::mask().0) != 0
}

pub fn vm_any(args: &[rhdl_core::TypedBits]) -> Result<rhdl_core::TypedBits, RHDLError> {
    Ok(args[0].any())
}

#[allow(non_camel_case_types)]
pub struct any<const N: usize> {}

impl<const N: usize> DigitalFn for any<N> {
    fn kernel_fn() -> Option<KernelFnKind> {
        Some(KernelFnKind::Extern(ExternalKernelDef {
            name: format!("any_{N}"),
            body: format!(
                "function [{}:0] any_{N}(input [{}:0] a); any_{N} = |a; endfunction",
                N - 1,
                N - 1
            ),
            vm_stub: Some(vm_any),
        }))
    }
}

#[cfg(test)]
mod tests {
    use rhdl_bits::bits;
    use rhdl_core::test_with_iverilog;

    use super::*;

    #[test]
    fn test_any() {
        let bits = Bits::<128>::mask();
        assert!(any(bits));
        let bits = Bits::<32>::mask();
        assert!(any(bits));
        let bits = Bits::<1>::mask();
        assert!(any(bits));
        let bits: Bits<5> = 0b11111.into();
        assert!(any(bits));
        let bits: Bits<5> = 0b00000.into();
        assert!(!any(bits));
    }

    #[test]
    fn test_iverilog() -> Result<(), RHDLError> {
        let test_values = (0..=255).map(bits).map(|x| (x,));
        test_with_iverilog(
            any::<8>,
            any::<8>::kernel_fn().unwrap().try_into()?,
            test_values,
        )
    }
}
