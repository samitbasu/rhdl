use rhdl_bits::Bits;
use rhdl_core::digital_fn::DigitalFn;
use rhdl_core::kernel::ExternalKernelDef;
use rhdl_core::kernel::KernelFnKind;

pub fn get_bit<const N: usize>(x: Bits<N>, i: u128) -> bool {
    (x.0 >> i) & 1 == 1
}

#[allow(non_camel_case_types)]
pub struct get_bit<const N: usize> {}

impl<const N: usize> DigitalFn for get_bit<N> {
    fn kernel_fn() -> KernelFnKind {
        KernelFnKind::Extern(ExternalKernelDef {
            name: format!("get_bit_{N}"),
            body: format!(
                "function [0:0] get_bit_{N}(input [{}:0] a, input integer i); get_bit_{N} = a[i]; endfunction",
                N - 1,
            ),
        })
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
}
