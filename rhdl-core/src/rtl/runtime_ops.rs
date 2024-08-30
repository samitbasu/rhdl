use crate::{
    dyn_bit_manip::{
        bit_neg, bits_and, bits_or, bits_shl, bits_shr, bits_shr_signed, bits_xor, from_bigint,
        from_biguint, full_add, full_sub, to_bigint, to_biguint,
    },
    rhif::spec::{AluBinary, AluUnary},
    types::error::DynamicTypeError,
    RHDLError,
};

use super::object::BitString;

fn bs_copy_sign(bits: Vec<bool>, arg: &BitString) -> BitString {
    match arg {
        BitString::Signed(_) => BitString::Signed(bits),
        BitString::Unsigned(_) => BitString::Unsigned(bits),
    }
}

fn get_shift(arg: &BitString) -> i64 {
    assert!(arg.len() < 63);
    let mut accum: i64 = 0;
    for bit in arg.bits().iter().rev() {
        accum <<= 1;
        accum |= *bit as i64;
    }
    accum
}

pub fn binary(op: AluBinary, arg1: &BitString, arg2: &BitString) -> Result<BitString, RHDLError> {
    if !op.is_shift() && (arg1.is_signed() ^ arg2.is_signed()) {
        return Err(RHDLError::RHDLDynamicTypeError(Box::new(
            DynamicTypeError::BinaryOperationRequiresCompatibleSign,
        )));
    }
    match op {
        AluBinary::Add => Ok(bs_copy_sign(full_add(arg1.bits(), arg2.bits()), arg1)),
        AluBinary::Sub => Ok(bs_copy_sign(full_sub(arg1.bits(), arg2.bits()), arg1)),
        AluBinary::BitXor => Ok(bs_copy_sign(bits_xor(arg1.bits(), arg2.bits()), arg1)),
        AluBinary::BitAnd => Ok(bs_copy_sign(bits_and(arg1.bits(), arg2.bits()), arg1)),
        AluBinary::BitOr => Ok(bs_copy_sign(bits_or(arg1.bits(), arg2.bits()), arg1)),
        AluBinary::Eq => Ok(BitString::Unsigned(vec![arg1.bits() == arg2.bits()])),
        AluBinary::Ne => Ok(BitString::Unsigned(vec![arg1.bits() != arg2.bits()])),
        AluBinary::Shl => Ok(bs_copy_sign(bits_shl(arg1.bits(), get_shift(arg2)), arg1)),
        AluBinary::Shr => {
            if arg1.is_signed() {
                Ok(bs_copy_sign(
                    bits_shr_signed(arg1.bits(), get_shift(arg2)),
                    arg1,
                ))
            } else {
                Ok(bs_copy_sign(bits_shr(arg1.bits(), get_shift(arg2)), arg1))
            }
        }
        AluBinary::Lt => {
            if arg1.is_signed() {
                Ok(BitString::Unsigned(vec![
                    to_bigint(arg1.bits()) < to_bigint(arg2.bits()),
                ]))
            } else {
                Ok(BitString::Unsigned(vec![
                    to_biguint(arg1.bits()) < to_biguint(arg2.bits()),
                ]))
            }
        }
        AluBinary::Le => {
            if arg1.is_signed() {
                Ok(BitString::Unsigned(vec![
                    to_bigint(arg1.bits()) <= to_bigint(arg2.bits()),
                ]))
            } else {
                Ok(BitString::Unsigned(vec![
                    to_biguint(arg1.bits()) <= to_biguint(arg2.bits()),
                ]))
            }
        }
        AluBinary::Gt => {
            if arg1.is_signed() {
                Ok(BitString::Unsigned(vec![
                    to_bigint(arg1.bits()) > to_bigint(arg2.bits()),
                ]))
            } else {
                Ok(BitString::Unsigned(vec![
                    to_biguint(arg1.bits()) > to_biguint(arg2.bits()),
                ]))
            }
        }
        AluBinary::Ge => {
            if arg1.is_signed() {
                Ok(BitString::Unsigned(vec![
                    to_bigint(arg1.bits()) >= to_bigint(arg2.bits()),
                ]))
            } else {
                Ok(BitString::Unsigned(vec![
                    to_biguint(arg1.bits()) >= to_biguint(arg2.bits()),
                ]))
            }
        }
        AluBinary::Mul => {
            if arg1.is_signed() {
                let a_bi = to_bigint(arg1.bits());
                let b_bi = to_bigint(arg2.bits());
                let result = a_bi * b_bi;
                Ok(BitString::Signed(from_bigint(
                    &result,
                    arg1.len() + arg2.len(),
                )))
            } else {
                let a_bi = to_biguint(arg1.bits());
                let b_bi = to_biguint(arg2.bits());
                let result = a_bi * b_bi;
                Ok(BitString::Unsigned(from_biguint(
                    &result,
                    arg1.len() + arg2.len(),
                )))
            }
        }
    }
}

