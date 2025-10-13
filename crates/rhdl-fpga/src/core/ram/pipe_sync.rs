//! A pipeline interface to a BRAM
//!
//! This is a pipeline RAM interface.  It provides the ability to
//! read and write from the RAM using the `Option<T>` interface
//! for a non-stallable pipeline.  What this means is that you can
//! think of reading from the RAM as injecting a command to read
//! from a given address A, and then some time later, out comes
//! the corresponding data element D.
//!
//! The schematic symbol for this unit looks like this:
#![doc = badascii_doc::badascii_formal!("
       +-+PipeSyncBRAM+--+    
 ?(A,D)|                 |    
+----->|write            |    
   ?A  |                 | ?D 
+----->|read         data+--->
       +-----------------+    
")]
//! where `?` is short for `Option`.  
//!
//!  The delay between the injection
//! of A and the receipt of D is fixed (one clock cycle for the BRAM).
//! It could also be variable (as in accessing a DRAM, for example).
//! But in this case it is fixed and constant.  This circuit tracks the
//! single cycle latency on the input and uses it to blank out the
//! output.  Like this:
#![doc = badascii_doc::badascii!("
+-+Timing+----------------------------------------------------+
|                                                             |
|        +----+    +----+    +----+    +----+    +            |
| clk  +-+    +----+    +----+    +----+    +----+            |
|        :         :         :         :         :            |
|  in    None +-+Some+--+--+Some+-+--+Some+-+  ...            |
|      +------+--+A1+---+---+A2+--+---+A3+--+-----+           |
|        :         :         :         :         :            |
| tag    :    +---------+---------+---------+  ...            |
|      +------+    :    +    :    +    :    +-----+           |
|        :         :         :         :         :            |
| addr   XXXX +---+A1+--+---+A2+--+---+A3+--+  ...            |
|      +------+---------+---------+---------+-----+           |
|        :         :         :         :         :            |
| data           XXXX   +---+D1+--+---+D2+--+---+D3+--+  ...  |
|      +----------------+---------+---------+---------+-----+ |
|                  :         :         :         :         :  |
| tag_out          :    +---------+---------+---------+  ...  |
|                +------+    :    +    :    +    :    +-----+ |
|                  :         :         :         :         :  |
| out              None +-+Some+--+--+Some+-+--+Some+-+  ...  |
|                +------+--+A1+---+---+A2+--+---+A3+--+-----+ |
|                                                             |
+-------------------------------------------------------------+
")]
//! From this diagram, it is clear what the internal guts of the
//! pipeline RAM look like.  The tag (valid bit) is stripped
//! off of the input signal, delayed by one clock cycle to account
//! for the latency of the BRAM read, and then merged with the
//! output value.
//!
#![doc = badascii_doc::badascii!("
+--+Pipe BRAM+-------------------------------+
|                                            |
|     Unwrap                        Wrap     |
|     +--+                          +--+     |
|     |  |tag  D +-----+ Q       tag|  |     |
| ?A  |  +------>| DFF +----------->|  | ?T  |
+---->|  |A      |     |            |  +---->|
|     |  +--+    +-----+        +-->|  |     |
|     +--+  |                   |   +--+     |
|           |   +---+?BRAM+---+ |T           |
|           |   |             | |            |
|           +-->|read_addr out+-+            |
|   ?(B<N>,T)   |             |              |
+-------------->|write        |              |
|               |             |              |
|               +-------------+              |
|                                            |
+--------------------------------------------+
")]
//!
//!
//!# Example
//!
//! Here is an example of the the `PipeSyncBRAM` being
//! used with the same test sequence illustrated above.
//!
//!```
#![doc = include_str!("../../../examples/pipe_sync.rs")]
//!```
//! The resulting trace file is:
#![doc = include_str!("../../../doc/pipe_ram.md")]

use rhdl::prelude::*;

use crate::core::{
    dff,
    option::{pack, unpack},
};

use super::option_sync::OptionSyncBRAM;

#[derive(PartialEq, Debug, Clone, Default, Synchronous, SynchronousDQ)]
/// The unit that implements the [PipeBRAM]
/// The `T` parameter indicates the type of data held in the BRAM.
/// The `N` parameter indicates the number of address bits.
pub struct PipeSyncBRAM<T: Digital, const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    ram: super::option_sync::OptionSyncBRAM<T, N>,
    delay: dff::DFF<bool>,
}

impl<T: Digital, const N: usize> PipeSyncBRAM<T, N>
where
    rhdl::bits::W<N>: BitWidth,
{
    /// Construct a new [PipeSyncBRAM] with the provided initial contents.
    pub fn new(initial: impl IntoIterator<Item = (Bits<N>, T)>) -> Self {
        Self {
            ram: OptionSyncBRAM::new(initial),
            delay: dff::DFF::default(),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// The inputs for the [PipeBRAM]
pub struct In<T: Digital, const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    /// The read commands.  For the output to be
    /// valid, you provide [Some] address.  And then
    /// one cycle later, the output will also be [Some].
    pub read: Option<Bits<N>>,
    /// The write commands.  See [OptionSyncBRAM] for how
    /// to use this.
    pub write: Option<(Bits<N>, T)>,
}

impl<T: Digital, const N: usize> SynchronousIO for PipeSyncBRAM<T, N>
where
    rhdl::bits::W<N>: BitWidth,
{
    type I = In<T, N>;
    type O = Option<T>;
    type Kernel = kernel<T, N>;
}

#[kernel]
/// The kernel for the [PipeBRAM]
pub fn kernel<T: Digital, const N: usize>(
    _cr: ClockReset,
    i: In<T, N>,
    q: Q<T, N>,
) -> (Option<T>, D<T, N>)
where
    rhdl::bits::W<N>: BitWidth,
{
    let mut d = D::<T, N>::dont_care();
    let (tag, addr) = unpack::<Bits<N>>(i.read, bits(0));
    d.ram.write = i.write;
    d.ram.read_addr = addr;
    d.delay = tag;
    let out = pack::<T>(q.delay, q.ram);
    (out, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operation() -> miette::Result<(), RHDLError> {
        // Generate the stream example from the timing diagram.
        // Read location 2, then 3 then 2 again
        let reads = [None, Some(b3(2)), Some(b3(3)), Some(b3(2)), None];
        // Write to location 2 while reading from 3
        let writes = [None, None, Some((b3(2), b8(42))), None, None];
        let inputs = reads
            .into_iter()
            .zip(writes)
            .map(|(r, w)| In { read: r, write: w })
            .with_reset(1)
            .clock_pos_edge(100);
        let uut = PipeSyncBRAM::new((0..).map(|x| (b3(x), b8(x))));
        let vcd = uut
            .run(inputs)
            .sample_at_pos_edge(|x| x.value.0.clock)
            .filter_map(|x| x.value.2)
            .collect::<Vec<b8>>();
        assert_eq!(vcd, vec![bits(2), bits(3), bits(42)]);
        Ok(())
    }
}
