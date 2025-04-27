//! This is a pipeline RAM interface.  It provides the ability to
//! read and write from the RAM using the `Option<T>` interface
//! for a non-stallable pipeline.  What this means is that you can
//! think of reading from the RAM as injecting a command to read
//! from a given address A, and then some time later, out comes
//! the corresponding data element D.
//!
//! The schematic symbol for this unit looks like this:
#![doc = badascii_doc::badascii!("
       +-+BRAM+----+    
 ?(A,D)|           |    
+----->|write      |    
   ?A  |           | ?D 
+----->|read   data+--->
       +-----------+    
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
//! # Example
//!
//! Here is an example of the the `PipeSyncBRAM` being
//! used with the same test sequence illustrated above.
//!
//!```
//!# use rhdl::prelude::*;
//!# use rhdl_fpga::core::ram::pipe_sync::{In, PipeBRAM};
//!#
//!# fn main() -> Result<(), RHDLError> {
//!     // Generate the stream example from the timing diagram.
//!     // Read location 2, then 3 then 2 again
//!     let reads = [None, Some(b3(2)), Some(b3(3)), Some(b3(2)), None];
//!     // Write to location 2 while reading from 3
//!     let writes = [None, None, Some((b3(2), b8(42))), None, None];
//!     let inputs = reads
//!         .into_iter()
//!         .zip(writes)
//!         .map(|(r, w)| In { read: r, write: w })
//!         .stream_after_reset(1)
//!         .clock_pos_edge(100);
//!     let uut = PipeBRAM::new((0..).map(|x| (b3(x), b8(x))));
//!     let vcd = uut.run(inputs)?.collect::<Vcd>();
//!#    rhdl_fpga::doc::write_svg_as_markdown(vcd,"pipe_ram.md",
//!#              SvgOptions::default()
//!#                 .with_label_width(20)
//!#                 .with_filter("(^top.clock.*)|(^top.input.*)|(^top.output.*)"),
//!#    ).unwrap();
//!#     Ok(())
//!# }
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
pub struct PipeBRAM<T: Digital + Default, N: BitWidth> {
    ram: super::option_sync::OptionSyncBRAM<T, N>,
    delay: dff::U<bool>,
}

impl<T: Digital + Default, N: BitWidth> PipeBRAM<T, N> {
    pub fn new(initial: impl IntoIterator<Item = (Bits<N>, T)>) -> Self {
        Self {
            ram: OptionSyncBRAM::new(initial),
            delay: dff::U::default(),
        }
    }
}

#[derive(PartialEq, Debug, Digital)]
pub struct In<T: Digital + Default, N: BitWidth> {
    pub read: Option<Bits<N>>,
    pub write: Option<(Bits<N>, T)>,
}

impl<T: Digital + Default, N: BitWidth> SynchronousIO for PipeBRAM<T, N> {
    type I = In<T, N>;
    type O = Option<T>;
    type Kernel = kernel<T, N>;
}

#[kernel]
pub fn kernel<T: Digital + Default, N: BitWidth>(
    _cr: ClockReset,
    i: In<T, N>,
    q: Q<T, N>,
) -> (Option<T>, D<T, N>) {
    let mut d = D::<T, N>::dont_care();
    let (tag, addr) = unpack::<Bits<N>>(i.read);
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
            .stream_after_reset(1)
            .clock_pos_edge(100);
        let uut = PipeBRAM::new((0..).map(|x| (b3(x), b8(x))));
        let vcd = uut
            .run(inputs)?
            .sample_at_pos_edge(|x| x.value.0.clock)
            .filter_map(|x| x.value.2)
            .collect::<Vec<b8>>();
        assert_eq!(vcd, vec![bits(2), bits(3), bits(42)]);
        Ok(())
    }
}
