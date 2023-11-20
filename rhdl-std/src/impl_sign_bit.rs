use rhdl_bits::SignedBits;
use rhdl_core::{
    digital_fn::DigitalFn,
    kernel::{ExternalKernelDef, KernelFnKind},
};

pub fn sign_bit<const N: usize>(x: SignedBits<N>) -> bool {
    (x.0 >> (N - 1)) & 1 == 1
}

#[allow(non_camel_case_types)]
pub struct sign_bit<const N: usize> {}

impl<const N: usize> DigitalFn for sign_bit<N> {
    fn kernel_fn() -> KernelFnKind {
        KernelFnKind::Extern(ExternalKernelDef {
            name: format!("sign_bit_{N}"),
            body: format!(
                "function [0:0] sign_bit_{N}(input signed [{}:0] a); sign_bit_{N} = a[{}]; endfunction",
                N - 1,
                N - 1,
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_bit() {
        assert!(sign_bit(SignedBits::<8>(-1)));
        assert!(!sign_bit(SignedBits::<8>(0)));
        assert!(!sign_bit(SignedBits::<8>(1)));
    }
}
