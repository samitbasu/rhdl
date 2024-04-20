use rhdl_bits::alias::*;
use rhdl_core::{circuit::signal::Signal, types::clock::Red};

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

#[test]
fn test_signal_if() {
    let x: Signal<_, Red> = Signal::new(b4(0b1010));
    let y: Signal<_, Red> = Signal::new(b4(0b0101));
    let z = x > y;
}
