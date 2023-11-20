use rhdl_bits::Bits;
use rhdl_core::digital_fn::DigitalFn;
use rhdl_core::kernel::ExternalKernelDef;
use rhdl_core::kernel::KernelFnKind;

pub fn slice<const N: usize, const M: usize>(x: Bits<N>, start: usize) -> Bits<M> {
    Bits((x.0 >> start) & Bits::<M>::mask().0)
}

#[allow(non_camel_case_types)]
pub struct slice<const N: usize, const M: usize> {}

impl<const N: usize, const M: usize> DigitalFn for slice<N, M> {
    fn kernel_fn() -> KernelFnKind {
        KernelFnKind::Extern(ExternalKernelDef {
            name: format!("slice_{N}_{M}"),
            body: format!(
                "function [{}:0] slice_{N}_{M}(input [{}:0] a, input integer start); slice_{N}_{M} = a[start+:{M}]; endfunction",
                M - 1,
                N - 1,
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slice() {
        let bits: Bits<8> = 0b1101_1010.into();
        let result = slice::<8, 4>(bits, 0);
        assert_eq!(result.0, 0b1010);
        let result = slice::<8, 4>(bits, 4);
        assert_eq!(result.0, 0b1101);
        let result = slice::<8, 2>(bits, 6);
        assert_eq!(result.0, 0b11);
    }
}
