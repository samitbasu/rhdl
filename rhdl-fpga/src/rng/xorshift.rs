use rhdl::prelude::*;

use crate::core::dff;

// A port of the LFSR from Alchitry.com's Lucid module `pn_gen`.
// The default seed is the same as the one in the Lucid module.
// Any non-zero seed will work.  It takes a boolean  signal
// as the "next" signal to advance the LFSR. - This in turn is
// actually a firmware implementation of XORSHIFT128.
#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
pub struct U {
    x: dff::U<Bits<W32>>,
    y: dff::U<Bits<W32>>,
    z: dff::U<Bits<W32>>,
    w: dff::U<Bits<W32>>,
}

const SEED: u128 = 0x843233523a613966423b622562592c62;

impl Default for U {
    fn default() -> Self {
        Self {
            x: dff::U::new(bits(SEED & 0xFFFF_FFFF)),
            y: dff::U::new(bits((SEED >> 32) & 0xFFFF_FFFF)),
            z: dff::U::new(bits((SEED >> 64) & 0xFFFF_FFFF)),
            w: dff::U::new(bits((SEED >> 96) & 0xFFFF_FFFF)),
        }
    }
}

impl SynchronousIO for U {
    type I = bool;
    type O = Bits<W32>;
    type Kernel = lfsr_kernel;
}

#[kernel]
pub fn lfsr_kernel(_cr: ClockReset, strobe: bool, q: Q) -> (Bits<W32>, D) {
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

/// For testing, it's handy to have a way to generate the same sequence as the hardware.
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
        let uut = U::default();
        let input = std::iter::repeat(true)
            .stream_after_reset(1)
            .clock_pos_edge(100);
        let values = uut
            .run(input)?
            .synchronous_sample()
            .skip(1) // Skip the first value with is zero
            .map(|x| x.value.2);
        let validate = XorShift128::default();
        for (value, expected) in values.zip(validate.take(100)) {
            assert_eq!((value.0 & 0xFFFF_FFFF) as u32, expected);
        }
        Ok(())
    }
}
