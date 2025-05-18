//! A Pseudorandom Number Generator based on XORSHIFT128
//!
//! A port of the LFSR from Alchitry.com's Lucid module `pn_gen`.
//! The default seed is the same as the one in the Lucid module.
//! Any non-zero seed will work.  It takes a boolean  signal
//! as the "next" signal to advance the LFSR. - This in turn is
//! actually a gateware implementation of XORSHIFT128.
//!
//!# Schematic symbol
//!
#![doc = badascii_formal!("
      ++XorShift+-----+     
bool  |               | b32 
+---->|input    output+---->
      |               |     
      +---------------+     
")]
//!
//! The usage is simple.  If `input` is `true`, then `output`
//! will advance to the next pseudo-random number in the sequence.
//! Otherwise, the output is latched by internal flip flops.
//!
//!# Example
//!
//! This core is simple to use.
//!
//!```
#![doc = include_str!("../../examples/xor_png.rs")]
//!```
//!
//! Here is the trace.
//!
#![doc = include_str!("../../doc/xor_png.md")]

use badascii_doc::badascii_formal;
use rhdl::prelude::*;

use crate::core::dff;

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
/// The [XorShift] core.  Note that resetting the
/// core resets the sequence.
pub struct XorShift {
    x: dff::DFF<Bits<U32>>,
    y: dff::DFF<Bits<U32>>,
    z: dff::DFF<Bits<U32>>,
    w: dff::DFF<Bits<U32>>,
}

const SEED: u128 = 0x843233523a613966423b622562592c62;

impl Default for XorShift {
    fn default() -> Self {
        Self {
            x: dff::DFF::new(bits(SEED & 0xFFFF_FFFF)),
            y: dff::DFF::new(bits((SEED >> 32) & 0xFFFF_FFFF)),
            z: dff::DFF::new(bits((SEED >> 64) & 0xFFFF_FFFF)),
            w: dff::DFF::new(bits((SEED >> 96) & 0xFFFF_FFFF)),
        }
    }
}

impl XorShift {
    /// Use the provided seed instead of the default.
    pub fn new(seed: u128) -> Self {
        Self {
            x: dff::DFF::new(bits(seed & 0xFFFF_FFFF)),
            y: dff::DFF::new(bits((seed >> 32) & 0xFFFF_FFFF)),
            z: dff::DFF::new(bits((seed >> 64) & 0xFFFF_FFFF)),
            w: dff::DFF::new(bits((seed >> 96) & 0xFFFF_FFFF)),
        }
    }
}

impl SynchronousIO for XorShift {
    type I = bool;
    type O = Bits<U32>;
    type Kernel = lfsr_kernel;
}

#[kernel]
#[doc(hidden)]
pub fn lfsr_kernel(_cr: ClockReset, strobe: bool, q: Q) -> (Bits<U32>, D) {
    let mut d = D::dont_care();
    d.x = q.x;
    d.y = q.y;
    d.z = q.z;
    d.w = q.w;
    let o = q.x ^ (q.x << 11);
    if strobe {
        d.x = q.y;
        d.y = q.z;
        d.z = q.w;
        d.w = q.w ^ (q.w >> 19) ^ o ^ (o >> 8);
    }
    (o, d)
}

/// For testing, it's handy to have a way to generate
/// the same sequence as the hardware.  This
/// struct `impl Iterator` and yields the same
/// sequence as the hardware with the default seed.
#[derive(Clone)]
pub struct XorShift128 {
    state: [u32; 4],
}

impl Default for XorShift128 {
    fn default() -> Self {
        Self {
            state: [
                ((SEED >> 96) & 0xFFFF_FFFF) as u32,
                ((SEED >> 64) & 0xFFFF_FFFF) as u32,
                ((SEED >> 32) & 0xFFFF_FFFF) as u32,
                (SEED & 0xFFFF_FFFF) as u32,
            ],
        }
    }
}

fn xorshift_128(state: &mut [u32; 4]) -> u32 {
    let mut t = state[3];
    let s = state[0];
    state[3] = state[2];
    state[2] = state[1];
    state[1] = s;
    t ^= t << 11;
    let q = t;
    t ^= t >> 8;
    state[0] = t ^ s ^ (s >> 19);
    q
}

impl Iterator for XorShift128 {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        Some(xorshift_128(&mut self.state))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xorshift_seq() {
        let rng = XorShift128::default();
        for value in rng.take(10) {
            eprintln!("{:x}", value);
        }
    }

    #[test]
    fn test_uut() -> miette::Result<()> {
        let uut = XorShift::default();
        let input = std::iter::repeat(true)
            .with_reset(1)
            .clock_pos_edge(100);
        let values = uut
            .run(input)?
            .synchronous_sample()
            .skip(1) // Skip the first value with is zero
            .map(|x| x.value.2);
        let validate = XorShift128::default();
        for (value, expected) in values.zip(validate.take(100)) {
            assert_eq!((value.raw() & 0xFFFF_FFFF) as u32, expected);
        }
        Ok(())
    }
}
