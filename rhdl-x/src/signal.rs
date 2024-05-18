use rhdl_bits::alias::*;
use rhdl_bits::Bits;
use rhdl_core::ClockColor;
use rhdl_core::Digital;
use rhdl_core::Kind;
use rhdl_core::{Clock, Sig, Timed};
use rhdl_macro::kernel;

/*

Thoughts:

*/

pub struct Sig<T: Digital, const C: char> {}

trait Clocked<T: Digital>: Copy + Sized + PartialEq + Eq {
    fn color() -> ClockColor;
    fn static_kind() -> Kind {
        Kind::make_signal(T::static_kind(), Self::color())
    }
    fn kind(&self) -> Kind {
        Self::static_kind()
    }
    fn retime(x: T) -> Self;
    fn val(self) -> T;
}

pub struct Async<T: Digital> {
    val: T,
}

pub struct Clk0<T: Digital> {
    val: T,
}

pub struct Clk1<T: Digital> {
    val: T,
}

impl<T: Digital> Clocked<T> for Clk0<T> {
    fn color() -> ClockColor {
        ClockColor::Red
    }
    fn val(self) -> T {
        self.val
    }
    fn retime(x: T) -> Self {
        Clk0 { val: x }
    }
}

fn retime<C: Clocked<T>, T: Digital>(x: T) -> C {
    C::retime(x)
}

fn run_it<C0: Clocked<b8>, C1: Clocked<b8>>(x: C0, y: C1) -> C0 {
    let a = x.val();
    let y = y.val();
    let b = a + y;
    retime(b)
}

// Can only tuples be used to express signals with multiple clocks?
struct Pair<C0: Clocked<b8>, C1: Clocked<b8>> {
    x: C0,
    y: C1,
}
// Need another trait for this to work since we need to introspect the
// pair to get the color and types of the clocks
// This is already in the core - its called Timed.

impl<C0, C1> Timed for Pair<C0, C1>
where
    C0: Clocked<b8>,
    C1: Clocked<b8>,
{
    fn static_kind() -> Kind {
        Kind::make_signal(b8::static_kind(), Self::color())
    }
    fn kind(&self) -> Kind {
        Self::static_kind()
    }
}

fn run_it_pair1<C0: Clocked<b8>, C1: Clocked<b8>>(x: Pair<C0, C1>) -> Pair<C0, C1> {
    let a = x.x.val();
    let y = x.y.val();
    let b = a + y;
    Pair {
        x: retime(b),
        y: retime(b),
    }
}

fn run_it_pair<C0: Clocked<b8>, C1: Clocked<b8>>(x: C0, y: C1) -> (C0, C1) {
    let a = x.val();
    let y = y.val();
    let b = a + y;
    (retime(b), retime(b))
}

//impl<T: Digital>

#[kernel]
fn add_sig<C: Clock, D: Clock>(
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
    let cmp = x > c;
    if cmp {
        z
    } else {
        w
    }
}

#[kernel]
fn add<C: Clock, D: Clock>(
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

#[kernel]
fn add1<C: Clock, D: Clock>(
    x: Sig<b8, C>,
    y: Sig<b8, C>,
    z: Sig<b8, D>,
    w: Sig<b8, D>,
    ndx: b8,
) -> b8 {
    let c = x + y;
    ndx
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
    fn test_signal_if1() {
        let add = compile_design::<add1<Red, Blue>>().unwrap();
    }

    #[test]
    fn test_timing() {
        let add = compile_design::<add_sig<Red, Blue>>().unwrap();
    }
}
