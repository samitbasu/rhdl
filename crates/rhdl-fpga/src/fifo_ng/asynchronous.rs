//! A simple asynchronous FIFO.
//!
//! This FIFO is designed to be as simple as possible
//! and thus be robust.  It is a two-port FIFO, with separate read and write
//! ports.  The FIFO is parameterized by the number of bits in each element.
//! The depth of the FIFO is 2^N elements.
//! The FIFO is asynchronous, meaning that the read and write ports are not
//! synchronized to each other.  This means that the read and write ports
//! can be in different clock domains.
//!
//! Here is the schematic symbol for the FIFO
#![doc = badascii_doc::badascii_formal!("
      +-----+AsyncFIFO+------------------------+     
  ?T  |                   +                    | ?T  
+---->| data       W      |     R         data +---->
      |         domain   <+>  domain           |     
<-----+ ready             |              ready |<---+
      |                   +                    |     
+---->| cr_w                              cr_r |<---+
      |                                        |     
      +----------------------------------------+     
")]
//!
//!#Internals
//!
#![doc = badascii_doc::badascii!("
                   ++FIFOWriteCore++                                                     bN ++FIFOReadCore+---+                       
  ?T           ?T  |               |  bN                                 +----------------->| wr_addr         | ?T          ?T        
+----> data   +--->|data     rd_adr|<-------------------------------+    |               b1 |            data +---->     +-----> data 
                b1 |               |  bool                          |    |    +-------------+ did_read        | b1                    
<----+ ready  <----+ready    did_wr+-------+   To FIFOReadCore      |    |    |  To      bN |           ready |<----+    <-----+ ready
 cr_w              |               |?(bN,T)+   To                   |    |    |  BRAM  <----+ ram_rdaddr      |                       
+------------+---->|cr         data+---------> BRAM                 |    |    |          T  |              cr |<-----+------+--+ cr_r 
             |     +---------------+       +                        |    |    |  From  +--->| data            |      |                
             |                             |                        |    |    |  BRAM       +-----------------+      |                
             |                             |                        |    |    |                                      |                
             |                             |            wr_count_4  +    |    |                                      |                
             |                             |           +-------------+   |    |                                      |                
             |                             +---------->| incr  count +---+    |                                      |                
             |                                   W     |             |    R   |                                      |                
             |                                 domain  |             | domain +                                      |                
             +-----------------+---------------------->| incr_cr  cr |<-------------+--------------------------------+                
                               |                       +-------------+        +     |                                                 
                               |                                    +         |     |                                                 
                               |                                    +         |     |                                                 
                               |              +-------------------------------+     |                                                 
                               |              |     rd_count_4_wr   +               |                                                 
                               |              |    +-------------+  |               |                                                 
                               |              +--->| incr  count +--+               |                                                 
                               |             W     |             |     R            |                                                 
                               |           domain  |             |  domain          |                                                 
                               |              +--->| incr_cr  cr |<---+             |                                                 
                               |              |    +-------------+    |             |                                                 
                               |              |                       +             |                                                 
                               |              +-------------------------------------+                                                 
                               |                                      +                                                               
                               +--------------------------------------+                                                               
")]
//!# Example
use crate::cdc::cross_counter;
use crate::core::ram;
use crate::stream::Ready;
use rhdl::prelude::*;

use super::read_logic;
use super::write_logic;

/// A simple asynchronous FIFO.  
///  `T` is the data type held by the FIFO.  Must satisfy
/// `T : Default`.
///  `W` the clock domain for the write side of the FIFO.
///  `R` the clock domain for the read side of the FIFO
///  `N` the number bits in the address.  FIFO holds `2^N` elements
///  when full.
#[derive(Clone, Circuit, CircuitDQ, Default)]
pub struct AsyncFIFO<T: Digital, W: Domain, R: Domain, const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    write_logic: Adapter<write_logic::FIFOWriteCore<T, N>, W>,
    read_logic: Adapter<read_logic::FIFOReadCore<T, N>, R>,
    ram: ram::option_async::OptionAsyncBram<T, W, R, N>,
    read_count_for_write_logic: cross_counter::CrossCounter<R, W, N>,
    write_count_for_read_logic: cross_counter::CrossCounter<W, R, N>,
}

