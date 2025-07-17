use rhdl::prelude::*;
use rhdl_fpga::core::{constant, dff};

#[derive(PartialEq, Debug, Digital)]
pub struct I {
    pub enable: bool,
}

#[derive(Clone, Debug, Synchronous)]
pub struct U<N: BitWidth> {
    counter: dff::DFF<Bits<N>>,
    threshold: constant::Constant<Bits<N>>,
}

impl<N: BitWidth> U<N> {
    pub fn new(threshold: Bits<N>) -> Self {
        Self {
            counter: dff::DFF::new(Bits::ZERO),
            threshold: constant::Constant::new(threshold),
        }
    }
}

#[derive(PartialEq, Debug, Digital)]
pub struct D<N: BitWidth> {
    counter: Bits<N>,
    threshold: (),
}

#[derive(PartialEq, Debug, Digital)]
pub struct Q<N: BitWidth> {
    counter: Bits<N>,
    threshold: Bits<N>,
}

impl<N: BitWidth> SynchronousIO for U<N> {
    type I = I;
    type O = bool;
    type Kernel = strobe<N>;
}

impl<N: BitWidth> SynchronousDQ for U<N> {
    type D = D<N>;
    type Q = Q<N>;
}

impl<N: BitWidth> Default for D<N> {
    fn default() -> Self {
        Self {
            counter: bits(0),
            threshold: (),
        }
    }
}

#[kernel]
pub fn strobe<N: BitWidth>(cr: ClockReset, i: I, q: Q<N>) -> (bool, D<N>) {
    let mut d = D::<N>::default();
    let count_next = if i.enable { q.counter + 1 } else { q.counter };
    let strobe = i.enable & (q.counter == q.threshold);
    let count_next = if strobe || cr.reset.any() {
        bits(0)
    } else {
        count_next
    };
    d.counter = count_next;
    (strobe, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strobe_timing() -> miette::Result<()> {
        let uut: U<U4> = U::new(bits(12));
        //let fg = uut.flow_graph("top")?;
        //eprintln!("{:?}", fg.timing_reports(trivial_cost).first().unwrap());
        //Ok(())
        todo!()
    }
}
