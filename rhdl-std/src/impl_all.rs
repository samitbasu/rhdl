use rhdl_bits::Bits;
use rhdl_core::kernel::ExternalKernelDef;
use rhdl_core::kernel::KernelFnKind;
use rhdl_core::DigitalFn;
use rhdl_core::TypedBits;

pub fn all<const N: usize>(x: Bits<N>) -> bool {
    (x.0 & Bits::<N>::mask().0) == Bits::<N>::mask().0
}

#[allow(non_camel_case_types)]
pub struct all<const N: usize> {}

fn vm_all(args: &[TypedBits]) -> anyhow::Result<TypedBits> {
    Ok(args[0].all())
}

impl<const N: usize> DigitalFn for all<N> {
    fn kernel_fn() -> Option<KernelFnKind> {
        Some(KernelFnKind::Extern(ExternalKernelDef {
            name: format!("all_{N}"),
            body: format!(
                "function [{}:0] all_{N}(input [{}:0] a); all_{N} = &a; endfunction",
                N - 1,
                N - 1
            ),
            vm_stub: Some(vm_all),
        }))
    }
}

#[cfg(test)]
mod tests {
    use rhdl_bits::bits;
    use rhdl_core::test_with_iverilog;

    use super::*;
    #[test]
    fn test_all() {
        let bits = Bits::<128>::mask();
        assert!(all(bits));
        let bits = Bits::<32>::mask();
        assert!(all(bits));
        let bits = Bits::<1>::mask();
        assert!(all(bits));
        let bits: Bits<5> = 0b11111.into();
        assert!(all(bits));
        let bits: Bits<5> = 0b11110.into();
        assert!(!all(bits));
    }

    #[test]
    fn test_iverilog() -> anyhow::Result<()> {
        let test_values = (0..=255).map(bits).map(|x| (x,));
        test_with_iverilog(
            all::<8>,
            all::<8>::kernel_fn().unwrap().try_into()?,
            test_values,
        )
    }
}
