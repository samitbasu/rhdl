//! Pseudo-random number generator for testing purposes.
//!
//! This core provides a stream of random numbers and accepts
//! backpressure from the downstream.  It is useful for testing
//! stream-based designs, and is synthesizable.
//!
use crate::{core::slice::lsbs, stream::Ready};
use rhdl::prelude::*;

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
#[rhdl(dq_no_prefix)]
/// The Pseudo-Random Number Generator (PRNG) Core
pub struct PRNG<const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    rng: crate::rng::xorshift::XorShift,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to the [PRNG] core
pub struct In<const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    /// The ready signal from the downstream
    pub ready: Ready<Bits<N>>,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from the [PRNG] core
pub struct Out<const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    /// The random data output by the core.  
    pub data: Option<Bits<N>>,
}

impl<const N: usize> SynchronousIO for PRNG<N>
where
    rhdl::bits::W<N>: BitWidth,
{
    type I = In<N>;
    type O = Out<N>;
    type Kernel = prng_kernel<N>;
}

#[kernel]
#[doc(hidden)]
pub fn prng_kernel<const N: usize>(_cr: ClockReset, i: In<N>, q: Q<N>) -> (Out<N>, D<N>)
where
    rhdl::bits::W<N>: BitWidth,
{
    let mut d = D::<N>::dont_care();
    let mut o = Out::<N>::dont_care();
    let will_run = i.ready.raw;
    // Advance the RNG state if we are going to run.
    d.rng = will_run;
    // The AXI rules say that if we do not have valid data present, we must
    // (independently of the ready signal) decide if we are going to be valid
    // or not.
    o.data = Some(lsbs::<N, 32>(q.rng));
    (o, d)
}
