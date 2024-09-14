use rhdl::prelude::*;

#[derive(Clone, Circuit)]
#[rhdl(kernel=async_counter)]
#[rhdl(auto_dq)]
pub struct U {
    counter: Adapter<crate::counter::U<4>, Red>,
}

#[kernel]
pub fn async_counter(i: I, q: Q) -> (O, D) {
    let mut d = D::uninit();
    d.counter.clock = i.clock;
    d.counter.reset = i.reset;
    d.counter.input = i.enable;
    let mut o = O::uninit();
    o.count = q.counter;
    (o, d)
}

#[derive(Clone, Copy, PartialEq, Debug, Digital, Timed)]
pub struct I {
    pub enable: Signal<crate::counter::I, Red>,
    pub clock: Signal<Clock, Red>,
    pub reset: Signal<Reset, Red>,
}

#[derive(Clone, Copy, PartialEq, Debug, Digital, Timed)]
pub struct O {
    pub count: Signal<Bits<4>, Red>,
}

impl CircuitIO for U {
    type I = I;
    type O = O;
}

impl Default for U {
    fn default() -> Self {
        Self {
            counter: Adapter::new(crate::counter::U::new()),
        }
    }
}
