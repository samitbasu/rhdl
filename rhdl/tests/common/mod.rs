#![allow(dead_code)]
use rhdl::prelude::*;

pub fn exhaustive<const N: usize>() -> Vec<Bits<N>> {
    (0..(1 << N)).map(bits).collect()
}

pub fn tuple_exhaustive_red<const N: usize>(
) -> impl Iterator<Item = (Signal<Bits<N>, Red>,)> + Clone {
    exhaustive::<N>().into_iter().map(|x| (signal(x),))
}

pub fn tuple_u8<C: Domain>() -> impl Iterator<Item = (Signal<u8, C>,)> + Clone {
    (0_u8..255_u8).map(|x| (signal(x),))
}

pub fn tuple_pair_b8_red() -> impl Iterator<Item = (Signal<b8, Red>, Signal<b8, Red>)> + Clone {
    exhaustive::<8>()
        .into_iter()
        .flat_map(|x| exhaustive::<8>().into_iter().map(move |y| (red(x), red(y))))
}

pub fn tuple_pair_s8_red() -> impl Iterator<Item = (Signal<s8, Red>, Signal<s8, Red>)> + Clone {
    exhaustive::<8>().into_iter().flat_map(|x| {
        exhaustive::<8>()
            .into_iter()
            .map(move |y| (red(x.as_signed()), red(y.as_signed())))
    })
}

pub fn red<T: Digital>(x: T) -> Signal<T, Red> {
    signal(x)
}
