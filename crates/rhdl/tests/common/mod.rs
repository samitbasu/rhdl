#![allow(dead_code)]
use rhdl::prelude::*;

pub fn exhaustive<const N: usize>() -> Vec<Bits<N>>
where
    rhdl::bits::W<N>: BitWidth,
{
    (0..(1 << N)).map(bits).collect()
}

pub fn exhaustive_signed<const N: usize>() -> Vec<SignedBits<N>>
where
    rhdl::bits::W<N>: BitWidth,
{
    (SignedBits::<N>::min_value()..=SignedBits::<N>::max_value())
        .map(signed)
        .collect()
}

pub fn tuple_exhaustive_red<const N: usize>()
-> impl Iterator<Item = (Signal<Bits<N>, Red>,)> + Clone
where
    rhdl::bits::W<N>: BitWidth,
{
    exhaustive::<N>().into_iter().map(|x| (signal(x),))
}

pub fn tuple_b8<C: Domain>() -> impl Iterator<Item = (Signal<b8, C>,)> + Clone {
    (0..=255).map(b8).map(|x| (signal(x),))
}

pub fn tuple_pair_bn_red<const N: usize>()
-> impl Iterator<Item = (Signal<Bits<N>, Red>, Signal<Bits<N>, Red>)> + Clone
where
    rhdl::bits::W<N>: BitWidth,
{
    exhaustive::<N>()
        .into_iter()
        .flat_map(|x| exhaustive::<N>().into_iter().map(move |y| (red(x), red(y))))
}

pub fn tuple_pair_b8_red() -> impl Iterator<Item = (Signal<b8, Red>, Signal<b8, Red>)> + Clone {
    exhaustive::<8>()
        .into_iter()
        .flat_map(|x| exhaustive::<8>().into_iter().map(move |y| (red(x), red(y))))
}

pub fn tuple_pair_b4_red() -> impl Iterator<Item = (Signal<b4, Red>, Signal<b4, Red>)> + Clone {
    exhaustive::<4>()
        .into_iter()
        .flat_map(|x| exhaustive::<4>().into_iter().map(move |y| (red(x), red(y))))
}

pub fn tuple_pair_sn_red<const N: usize>()
-> impl Iterator<Item = (Signal<SignedBits<N>, Red>, Signal<SignedBits<N>, Red>)> + Clone
where
    rhdl::bits::W<N>: BitWidth,
{
    exhaustive_signed::<N>().into_iter().flat_map(|x| {
        exhaustive_signed::<N>()
            .into_iter()
            .map(move |y| (red(x), red(y)))
    })
}

pub fn tuple_pair_s8_red() -> impl Iterator<Item = (Signal<s8, Red>, Signal<s8, Red>)> + Clone {
    exhaustive::<8>().into_iter().flat_map(|x| {
        exhaustive::<8>()
            .into_iter()
            .map(move |y| (red(x.as_signed()), red(y.as_signed())))
    })
}

pub fn s8_red() -> impl Iterator<Item = (Signal<s8, Red>,)> + Clone {
    exhaustive::<8>().into_iter().map(|x| (red(x.as_signed()),))
}

pub fn red<T: Digital>(x: T) -> Signal<T, Red> {
    signal(x)
}

pub fn miette_report(err: RHDLError) -> String {
    let handler =
        miette::GraphicalReportHandler::new_themed(miette::GraphicalTheme::unicode_nocolor());
    let mut msg = String::new();
    handler.render_report(&mut msg, &err).unwrap();
    msg
}
