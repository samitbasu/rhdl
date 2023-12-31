use rhdl_bits::Bits;
use rhdl_core::digital_fn::DigitalFn;
use rhdl_core::kernel::ExternalKernelDef;
use rhdl_core::kernel::KernelFnKind;

pub fn slice<const N: usize, const M: usize>(x: Bits<N>, start: u128) -> Bits<M> {
    assert!(start + M as u128 <= N as u128);
    Bits((x.0 >> start) & Bits::<M>::mask().0)
}

fn vm_slice<const M: usize>(args: &[rhdl_core::TypedBits]) -> anyhow::Result<rhdl_core::TypedBits> {
    args[0].slice(args[1].as_i64()? as usize, M)
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
            vm_stub: Some(vm_slice::<M>),
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

    #[test]
    fn test_iverilog() -> anyhow::Result<()> {
        let test_values = (0..=255).map(Bits::<8>::from).map(|x| (x, x.raw() % 5));
        rhdl_core::test_with_iverilog(
            slice::<8, 3>,
            slice::<8, 3>::kernel_fn().try_into()?,
            test_values,
        )
    }
}
