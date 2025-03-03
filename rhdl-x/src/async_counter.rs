use rhdl::{core::ClockReset, prelude::*};

#[derive(Clone, Circuit, CircuitDQ)]
pub struct U {
    counter: Adapter<crate::counter::U<4>, Red>,
}

#[kernel]
pub fn async_counter(i: I, q: Q) -> (O, D) {
    let mut d = D::dont_care();
    d.counter.clock_reset = i.clock_reset;
    d.counter.input = i.enable;
    let mut o = O::dont_care();
    o.count = q.counter;
    (o, d)
}

#[derive(PartialEq, Debug, Digital, Timed)]
pub struct I {
    pub clock_reset: Signal<ClockReset, Red>,
    pub enable: Signal<crate::counter::I, Red>,
}

#[derive(PartialEq, Debug, Digital, Timed)]
pub struct O {
    pub count: Signal<Bits<U4>, Red>,
}

impl CircuitIO for U {
    type I = I;
    type O = O;
    type Kernel = async_counter;
}

impl Default for U {
    fn default() -> Self {
        Self {
            counter: Adapter::new(crate::counter::U::default()),
        }
    }
}
