use crate::core::dff;
use crate::core::synchronous_ram;
use rhdl::prelude::*;

use super::read_logic;
use super::write_logic;

/// A simple synchronous FIFO.  This FIFO is designed to be as simple as possible
/// while still being robust.  It is a two-port FIFO, with separate read and write
/// ports.  The FIFO is parameterized by the number of bits in each element.
/// The depth of the FIFO is 2^N-1 elements.  You cannot fill the FIFO to 2^N elements.
#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U<T: Digital, const N: usize> {
    write_logic: write_logic::U<N>,
    read_logic: read_logic::U<N>,
    ram: synchronous_ram::U<T, N>,
}

#[derive(Clone, Copy, Debug, PartialEq, Digital)]
pub struct I<T: Digital> {
    data: T,
    write_enable: bool,
    next: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Digital)]
pub struct O<T: Digital> {
    data: T,
    empty: bool,
    full: bool,
    almost_empty: bool,
    almost_full: bool,
    overflow: bool,
    underflow: bool,
}

impl<T: Digital, const N: usize> SynchronousIO for U<T, N> {
    type I = I<T>;
    type O = O<T>;
    type Kernel = fifo_kernel<T, N>;
}

#[kernel]
pub fn fifo_kernel<T: Digital, const N: usize>(
    _cr: ClockReset,
    i: I<T>,
    q: Q<T, N>,
) -> (O<T>, D<T, N>) {
    // This is essentially a wiring exercise.  The clock
    // and reset are propagated to the sub-elements automatically
    // so we just need to route the signals.
    let mut d = D::<T, N>::init();
    let mut o = O::<T>::init();
    // Connect the read logic inputs
    d.read_logic.write_address = q.write_logic.write_address;
    d.read_logic.next = i.next;
    // Connect the write logic inputs
    d.write_logic.read_address = q.read_logic.read_address;
    d.write_logic.write_enable = i.write_enable;
    // Connect the RAM inputs
    d.ram.write.addr = q.write_logic.write_address;
    d.ram.write.value = i.data;
    d.ram.write.enable = i.write_enable;
    d.ram.read_addr = q.read_logic.read_address;
    // Populate the outputs
    o.data = q.ram;
    o.empty = q.read_logic.empty;
    o.full = q.write_logic.full;
    o.almost_empty = q.read_logic.almost_empty;
    o.almost_full = q.write_logic.almost_full;
    o.overflow = q.write_logic.overflow;
    o.underflow = q.read_logic.underflow;
    (o, d)
}

#[cfg(test)]
mod tests {
    use stream::reset_pulse;

    use super::*;

    fn write(data: b8) -> I<Bits<8>> {
        I {
            data,
            write_enable: true,
            next: false,
        }
    }

    fn read() -> I<Bits<8>> {
        I {
            data: Bits::default(),
            write_enable: false,
            next: true,
        }
    }

    #[test]
    fn basic_write_then_read_test() {
        type UC = U<Bits<8>, 3>;
        let uut = U::<Bits<8>, 3>::default();
        let write_seq = (0..7).map(|i| write(bits(i + 1)));
        let read_seq = (0..7).map(|_| read());
        let stream = stream(write_seq.chain(read_seq));
        let stream = reset_pulse(1).chain(stream);
        let stream = clock_pos_edge(stream, 100);
        validate_synchronous(
            &uut,
            stream,
            //            &mut [glitch_check_synchronous::<UC>()],
            &mut [],
            ValidateOptions::default().vcd("fifo.vcd"),
        );
    }
}
