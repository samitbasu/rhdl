#![allow(dead_code)]
use rhdl::prelude::*;

pub fn exhaustive<N: BitWidth>() -> Vec<Bits<N>> {
    (0..(1 << N::BITS)).map(bits).collect()
}

pub fn exhaustive_signed<N: BitWidth>() -> Vec<SignedBits<N>> {
    (SignedBits::<N>::min_value()..=SignedBits::<N>::max_value())
        .map(signed)
        .collect()
}

pub fn tuple_exhaustive_red<N: BitWidth>() -> impl Iterator<Item = (Signal<Bits<N>, Red>,)> + Clone
{
    exhaustive::<N>().into_iter().map(|x| (signal(x),))
}

pub fn tuple_b8<C: Domain>() -> impl Iterator<Item = (Signal<b8, C>,)> + Clone {
    (0..=255).map(b8).map(|x| (signal(x),))
}

pub fn tuple_pair_b8_red() -> impl Iterator<Item = (Signal<b8, Red>, Signal<b8, Red>)> + Clone {
    exhaustive::<U8>().into_iter().flat_map(|x| {
        exhaustive::<U8>()
            .into_iter()
            .map(move |y| (red(x), red(y)))
    })
}

pub fn tuple_pair_b4_red() -> impl Iterator<Item = (Signal<b4, Red>, Signal<b4, Red>)> + Clone {
    exhaustive::<U4>().into_iter().flat_map(|x| {
        exhaustive::<U4>()
            .into_iter()
            .map(move |y| (red(x), red(y)))
    })
}

pub fn tuple_pair_s8_red() -> impl Iterator<Item = (Signal<s8, Red>, Signal<s8, Red>)> + Clone {
    exhaustive::<U8>().into_iter().flat_map(|x| {
        exhaustive::<U8>()
            .into_iter()
            .map(move |y| (red(x.as_signed()), red(y.as_signed())))
    })
}

pub fn s8_red() -> impl Iterator<Item = (Signal<s8, Red>,)> + Clone {
    exhaustive::<U8>()
        .into_iter()
        .map(|x| (red(x.as_signed()),))
}

pub fn red<T: Digital>(x: T) -> Signal<T, Red> {
    signal(x)
}
