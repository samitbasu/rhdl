use crate::{
    core::{constant, dff, slice::msbs},
    rng::xorshift::XorShift,
    stream::{StreamIO, ready},
};
use rhdl::prelude::*;

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
#[rhdl(dq_no_prefix)]
pub struct Staller<T: Digital> {
    store: dff::DFF<Option<T>>,
    rng: XorShift,
    stall_probability: constant::Constant<Bits<16>>,
    sleep_len: constant::Constant<Bits<4>>,
    sleep_counter: dff::DFF<Bits<4>>,
}

impl<T: Digital> Staller<T> {
    pub fn new(stall_probability: f32, sleep_len: u8) -> Self {
        let stall_probability = 65535.0 * stall_probability.clamp(0.0, 1.0);
        Self {
            store: dff::DFF::new(None),
            rng: XorShift::default(),
            stall_probability: constant::Constant::new(b16(stall_probability as u128)),
            sleep_len: constant::Constant::new(b4(sleep_len as u128)),
            sleep_counter: dff::DFF::new(b4(0)),
        }
    }
}

impl<T: Digital> SynchronousIO for Staller<T> {
    type I = StreamIO<T, T>;
    type O = StreamIO<T, T>;
    type Kernel = staller<T>;
}

#[kernel]
#[doc(hidden)]
pub fn staller<T: Digital>(_cr: ClockReset, i: StreamIO<T, T>, q: Q<T>) -> (StreamIO<T, T>, D<T>) {
    let mut o = StreamIO::<T, T>::dont_care();
    let mut d = D::<T>::dont_care();
    let is_sleeping = q.sleep_counter != 0;
    trace("is_sleeping", &is_sleeping);
    let will_advance = !is_sleeping && i.ready.raw;
    trace("will_advance", &will_advance);
    d.store = q.store;
    if will_advance {
        d.store = i.data;
    }
    d.rng = false;
    if will_advance && q.sleep_counter == 0 {
        d.rng = true;
        let p = msbs::<16, 32>(q.rng);
        trace("p", &p);
        d.sleep_counter = if p < q.stall_probability {
            q.sleep_len
        } else {
            b4(0)
        };
    } else {
        d.sleep_counter = if q.sleep_counter != 0 {
            q.sleep_counter - 1
        } else {
            b4(0)
        }
    };
    o.ready = ready::<T>(will_advance);
    o.data = if q.sleep_counter != 0 { None } else { q.store };
    (o, d)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_filler() -> miette::Result<()> {
        Ok(())
    }
}
