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
    if arg1.is_signed() ^ arg2.is_signed() {
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
    use super::*;

    #[test]
    fn test_binary() {
        let a = BitString::Unsigned(vec![true, false, true, false]); // 5
        let b = BitString::Unsigned(vec![true, true, false, false]); // 3
        assert_eq!(
            binary(AluBinary::Add, &a, &b).unwrap(),
            BitString::Unsigned(vec![true, false, false, true, false])
        );
        assert_eq!(
            binary(AluBinary::Sub, &a, &b).unwrap(),
            BitString::Unsigned(vec![false, true, true, false, false])
        );
        assert_eq!(
            binary(AluBinary::BitXor, &a, &b).unwrap(),
            BitString::Unsigned(vec![false, true, true, true, false])
        );
        assert_eq!(
            binary(AluBinary::BitAnd, &a, &b).unwrap(),
            BitString::Unsigned(vec![true, false, false, false, false])
        );
        assert_eq!(
            binary(AluBinary::BitOr, &a, &b).unwrap(),
            BitString::Unsigned(vec![true, true, true, true, false])
        );
        assert_eq!(
            binary(AluBinary::Eq, &a, &b).unwrap(),
            BitString::Unsigned(vec![false])
        );
        assert_eq!(
            binary(AluBinary::Ne, &a, &b).unwrap(),
            BitString::Unsigned(vec![true])
        );
        assert_eq!(
            binary(AluBinary::Shl, &a, &BitString::Unsigned(vec![true, false])).unwrap(),
            BitString::Unsigned(vec![true, false, true, false, false])
        );
        assert_eq!(
            binary(AluBinary::Shr, &a, &BitString::Unsigned(vec![true, false])).unwrap(),
            BitString::Unsigned(vec![false, true, false, true])
        );
        assert_eq!(
            binary(AluBinary::Lt, &a, &b).unwrap(),
            BitString::Unsigned(vec![true])
        );
        assert_eq!(
            binary(AluBinary::Le, &a, &b).unwrap(),
            BitString::Unsigned(vec![true])
        );
    }
}
