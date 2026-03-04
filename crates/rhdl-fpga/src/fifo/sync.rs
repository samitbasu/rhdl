use crate::core::ram;
use rhdl::prelude::*;

use super::read_core;
use super::write_core;

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
#[rhdl(dq_no_prefix)]
pub struct SyncFIFO<T: Digital, const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    write_logic: write_core::FIFOWriteCore<T, N>,
    read_logic: read_core::FIFOReadCore<T, N>,
    ram: ram::option_sync::OptionSyncBRAM<T, N>,
}

impl<T: Digital, const N: usize> Default for SyncFIFO<T, N>
where
    rhdl::bits::W<N>: BitWidth,
{
    fn default() -> Self {
        Self {
            write_logic: write_core::FIFOWriteCore::default(),
            read_logic: read_core::FIFOReadCore::default(),
            ram: ram::option_sync::OptionSyncBRAM::default(),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
pub struct In<T: Digital> {
    pub data: Option<T>,
    pub ready: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
pub struct Out<T: Digital> {
    pub data: Option<T>,
    pub ready: bool,
}

impl<T: Digital, const N: usize> SynchronousIO for SyncFIFO<T, N>
where
    rhdl::bits::W<N>: BitWidth,
{
    type I = In<T>;
    type O = Out<T>;
    type Kernel = fifo_kernel<T, N>;
}

#[kernel]
pub fn fifo_kernel<T: Digital, const N: usize>(
    _cr: ClockReset,
    i: In<T>,
    q: Q<T, N>,
) -> (Out<T>, D<T, N>)
where
    rhdl::bits::W<N>: BitWidth,
{
    let mut d = D::<T, N>::dont_care();
    d.write_logic.data = i.data;
    d.write_logic.read_address = q.read_logic.read_address_delayed;
    d.read_logic.write_address = q.write_logic.write_address;
    d.read_logic.ready = i.ready;
    d.read_logic.data = q.ram;
    d.ram.write = q.write_logic.data;
    d.ram.read_addr = q.read_logic.ram_read_address;
    let o = Out::<T> {
        data: q.read_logic.data_out,
        ready: q.write_logic.ready,
    };
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hdl_generation_fifo() -> miette::Result<()> {
        let uut = SyncFIFO::<Bits<8>, 3>::default();
        let desc = uut.descriptor(ScopedName::top())?;
        let hdl = desc.hdl()?;
        let verilog = hdl.modules.pretty();
        expect_test::expect_file!["testing/sync_fifo.v"].assert_eq(&verilog);
        Ok(())
    }

    #[test]
    fn test_no_combinatorial_paths() -> miette::Result<()> {
        let uut = SyncFIFO::<Bits<8>, 3>::default();
        drc::no_combinatorial_paths(&uut)?;
        Ok(())
    }
}
