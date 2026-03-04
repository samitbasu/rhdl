use crate::core::{dff, option::is_some};
use rhdl::prelude::*;

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
#[rhdl(dq_no_prefix)]
/// The FIFO read logic as a core
pub struct FIFOReadCore<T, const N: usize>
where
    T: Digital,
    rhdl::bits::W<N>: BitWidth,
{
    ram_read_address: dff::DFF<Bits<N>>,
    read_address_delayed: dff::DFF<Bits<N>>,
    output_buffer: dff::DFF<Option<T>>,
}

impl<T, const N: usize> Default for FIFOReadCore<T, N>
where
    T: Digital,
    rhdl::bits::W<N>: BitWidth,
{
    fn default() -> Self {
        Self {
            ram_read_address: dff::DFF::new(bits(0)),
            read_address_delayed: dff::DFF::new(bits(0)),
            output_buffer: dff::DFF::new(None),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// The inputs to the [FIFOReadCore]
pub struct In<T, const N: usize>
where
    T: Digital,
    rhdl::bits::W<N>: BitWidth,
{
    /// The current write address
    pub write_address: Bits<N>,
    /// The data back from the RAM
    pub data: T,
    /// The ready signal for downstream logic.
    pub ready: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// The outputs from the [FIFOReadCore]
pub struct Out<T, const N: usize>
where
    T: Digital,
    rhdl::bits::W<N>: BitWidth,
{
    /// The read address to the RAM
    pub ram_read_address: Bits<N>,
    /// The read address delayed to send to the write side
    pub read_address_delayed: Bits<N>,
    /// The data output from the internal buffer.
    pub data_out: Option<T>,
}

impl<T, const N: usize> SynchronousIO for FIFOReadCore<T, N>
where
    T: Digital,
    rhdl::bits::W<N>: BitWidth,
{
    type I = In<T, N>;
    type O = Out<T, N>;
    type Kernel = read_logic<T, N>;
}

#[kernel]
pub fn read_logic<T, const N: usize>(
    _cr: ClockReset,
    i: In<T, N>,
    q: Q<T, N>,
) -> (Out<T, N>, D<T, N>)
where
    T: Digital,
    rhdl::bits::W<N>: BitWidth,
{
    let mut d = D::<T, N>::dont_care();
    // The behavior of the read logic depends on 3 booleans.
    //  - empty is true if the read and write pointers are
    //    equal, indicating the FIFO is empty.
    //  - downstream_ready is true if the downstream logic is ready to accept data.
    //  - output_is_some is true if the output buffer contains valid data.
    let empty = i.write_address == q.ram_read_address;
    let downstream_ready = i.ready;
    let output_is_some = is_some::<T>(q.output_buffer);
    let will_consume = downstream_ready && output_is_some;
    // Latch prevention
    d.output_buffer = q.output_buffer;
    let will_advance = !empty && (downstream_ready || !output_is_some);
    let next_address = if will_advance {
        q.ram_read_address + 1
    } else {
        q.ram_read_address
    };
    if will_advance {
        d.output_buffer = Some(i.data);
    } else if will_consume {
        d.output_buffer = None;
    }
    d.ram_read_address = next_address;
    d.read_address_delayed = next_address;
    let o = Out::<T, N> {
        ram_read_address: next_address,
        read_address_delayed: q.read_address_delayed,
        data_out: q.output_buffer,
    };
    (o, d)
}
