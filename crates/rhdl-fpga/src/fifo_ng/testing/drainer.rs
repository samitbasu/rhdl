use crate::{
    core::{dff, slice::lsbs},
    stream::{Ready, ready},
};
use rhdl::prelude::*;

#[derive(Debug, Synchronous, SynchronousDQ)]
#[rhdl(dq_no_prefix)]
/// The drainer core for PRNG sequences
pub struct Drainer<const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    rng: crate::rng::xorshift::XorShift,
    _phantom: core::marker::PhantomData<Bits<N>>,
    valid: dff::DFF<bool>,
}

impl<const N: usize> Default for Drainer<N>
where
    rhdl::bits::W<N>: BitWidth,
{
    fn default() -> Self {
        Self {
            rng: crate::rng::xorshift::XorShift::default(),
            _phantom: core::marker::PhantomData,
            valid: dff::DFF::new(true),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
pub struct In<const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    pub data: Option<Bits<N>>,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
pub struct Out<const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    pub ready: Ready<Bits<N>>,
    pub valid: bool,
}

impl<const N: usize> SynchronousIO for Drainer<N>
where
    rhdl::bits::W<N>: BitWidth,
{
    type I = In<N>;
    type O = Out<N>;
    type Kernel = drainer_kernel<N>;
}

#[kernel]
#[doc(hidden)]
pub fn drainer_kernel<const N: usize>(_cr: ClockReset, i: In<N>, q: Q<N>) -> (Out<N>, D<N>)
where
    rhdl::bits::W<N>: BitWidth,
{
    let mut d = D::<N>::dont_care();
    d.rng = false;
    d.valid = q.valid;
    if let Some(data) = i.data {
        d.rng = true;
        d.valid = q.valid && data == lsbs::<N, 32>(q.rng);
    }
    let o = Out::<N> {
        ready: ready::<Bits<N>>(true),
        valid: q.valid,
    };
    (o, d)
}
