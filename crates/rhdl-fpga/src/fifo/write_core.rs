use crate::core::dff;
use rhdl::prelude::*;

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
#[rhdl(dq_no_prefix)]
/// The FIFO write logic as a core
pub struct FIFOWriteCore<T, const N: usize>
where
    T: Digital,
    rhdl::bits::W<N>: BitWidth,
{
    write_address: dff::DFF<Bits<N>>,
    // We delay the write address by one clock before sending
    // it to the read side of the FIFO.  This is because it will
    // take one clock for the write to actually happen, and we
    // want to make sure the value is valid on the read side before
    // "counting" the write.
    write_address_delayed: dff::DFF<Bits<N>>,
    // Marker for the type T
    phantom: std::marker::PhantomData<T>,
}

impl<T, const N: usize> Default for FIFOWriteCore<T, N>
where
    T: Digital,
    rhdl::bits::W<N>: BitWidth,
{
    fn default() -> Self {
        Self {
            write_address: dff::DFF::new(bits(0)),
            write_address_delayed: dff::DFF::new(bits(0)),
            phantom: std::marker::PhantomData,
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// The inputs to the [FIFOWriteCore]
pub struct In<T: Digital, const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    /// The input data
    pub data: Option<T>,
    /// The current read address
    pub read_address: Bits<N>,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// The outputs from the [FIFOWriteCore]
pub struct Out<T, const N: usize>
where
    T: Digital,
    rhdl::bits::W<N>: BitWidth,
{
    /// The generated ready signal
    pub ready: bool,
    /// The write address to send to the BRAM
    pub ram_write_address: Bits<N>,
    /// The write address to send to the read side (delayed)
    pub write_address: Bits<N>,
    /// The data to write to the BRAM
    pub data: Option<(Bits<N>, T)>,
}

impl<T, const N: usize> SynchronousIO for FIFOWriteCore<T, N>
where
    T: Digital,
    rhdl::bits::W<N>: BitWidth,
{
    type I = In<T, N>;
    type O = Out<T, N>;
    type Kernel = write_logic<T, N>;
}

#[kernel]
/// Kernel for the [FIFOWriteCore]
pub fn write_logic<T, const N: usize>(
    _cr: ClockReset,
    i: In<T, N>,
    q: Q<T, N>,
) -> (Out<T, N>, D<T, N>)
where
    T: Digital,
    rhdl::bits::W<N>: BitWidth,
{
    let mut o = Out::<T, N>::dont_care();
    // Compute the full flag
    let full = (q.write_address + 1) == i.read_address;
    let mut will_write = false;
    o.data = if let Some(data) = i.data {
        if !full {
            will_write = true;
            Some((q.write_address, data))
        } else {
            None
        }
    } else {
        None
    };
    o.ready = !full;
    o.ram_write_address = q.write_address;
    o.write_address = q.write_address_delayed;
    let mut d = D::<T, N>::dont_care();
    d.write_address = if will_write {
        q.write_address + 1
    } else {
        q.write_address
    };
    d.write_address_delayed = q.write_address;
    (o, d)
}