#[derive(PartialEq, Debug, Digital, Copy, Timed, Clone)]
/// Inputs for the FIFO
pub struct In<T: Digital, W: Domain, R: Domain> {
    /// The data to be written to the FIFO in the W domain
    pub data: Signal<Option<T>, W>,
    /// The ready signal for the read logic in the R domain
    pub ready: Signal<Ready<T>, R>,
    /// The clock and reset for the W domain
    pub cr_w: Signal<ClockReset, W>,
    /// The clock and reset for the R domain
    pub cr_r: Signal<ClockReset, R>,
}

#[derive(PartialEq, Debug, Digital, Copy, Timed, Clone)]
/// Outputs from the FIFO
pub struct Out<T: Digital, W: Domain, R: Domain> {
    /// The data read from the FIFO in the R domain
    pub data: Signal<Option<T>, R>,
    /// The ready signal for the write logic in the W domain
    pub ready: Signal<Ready<T>, W>,
}

impl<T: Digital, W: Domain, R: Domain, const N: usize> CircuitIO for AsyncFIFO<T, W, R, N>
where
    rhdl::bits::W<N>: BitWidth,
{
    type I = In<T, W, R>;
    type O = Out<T, W, R>;
    type Kernel = async_fifo_kernel<T, W, R, N>;
}

#[kernel]
/// Async FIFO kernel
pub fn async_fifo_kernel<T: Digital, W: Domain, R: Domain, const N: usize>(
    i: In<T, W, R>,
    q: AsyncFIFOQ<T, W, R, N>,
) -> (Out<T, W, R>, AsyncFIFOD<T, W, R, N>)
where
    rhdl::bits::W<N>: BitWidth,
{
    let mut d = AsyncFIFOD::<T, W, R, N>::dont_care();
    // Clock the write core
    d.write_logic.clock_reset = i.cr_w;
    // Clock the read core
    d.read_logic.clock_reset = i.cr_r;
    // Create the inputs for the write logic
    let mut wc_in = write_logic::In::<T, N>::dont_care();
    // The read address for the write core comes from the read_count_for_write_logic
    wc_in.read_address = q.read_count_for_write_logic.count.val();
    wc_in.data = i.data.val();
    d.write_logic.input = signal(wc_in);
    // Create the inputs for the read logic
    let mut rc_in = read_logic::In::<T, N>::dont_care();
    rc_in.write_address = q.write_count_for_read_logic.count.val();
    rc_in.data = q.ram.val();
    rc_in.ready = i.ready.val();
    d.read_logic.input = signal(rc_in);
    // Create the inputs for the read_count_for_write_logic
    d.read_count_for_write_logic.incr_cr = i.cr_r;
    d.read_count_for_write_logic.cr = i.cr_w;
    d.read_count_for_write_logic.incr = signal(q.read_logic.val().did_read);
    // Create the inputs for the write_count_for_read_logic
    d.write_count_for_read_logic.incr_cr = i.cr_w;
    d.write_count_for_read_logic.cr = i.cr_r;
    d.write_count_for_read_logic.incr = signal(q.write_logic.val().did_write);
    // Create the inputs for the read side of the RAM
    let mut ram_read = ram::asynchronous::ReadI::<N>::dont_care();
    ram_read.clock = i.cr_r.val().clock;
    ram_read.addr = q.read_logic.val().ram_read_address;
    d.ram.read = signal(ram_read);
    let mut ram_write = ram::option_async::WriteI::<T, N>::dont_care();
    ram_write.clock = i.cr_w.val().clock;
    ram_write.data = q.write_logic.val().data;
    d.ram.write = signal(ram_write);
    let mut o = Out::<T, W, R>::dont_care();
    o.ready = signal(q.write_logic.val().ready);
    o.data = signal(q.read_logic.val().data_out);
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_combinatorial_paths() -> miette::Result<()> {
        let uut = AsyncFIFO::<Bits<8>, Red, Blue, 5>::default();
        let descriptor = uut.descriptor(ScopedName::top())?;
        let schematic = descriptor.schematic()?;
        let ports = schematic.all_ports();
        // Check that no ports are duplicated in the list
        let unique_ports: std::collections::HashSet<_> = ports.iter().cloned().collect();
        assert_eq!(
            ports.len(),
            unique_ports.len(),
            "Duplicate ports found in schematic"
        );
        schematic.check_for_combinatorial_io_paths()?;
        Ok(())
    }
}
