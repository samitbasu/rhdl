use std::iter::repeat;

pub(crate) fn add_one(a: &[bool]) -> Vec<bool> {
    a.iter()
        .scan(true, |carry, b| {
            let sum = b ^ *carry;
            *carry = *carry & b;
            Some(sum)
        })
        .collect()
}

pub(crate) fn full_add(a: &[bool], b: &[bool]) -> Vec<bool> {
    a.iter()
        .zip(b.iter())
        .scan(false, |carry, (a, b)| {
            let sum = a ^ b ^ *carry;
            let new_carry = (a & b) | (a & *carry) | (b & *carry);
            *carry = new_carry;
            Some(sum)
        })
        .collect()
}

pub(crate) fn bit_not(a: &[bool]) -> Vec<bool> {
    a.iter().map(|b| !b).collect()
}

pub(crate) fn bit_neg(a: &[bool]) -> Vec<bool> {
    add_one(&bit_not(a))
}

pub(crate) fn full_sub(a: &[bool], b: &[bool]) -> Vec<bool> {
    full_add(a, &bit_neg(b))
}

pub(crate) fn bits_xor(a: &[bool], b: &[bool]) -> Vec<bool> {
    a.iter().zip(b.iter()).map(|(a, b)| a ^ b).collect()
}

pub(crate) fn bits_and(a: &[bool], b: &[bool]) -> Vec<bool> {
    a.iter().zip(b.iter()).map(|(a, b)| a & b).collect()
}

pub(crate) fn bits_or(a: &[bool], b: &[bool]) -> Vec<bool> {
    a.iter().zip(b.iter()).map(|(a, b)| a | b).collect()
}

pub(crate) fn bits_shl(a: &[bool], b: i64) -> Vec<bool> {
    repeat(false)
        .take(b as usize)
        .chain(a.iter().copied())
        .take(a.len())
        .collect()
}

pub(crate) fn bits_shr(a: &[bool], b: i64) -> Vec<bool> {
    a.iter()
        .copied()
        .skip(b as usize)
        .chain(repeat(false).take(b as usize))
        .take(a.len())
        .collect()
}

pub(crate) fn bits_eq(a: &[bool], b: &[bool]) -> bool {
    a.iter().zip(b.iter()).all(|(a, b)| a == b)
}

pub(crate) fn bits_lt(a: &[bool], b: &[bool]) -> bool {
    a.iter().rev().zip(b.iter().rev()).any(|(a, b)| !a && *b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bits_lt() {
        assert!(bits_lt(&[false, false, false], &[false, false, true]));
        assert!(!bits_lt(&[false, false, true], &[false, true, false]));
    }
}