pub fn unary(op: AluUnary, arg1: BitString) -> Result<BitString, RHDLError> {
    match op {
        AluUnary::Not => Ok(BitString::Unsigned(
            arg1.bits().iter().map(|b| !b).collect(),
        )),
        AluUnary::Neg => Ok(BitString::Signed(bit_neg(arg1.bits()))),
        AluUnary::All => Ok(BitString::Unsigned(vec![arg1.bits().iter().all(|b| *b)])),
        AluUnary::Any => Ok(BitString::Unsigned(vec![arg1.bits().iter().any(|b| *b)])),
        AluUnary::Signed => Ok(BitString::Signed(arg1.bits().to_owned())),
        AluUnary::Unsigned => Ok(BitString::Unsigned(arg1.bits().to_owned())),
        AluUnary::Xor => Ok(BitString::Unsigned(vec![arg1
            .bits()
            .iter()
            .fold(false, |a, b| a ^ b)])),
        AluUnary::Val => Ok(arg1),
    }
}

#[cfg(test)]
mod tests {
    use crate::{Kind, TypedBits};

    use super::*;

    #[test]
    fn test_binary_unsigned() {
        let a = TypedBits {
            bits: vec![true, false, true, false],
            kind: Kind::make_bits(4),
        };
        let a_bs = BitString::Unsigned(a.bits.clone());
        let b = TypedBits {
            bits: vec![true, true, false, false],
            kind: Kind::make_bits(4),
        };
        let b_bs = BitString::Unsigned(b.bits.clone());
        // Loop over each operation in AluBinary
        for op in [
            AluBinary::Add,
            AluBinary::BitAnd,
            AluBinary::BitOr,
            AluBinary::BitXor,
            AluBinary::Eq,
            AluBinary::Ge,
            AluBinary::Gt,
            AluBinary::Le,
            AluBinary::Lt,
            AluBinary::Mul,
            AluBinary::Ne,
            AluBinary::Shl,
            AluBinary::Shr,
        ] {
            let res1 = binary(op, &a_bs, &b_bs).unwrap();
            let res2 = crate::rhif::runtime_ops::binary(op, a.clone(), b.clone()).unwrap();
            assert_eq!(res1.bits(), &res2.bits);
        }
    }

    #[test]
    fn test_binary_signed() {
        let a = TypedBits {
            bits: vec![true, false, true, true],
            kind: Kind::make_signed(4),
        };
        let a_bs = BitString::Signed(a.bits.clone());
        let b = TypedBits {
            bits: vec![false, true, false, false],
            kind: Kind::make_signed(4),
        };
        let b_bs = BitString::Signed(b.bits.clone());
        let c = TypedBits {
            bits: vec![false, true, false, false],
            kind: Kind::make_bits(4),
        };
        let c_bs = BitString::Unsigned(c.bits.clone());
        // Loop over each operation in AluBinary
        for op in [
            AluBinary::Add,
            AluBinary::BitAnd,
            AluBinary::BitOr,
            AluBinary::BitXor,
            AluBinary::Eq,
            AluBinary::Ge,
            AluBinary::Gt,
            AluBinary::Le,
            AluBinary::Lt,
            AluBinary::Mul,
            AluBinary::Ne,
        ] {
            let res1 = binary(op, &a_bs, &b_bs).unwrap();
            let res2 = crate::rhif::runtime_ops::binary(op, a.clone(), b.clone()).unwrap();
            assert_eq!(res1.bits(), &res2.bits);
        }
        for op in [AluBinary::Shl, AluBinary::Shr] {
            let res1 = binary(op, &a_bs, &c_bs).unwrap();
            let res2 = crate::rhif::runtime_ops::binary(op, a.clone(), c.clone()).unwrap();
            assert_eq!(res1.bits(), &res2.bits);
        }
    }
}
