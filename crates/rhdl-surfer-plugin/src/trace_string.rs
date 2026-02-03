use num::BigUint;
use surfer_translation_types::ValueRepr;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TraceBit {
    Zero,
    One,
    X,
    Z,
    Illegal,
}

pub fn bit_char(x: TraceBit) -> char {
    match x {
        TraceBit::Zero => '0',
        TraceBit::One => '1',
        TraceBit::X => 'x',
        TraceBit::Z => 'z',
        TraceBit::Illegal => '?',
    }
}

impl From<TraceBit> for ValueRepr {
    fn from(bit: TraceBit) -> ValueRepr {
        ValueRepr::Bit(bit_char(bit))
    }
}

pub struct TraceString(pub Vec<TraceBit>);

impl std::fmt::Debug for TraceString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for bit in self.0.iter().rev() {
            match bit {
                TraceBit::Zero => write!(f, "0")?,
                TraceBit::One => write!(f, "1")?,
                TraceBit::X => write!(f, "x")?,
                TraceBit::Z => write!(f, "z")?,
                TraceBit::Illegal => write!(f, "?")?,
            }
        }
        Ok(())
    }
}

impl From<(&BigUint, usize)> for TraceString {
    fn from((v, width): (&BigUint, usize)) -> Self {
        let mut bits = Vec::with_capacity(width);
        for i in 0..width {
            let bit = if v.bit(i as u64) {
                TraceBit::One
            } else {
                TraceBit::Zero
            };
            bits.push(bit);
        }
        Self(bits)
    }
}

impl From<(&str, usize)> for TraceString {
    fn from((v, width): (&str, usize)) -> Self {
        let mut bits = Vec::with_capacity(width);
        for c in v.chars().rev().take(width) {
            let bit = match c {
                '0' => TraceBit::Zero,
                '1' => TraceBit::One,
                'x' | 'X' => TraceBit::X,
                'z' | 'Z' => TraceBit::Z,
                _ => TraceBit::Illegal,
            };
            bits.push(bit);
        }
        (0..(width.saturating_sub(bits.len()))).for_each(|_| bits.push(TraceBit::Illegal));
        Self(bits)
    }
}
