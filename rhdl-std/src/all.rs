use rhdl_bits::Bits;
use rhdl_core::digital_fn::DigitalFn;
use rhdl_core::kernel::ExternalKernelDef;
use rhdl_core::kernel::KernelFnKind;

pub fn all<const N: usize>(x: Bits<N>) -> bool {
    (x.0 & Bits::<N>::mask().0) == Bits::<N>::mask().0
}

#[allow(non_camel_case_types)]
pub struct all<const N: usize> {}

impl<const N: usize> DigitalFn for all<N> {
    fn kernel_fn() -> KernelFnKind {
        KernelFnKind::Extern(ExternalKernelDef {
            name: format!("all_{N}"),
            body: format!(
                "function [{}:0] all_{N}(input [{}:0] a); all_{N} = &a; endfunction",
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
}
