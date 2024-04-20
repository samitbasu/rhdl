use rhdl_bits::alias::*;
use rhdl_bits::Bits;
use rhdl_core::{circuit::signal::Signal, ClockType};
use rhdl_macro::kernel;

#[kernel]
fn add<C: ClockType, D: ClockType>(
    x: Signal<b8, C>,
    y: Signal<b8, C>,
    z: Signal<b8, D>,
    w: Signal<b8, D>,
    ndx: b8,
) -> Signal<b8, D> {
    let c = x + y;
    let d = x > y;
    let e = d && (!d ^ d);
    let q = z > w;
    let x = [c, c, c];
    let z2 = x[ndx];
    let res = if q { w } else { z };
    let h = z.val();
    match h + 1 {
        Bits::<8>(0) => z,
        _ => w,
    }
}

#[cfg(test)]
mod tests {

    use rhdl_core::{
        compile_design,
        types::clock::{Blue, Red},
    };

    use super::*;

    #[test]
    fn test_signal_if() {
        let add = compile_design::<add<Red, Blue>>().unwrap();
    }
}
