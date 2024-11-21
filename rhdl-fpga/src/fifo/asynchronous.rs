use crate::cdc::split_counter;
use crate::core::ram;
use rhdl::prelude::*;

use super::read_logic;
use super::write_logic;

/// A simple asynchronous FIFO.  This FIFO is designed to be as simple as possible
/// and thus be robust.  It is a two-port FIFO, with separate read and write
/// ports.  The FIFO is parameterized by the number of bits in each element.
/// The depth of the FIFO is 2^N-1 elements.  You cannot fill the FIFO to 2^N elements.
/// The FIFO is asynchronous, meaning that the read and write ports are not
/// synchronized to each other.  This means that the read and write ports
/// can be in different clock domains.
#[derive(Clone, Circuit, CircuitDQ)]
pub struct U<T: Digital, W: Domain, R: Domain, const N: usize> {
    write_logic: Adapter<write_logic::U<N>, W>,
    read_logic: Adapter<read_logic::U<N>, R>,
    ram: ram::U<T, W, R, N>,
    read_count_for_write_logic: split_counter::U<R, W, N>,
    write_count_for_read_logic: split_counter::U<W, R, N>,
}

#[derive(Clone, Copy, Debug, PartialEq, Digital, Timed)]
pub struct I<T: Digital, W: Domain, R: Domain> {
    /// The data to be written to the FIFO in the W domain
    data: Signal<Option<T>, W>,
    /// The next signal for the read logic in the R domain
    next: Signal<bool, R>,
    /// The clock and reset for the W domain
    cr_w: Signal<ClockReset, W>,
    /// The clock and reset for the R domain
    cr_r: Signal<ClockReset, R>,
}

#[derive(Clone, Copy, Debug, PartialEq, Digital, Timed)]
pub struct O<T: Digital, W: Domain, R: Domain> {
    /// The data read from the FIFO in the R domain
    data: Signal<Option<T>, R>,
    /// The almost empty flag in the R domain
    almost_empty: Signal<bool, R>,
    /// The underflow flag in the R domain
    underflow: Signal<bool, R>,
    /// The full flag in the W domain
    full: Signal<bool, W>,
    /// The almost full flag in the W domain
    almost_full: Signal<bool, W>,
    /// The overflow flag in the W domain
    overflow: Signal<bool, W>,
}

impl<T: Digital, W: Domain, R: Domain, const N: usize> CircuitIO for U<T, W, R, N> {
    type I = I<T, W, R>;
    type O = O<T, W, R>;
    type Kernel = async_fifo_kernel<T, W, R, N>;
}

//#[kernel]
pub fn async_fifo_kernel<T: Digital, W: Domain, R: Domain, const N: usize>(
    i: I<T, W, R>,
    q: Q<T, W, R, N>,
) -> (O<T, W, R>, D<T, W, R, N>) {
    let mut d = D::<T, W, R, N>::init();
    // Clock the various components
    d.write_logic.clock_reset = i.cr_w;
    d.read_logic.clock_reset = i.cr_r;
    d.ram.write.clock = i.cr_w;

    //todo!()
}
