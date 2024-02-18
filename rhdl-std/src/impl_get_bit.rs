use rhdl_bits::Bits;
use rhdl_core::kernel::ExternalKernelDef;
use rhdl_core::kernel::KernelFnKind;
use rhdl_core::DigitalFn;

pub fn get_bit<const N: usize>(x: Bits<N>, i: u8) -> bool {
    (x.0 >> i) & 1 == 1
}

fn vm_get_bit(args: &[rhdl_core::TypedBits]) -> anyhow::Result<rhdl_core::TypedBits> {
    args[0].get_bit(args[1].as_i64()? as usize)
}

#[allow(non_camel_case_types)]
pub struct get_bit<const N: usize> {}

impl<const N: usize> DigitalFn for get_bit<N> {
    fn kernel_fn() -> Option<KernelFnKind> {
        Some(KernelFnKind::Extern(ExternalKernelDef {
            name: format!("get_bit_{N}"),
            body: format!(
                "function [0:0] get_bit_{N}(input [{}:0] a, input integer i); get_bit_{N} = a[i]; endfunction",
                N - 1,
            ),
            vm_stub: Some(vm_get_bit),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_bit() {
        let bits = Bits::<128>::mask();
        assert!(get_bit(bits, 0));
        assert!(get_bit(bits, 127));
        assert!(get_bit(bits, 64));
        assert!(get_bit(bits, 32));
        assert!(get_bit(bits, 16));
        assert!(get_bit(bits, 8));
        assert!(get_bit(bits, 4));
        assert!(get_bit(bits, 2));
        assert!(get_bit(bits, 1));
        let bits = Bits::<32>::mask();
        assert!(get_bit(bits, 0));
        assert!(get_bit(bits, 31));
        assert!(get_bit(bits, 16));
        assert!(get_bit(bits, 8));
        assert!(get_bit(bits, 4));
        assert!(get_bit(bits, 2));
        assert!(get_bit(bits, 1));
        let bits = Bits::<1>::mask();
        assert!(get_bit(bits, 0));
        let bits: Bits<5> = 0b11010.into();
        assert!(get_bit(bits, 4));
        assert!(get_bit(bits, 3));
        assert!(!get_bit(bits, 2));
        assert!(get_bit(bits, 1));
        assert!(!get_bit(bits, 0));
    }

    #[test]
    fn test_iverilog() -> anyhow::Result<()> {
        let test_values = (0..=255).map(|x| (Bits::<8>::from(x), (x % 8) as u8));
        rhdl_core::test_with_iverilog(
            get_bit::<8>,
            get_bit::<8>::kernel_fn().unwrap().try_into()?,
            test_values,
        )
    }
}
