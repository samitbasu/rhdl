use num_bigint::BigUint;
use num_bigint::{BigInt, Sign};
use std::iter::repeat;

use crate::types::bitx::BitX;
use crate::RHDLError;

pub fn to_bigint(bits: &[BitX]) -> Result<BigInt, RHDLError> {
    let bits = bits
        .iter()
        .map(|&x| x.try_into())
        .collect::<Result<Vec<_>, _>>()?;
    if bits.last() != Some(&true) {
        let bits = bits
            .iter()
            .map(|x| if *x { 1 } else { 0 })
            .collect::<Vec<_>>();
        Ok(BigInt::from_radix_le(Sign::Plus, &bits, 2).unwrap())
    } else {
        let bits = bits
            .iter()
            .map(|x| if *x { 0 } else { 1 })
            .collect::<Vec<_>>();
        Ok(-(BigInt::from_radix_le(Sign::Plus, &bits, 2).unwrap() + 1_i32))
    }
}

pub fn from_bigint(bi: &BigInt, len: usize) -> Vec<BitX> {
    if bi < &BigInt::ZERO {
        let bi = -bi - 1_i32;
        let bits = from_bigint(&bi, len);
        bits.iter().map(|&x| !x).collect::<Vec<_>>()
    } else {
        (0..len as u64).map(|pos| bi.bit(pos).into()).collect()
    }
}

pub fn to_biguint(bits: &[BitX]) -> Result<BigUint, RHDLError> {
    let bits = bits
        .iter()
        .map(|&x| x.try_into())
        .collect::<Result<Vec<_>, _>>()?;
    let bits = bits
        .iter()
        .map(|x| if *x { 1 } else { 0 })
        .collect::<Vec<_>>();
    Ok(BigUint::from_radix_le(&bits, 2).unwrap())
}

pub fn from_biguint(bi: &BigUint, len: usize) -> Vec<BitX> {
    (0..len as u64).map(|pos| bi.bit(pos).into()).collect()
}

pub(crate) fn add_one(a: &[BitX]) -> Vec<BitX> {
    a.iter()
        .scan(BitX::One, |carry, &b| {
            let sum = b ^ *carry;
            *carry &= b;
            Some(sum)
        })
        .collect()
}

pub(crate) fn full_add(a: &[BitX], b: &[BitX]) -> Vec<BitX> {
    a.iter()
        .zip(b.iter())
        .scan(BitX::Zero, |carry, (&a, &b)| {
            let sum = a ^ b ^ *carry;
            let new_carry = (a & b) | (a & *carry) | (b & *carry);
            *carry = new_carry;
            Some(sum)
        })
        .collect()
}

pub(crate) fn bit_not(a: &[BitX]) -> Vec<BitX> {
    a.iter().map(|&b| !b).collect()
}

pub(crate) fn bit_neg(a: &[BitX]) -> Vec<BitX> {
    add_one(&bit_not(a))
}

pub(crate) fn full_sub(a: &[BitX], b: &[BitX]) -> Vec<BitX> {
    full_add(a, &bit_neg(b))
}

pub(crate) fn bits_xor(a: &[BitX], b: &[BitX]) -> Vec<BitX> {
    a.iter().zip(b.iter()).map(|(&a, &b)| a ^ b).collect()
}

pub(crate) fn bits_and(a: &[BitX], b: &[BitX]) -> Vec<BitX> {
    a.iter().zip(b.iter()).map(|(&a, &b)| a & b).collect()
}

pub(crate) fn bits_or(a: &[BitX], b: &[BitX]) -> Vec<BitX> {
    a.iter().zip(b.iter()).map(|(&a, &b)| a | b).collect()
}

pub(crate) fn bits_shl(a: &[BitX], b: i64) -> Vec<BitX> {
    repeat(BitX::Zero)
        .take(b as usize)
        .chain(a.iter().copied())
        .take(a.len())
        .collect()
}

pub(crate) fn bits_shr(a: &[BitX], b: i64) -> Vec<BitX> {
    a.iter()
        .copied()
        .skip(b as usize)
        .chain(repeat(BitX::Zero).take(b as usize))
        .take(a.len())
        .collect()
}

pub(crate) fn bits_shr_signed(a: &[BitX], b: i64) -> Vec<BitX> {
    let sign = a.last().copied().unwrap_or_default();
    a.iter()
        .copied()
        .skip(b as usize)
        .chain(repeat(sign))
        .take(a.len())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bigint_conversion() -> Result<(), RHDLError> {
        let bits = [true, false, true, false]; // 5
        let bits = bits.iter().map(|&x| x.into()).collect::<Vec<_>>();
        let bi = to_bigint(&bits)?;
        assert_eq!(bi, BigInt::from(5));
        let bits_regen = from_bigint(&bi, 4);
        assert_eq!(bits_regen, bits);
        let bits = [true, true, false, true]; // -5
        let bits = bits.iter().map(|&x| x.into()).collect::<Vec<_>>();
        let bi = to_bigint(&bits)?;
        assert_eq!(bi, BigInt::from(-5));
        let bits_regen = from_bigint(&bi, 4);
        assert_eq!(bits_regen, bits);
        Ok(())
    }

    #[test]
    fn test_bigint_extend_behavior() -> Result<(), RHDLError> {
        let bits = [true, false, true, false]; // 5
        let bits = bits.iter().map(|&x| x.into()).collect::<Vec<_>>();
        let bi = to_bigint(&bits)?;
        let bits_regen = from_bigint(&bi, 8);
        assert_eq!(
            bits_regen,
            vec![
                BitX::One,
                BitX::Zero,
                BitX::One,
                BitX::Zero,
                BitX::Zero,
                BitX::Zero,
                BitX::Zero,
                BitX::Zero
            ]
        );
        Ok(())
    }
}
