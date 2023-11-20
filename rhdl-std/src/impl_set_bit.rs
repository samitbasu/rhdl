use rhdl_bits::bits;
use rhdl_bits::Bits;
use rhdl_core::digital_fn::DigitalFn;
use rhdl_core::kernel::ExternalKernelDef;
use rhdl_core::kernel::KernelFnKind;

pub fn set_bit<const N: usize>(x: Bits<N>, i: u128, value: bool) -> Bits<N> {
    let selector = 1_u128 << i;
    let x = if value {
        x.0 | selector
    } else {
        x.0 & !selector
    };
    bits(x)
}

#[allow(non_camel_case_types)]
pub struct set_bit<const N: usize> {}

impl<const N: usize> DigitalFn for set_bit<N> {
    fn kernel_fn() -> KernelFnKind {
        KernelFnKind::Extern(ExternalKernelDef {
            name: format!("set_bit_{N}"),
            body: format!(
                "function [{}:0] set_bit_{N}(input [{}:0] a, input integer i, input [0:0] value); set_bit_{N} = value ? a | (1 << i) : a & ~(1 << i); endfunction",
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
    fn test_set_bit() {
        let mut bits = Bits::<128>::mask();
        bits = set_bit(bits, 0, false);
        assert_eq!(bits.0, u128::MAX - 1);
        bits = set_bit(bits, 0, true);
        assert_eq!(bits.0, u128::MAX);
        bits = set_bit(bits, 127, false);
        assert_eq!(bits.0, u128::MAX - (1 << 127));
        bits = set_bit(bits, 127, true);
        assert_eq!(bits.0, u128::MAX);
        bits = set_bit(bits, 64, false);
        assert_eq!(bits.0, u128::MAX - (1 << 64));
        bits = set_bit(bits, 64, true);
        assert_eq!(bits.0, u128::MAX);
        bits = set_bit(bits, 32, false);
        assert_eq!(bits.0, u128::MAX - (1 << 32));
        bits = set_bit(bits, 32, true);
        assert_eq!(bits.0, u128::MAX);
        bits = set_bit(bits, 16, false);
        assert_eq!(bits.0, u128::MAX - (1 << 16));
        bits = set_bit(bits, 16, true);
        assert_eq!(bits.0, u128::MAX);
        bits = set_bit(bits, 8, false);
        assert_eq!(bits.0, u128::MAX - (1 << 8));
        bits = set_bit(bits, 8, true);
        assert_eq!(bits.0, u128::MAX);
        bits = set_bit(bits, 4, false);
        assert_eq!(bits.0, u128::MAX - (1 << 4));
        bits = set_bit(bits, 4, true);
        assert_eq!(bits.0, u128::MAX);
        bits = set_bit(bits, 2, false);
        assert_eq!(bits.0, u128::MAX - (1 << 2));
        bits = set_bit(bits, 2, true);
        assert_eq!(bits.0, u128::MAX);
        bits = set_bit(bits, 1, false);
        assert_eq!(bits.0, u128::MAX - (1 << 1));
        bits = set_bit(bits, 1, true);
        assert_eq!(bits.0, u128::MAX);
    }
}
