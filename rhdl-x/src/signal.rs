use rhdl_bits::alias::*;
use rhdl_core::{
    circuit::signal::Signal,
    types::clock::{Blue, Red},
    ClockType, Digital,
};

// Create a macro that acts like an if expression, except that it calls
// `unsafe (expr.val())` on the expression target of the if.  It should
// also support else clauses, etc.
macro_rules! signal_if {
    ($cond:expr, $expr:expr) => {
        if {unsafe $cond.val()} {
            $expr
        }
    };
    ($cond:expr, $expr:expr, $else:expr) => {
        if {unsafe $cond.val()} {
            $expr.val()
        } else {
            $else.val()
        }
    };
}

struct Unify<T1: Digital, T2: Digital, C: ClockType>(Signal<T1, C>, Signal<T2, C>);

struct U2<T: Digital, C: ClockType>(Signal<T, C>, C);

#[test]
fn test_signal_if() {
    let x: Signal<_, Red> = Signal::new(b4(0b1010));
    let y: Signal<_, Red> = Signal::new(b4(0b0101));
    let q: Signal<_, Blue> = Signal::new(b4(0b0000));
    let w: Signal<_, Blue> = Signal::new(b4(0b1111));
    let z = x > y;
    let a = !z;
    let b = a & z;
    let b_c = Signal::new(b);
    let c = if b { q } else { w };
    let d = match z_c.val() {
        true => q,
        false => w,
    };
    // We want to unify z with x and with y...  so we could do something like.
    Unify(x, y);
    Unify(x, z_c);
    Unify(z_c, q);
    Unify(z_c, w);
}

// If we have a match .val() or if then we
