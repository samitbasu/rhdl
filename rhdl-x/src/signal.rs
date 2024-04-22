use rhdl_bits::alias::*;
use rhdl_bits::Bits;
use rhdl_core::{ClockType, Sig, Timed};
use rhdl_macro::kernel;

/*

Thoughts:


*/
#[kernel]
fn add_sig<C: ClockType, D: ClockType>(
    x: Sig<b8, C>,
    y: Sig<b8, C>,
    z: Sig<b8, D>,
    w: Sig<b8, D>,
    e: b8,
) -> Sig<b8, D> {
    let c = x - y;
    let q = x + 3;
    let h = x > e;
    //let c = b8(4);
    //let cmp = x > c;
    if true {
        z
    } else {
        w
    }
}

#[kernel]
fn add<C: ClockType, D: ClockType>(
    x: Sig<b8, C>,
    y: Sig<b8, C>,
    z: Sig<b8, D>,
    w: Sig<b8, D>,
    ndx: b8,
) -> Sig<b8, D> {
    let c = x + y;
    let d = x > y;
    let bx = x.val();
    let zz = 2 < bx;
    let e = d && (!d ^ d);
    let q = z > w;
    let x = [c, c, c];
    let z2 = x[ndx];
    let res = if q { w } else { z };
    let h = z.val();
    let qq = h + y.val();
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

    #[test]
    fn test_timing() {
        let add = compile_design::<add_sig<Red, Blue>>().unwrap();
    }
}
