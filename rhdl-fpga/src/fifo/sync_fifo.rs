use crate::core::synchronous_ram;
use rhdl::prelude::*;

use super::read_logic;
use super::write_logic;

/// A simple synchronous FIFO.  This FIFO is designed to be as simple as possible
/// and thus be robust.  It is a two-port FIFO, with separate read and write
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
    data: Option<T>,
    next: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Digital)]
pub struct O<T: Digital> {
    data: Option<T>,
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
    let (write_data, write_enable) = match i.data {
        Some(data) => (data, true),
        None => (T::init(), false),
    };
    // Connect the read logic inputs
    d.read_logic.write_address = q.write_logic.write_address;
    d.read_logic.next = i.next;
    // Connect the write logic inputs
    d.write_logic.read_address = q.read_logic.read_address;
    d.write_logic.write_enable = write_enable;
    // Connect the RAM inputs
    d.ram.write.addr = q.write_logic.write_address;
    d.ram.write.value = write_data;
    d.ram.write.enable = write_enable;
    d.ram.read_addr = q.read_logic.read_address;
    // Populate the outputs
    o.data = if q.read_logic.empty {
        None
    } else {
        Some(q.ram)
    };
    o.full = q.write_logic.full;
    o.almost_empty = q.read_logic.almost_empty;
    o.almost_full = q.write_logic.almost_full;
    o.overflow = q.write_logic.overflow;
    o.underflow = q.read_logic.underflow;
    (o, d)
}

/*
#[cfg(test)]
mod tests {
    use stream::reset_pulse;

    use super::*;

    fn write(data: b8) -> I<Bits<8>> {
        I {
            data: Some(data),
            next: false,
        }
    }

    fn read() -> I<Bits<8>> {
        I {
            data: None,
            next: true,
        }
    }

    fn test_seq() -> impl Iterator<Item = TimedSample<(ClockReset, I<Bits<8>>)>> {
        let write_seq = (0..7).map(|i| write(bits(i + 1)));
        let read_seq = (0..7).map(|_| read());
        let stream = stream(write_seq.chain(read_seq));
        let stream = reset_pulse(1).chain(stream);
        clock_pos_edge(stream, 100)
    }

    #[test]
    fn basic_write_then_read_test() {
        type UC = U<Bits<8>, 3>;
        let uut = U::<Bits<8>, 3>::default();
        let stream = test_seq();
        validate_synchronous(
            &uut,
            stream,
            //            &mut [glitch_check_synchronous::<UC>()],
            &mut [],
            ValidateOptions::default().vcd("fifo.vcd"),
        );
    }

    #[test]
    fn test_hdl_generation_rtl() -> miette::Result<()> {
        type UC = U<Bits<8>, 3>;
        let uut = U::<Bits<8>, 3>::default();
        let options = TestModuleOptions {
            skip_first_cases: !0,
            vcd_file: Some("fifo_sync_rtl.vcd".into()),
            hold_time: 1,
            ..Default::default()
        };
        let stream = test_seq();
        let test_mod = build_rtl_testmodule_synchronous(&uut, stream, options)?;
        std::fs::write("fifo_sync_rtl.v", test_mod.to_string()).unwrap();
        test_mod.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_hdl_generation_fg() -> miette::Result<()> {
        type UC = U<Bits<8>, 3>;
        let uut = UC::default();
        let options = TestModuleOptions {
            vcd_file: Some("fifo_sync_fg.vcd".into()),
            flow_graph_level: true,
            ..Default::default()
        };
        let stream = test_seq();
        let test_mod = build_rtl_testmodule_synchronous(&uut, stream, options)?;
        std::fs::write("fifo_sync_fg.v", test_mod.to_string()).unwrap();
        test_mod.run_iverilog()?;
        Ok(())
    }
}
*/
