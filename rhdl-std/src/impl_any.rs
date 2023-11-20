use rhdl_bits::Bits;
use rhdl_core::digital_fn::DigitalFn;
use rhdl_core::kernel::ExternalKernelDef;
use rhdl_core::kernel::KernelFnKind;

pub fn any<const N: usize>(x: Bits<N>) -> bool {
    (x.0 & Bits::<N>::mask().0) != 0
}

#[allow(non_camel_case_types)]
pub struct any<const N: usize> {}

impl<const N: usize> DigitalFn for any<N> {
    fn kernel_fn() -> KernelFnKind {
        KernelFnKind::Extern(ExternalKernelDef {
            name: format!("any_{N}"),
            body: format!(
                "function [{}:0] any_{N}(input [{}:0] a); any_{N} = |a; endfunction",
                N - 1,
                N - 1
            ),
        })
    }
}

#[cfg(test)]
mod tests {
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
}
