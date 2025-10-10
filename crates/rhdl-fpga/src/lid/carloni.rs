//! A Carloni Relay station (aka Skid Buffer)
//!
//! Implement a Carloni Relay station.  As described in the
//! paper "From Latency-Insensitive Design to Communication-Based
//! System-Level Design" by Carloni. Proceedings of the IEEE, 2015.
//!
//! This is an implementation of the relay station as shown in Figure 4.
//! A relay station is essentially a set of flip flops that can
//! be inserted into a Ready-Valid bus to break long paths.  They
//! add latency, but do not affect overall throughput.
//!
//!# Schematic symbol
//!
//! Here is a symbol for the buffer, as depicted in the original paper.
//!
#![doc = badascii_formal!("
     +-----+Carloni+-------+    
+--->| data_in    data_out +--->
     |                     |    
 <---+ stop_out   stop_in  |<--+
     |                     +    
+--->| void_in    void_out +--->
     +---------------------+    
")]
//! The `void` signal indicates the validity of the data line,
//! and `stop` is an active-high flow control mechanism.
//!
//!# Internal details
//!
//! The details of the buffer are well explained in the paper.  Here is
//! rough sketch of the internals of the buffer component.
//!
#![doc = badascii!(r"
                 +      ++main++                          
data_in          |\     | FF   |                          
 ++------------->|0+    |      |                          
  |   ++Aux++    | +--->|d    q+---> data_out             
  +-->|d   q+--->|1+    |  en  |                          
      | FF  |    |/     +------+                          
      | en  |    +^         ^     +-+Control+-------+     
      +-----+     |sel      +-----+ main_en         |     
         ^        |         |     |         stop_in +---->
         |        +---------+-----+ sel             |     
         |        |         |     |                 |     
         +--------+---------+-----+ aux_en          |     
                  |         |     |                 |     
   <--------------+---------+-----+ stop_out        |     
                  |sel      v     +-----------------+     
                 +v     +------+                          
                 |\     |  en  |                          
      void_in +->|0+    |      |                          
                 | +--->|d    q+---> void_out             
           0 +-->|1+    | FF   |                          
                 |/     ++void++                          
                 +                                        
")]
//!
//! Note that the flip flops in this design have enable
//! signals.  The logic of the control block is shown below.
#![doc = badascii!("
 !stop_in +                                           
 (!void_in & void_out)        stop_in & void_in       
+---------------------+       +---------------+       
 sel = 0            +-------+     sel = 0             
 main_en = 1     +->|       |<-+  main_en = 0         
 aux_end = 0     |  |       |  |  aux_en = 0          
 stop_out = 0    |  |  Run  |  |  stop_out = 0        
                 +--+       +--+                      
                    +-----+-+                         
       !stop_in       ^   |                           
     +------------+   |   |  stop_in & !void_in       
       sel = 1        |   |  & !void_out              
       main_en = 1    |   |  +----------------+       
       aux_en = 0     |   |      sel = 0              
       stop_out = 1   |   |      main_en = 0          
                      |   |      aux_en = 1           
                      |   v      stop_out = 0         
                    +-+-----+                         
                    |       |                         
                    |       |<---+  stop_in           
                    | Stall |    |  +----------------+
                    |       |    |      sel = 0       
                    |       +----+      main_en = 0   
                    +-------+           aux_en = 0    
                                        stop_out = 1  
")]
//! For more details, see the paper.
//!
//!# Example
//!
//! The following example uses the `run_fn` method on
//! [Synchronous] circuits to provide a feedback-type test
//! harness (i.e., the input depends on the outputs of the
//! circuit at the last clock edge).
//!
//!```
#![doc = include_str!("../../examples/carloni.rs")]
//!```
//!
//! Here is the trace
//!
#![doc = include_str!("../../doc/carloni.md")]

use badascii_doc::{badascii, badascii_formal};
use rhdl::prelude::*;

use crate::core::dff;

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
/// A Carloni (skid) buffer
///
/// `T` is the type of the data flowing through
/// the buffer.
///
pub struct Carloni<T: Digital> {
    // The main FF
    main_ff: dff::DFF<T>,
    // The aux FF
    aux_ff: dff::DFF<T>,
    // The void FF
    void_ff: dff::DFF<bool>,
    // The state FF
    state_ff: dff::DFF<State>,
}

#[derive(PartialEq, Debug, Digital, Clone, Default)]
#[doc(hidden)]
pub enum State {
    #[default]
    Run,
    Stall,
}

impl<T: Digital> Default for Carloni<T> {
    fn default() -> Self {
        Self {
            main_ff: dff::DFF::new(T::dont_care()),
            aux_ff: dff::DFF::new(T::dont_care()),
            void_ff: dff::DFF::new(true),
            state_ff: dff::DFF::new(State::Run),
        }
    }
}

impl<T: Digital> Carloni<T> {
    /// Create a new [Carloni] buffer with the provided reset value
    pub fn new_with_reset_value(value: T) -> Self {
        Self {
            main_ff: dff::DFF::new(value),
            aux_ff: dff::DFF::new(value),
            void_ff: dff::DFF::new(true),
            state_ff: dff::DFF::new(State::Run),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone)]
/// The inputs for the [Carloni] buffer
pub struct In<T: Digital> {
    /// The data coming into the buffer
    pub data_in: T,
    /// The void signal coming in (indicates the `data_in` signal is valid or not)
    pub void_in: bool,
    /// The flow control `stop` signal coming from ownstream
    pub stop_in: bool,
}

#[derive(PartialEq, Debug, Digital, Clone)]
/// The outputs for the [Carloni] buffer
pub struct Out<T: Digital> {
    /// The data going downstream
    pub data_out: T,
    /// The void signal that indicates if the `data_out` is valid or not
    pub void_out: bool,
    /// The stop signal being propagated upstream
    pub stop_out: bool,
}

impl<T: Digital> SynchronousIO for Carloni<T> {
    type I = In<T>;
    type O = Out<T>;
    type Kernel = carloni_kernel<T>;
}

#[kernel]
#[doc(hidden)]
pub fn carloni_kernel<T: Digital>(cr: ClockReset, i: In<T>, q: Q<T>) -> (Out<T>, D<T>) {
    let mut d = D::<T>::dont_care();
    let mut o = Out::<T>::dont_care();
    // There are 4 control signals
    let mut sel = false;
    let mut main_en = false;
    let mut aux_en = false;
    let mut stop_out = false;
    // These are just renames for signals to match the diagram
    let void_out = q.void_ff;
    // Calculate the next state and update the control signals
    let will_stall = i.stop_in & (!i.void_in & !void_out);
    d.state_ff = q.state_ff;
    match q.state_ff {
        State::Run => {
            if !i.stop_in | (!i.void_in & void_out) {
                main_en = true;
            } else if will_stall {
                d.state_ff = State::Stall;
                aux_en = true;
            }
        }
        State::Stall => {
            if i.stop_in {
                stop_out = true;
            } else {
                sel = true;
                main_en = true;
                stop_out = true;
                d.state_ff = State::Run;
            }
        }
    }
    // Assemble the aux fifo
    d.aux_ff = if aux_en { i.data_in } else { q.aux_ff };
    let d_mux = if sel { q.aux_ff } else { i.data_in };
    d.main_ff = if main_en { d_mux } else { q.main_ff };
    let v_mux = if sel { false } else { i.void_in };
    d.void_ff = if main_en { v_mux } else { q.void_ff };
    o.data_out = q.main_ff;
    o.void_out = q.void_ff;
    o.stop_out = stop_out;
    if cr.reset.any() {
        o.void_out = true;
        o.stop_out = true;
    }
    (o, d)
}

#[cfg(test)]
mod tests {
    use rhdl::core::sim::ResetOrData;

    use crate::rng::xorshift::XorShift128;

    use super::*;
    #[test]
    fn test_no_combinatorial_paths() -> miette::Result<()> {
        let uut = Carloni::<b4>::default();
        drc::no_combinatorial_paths(&uut)?;
        Ok(())
    }

    #[test]
    fn test_carloni_buffer() {
        let uut = Carloni::<b32>::default();
        let mut need_reset = true;
        let mut source_rng = XorShift128::default();
        let mut output_rng = XorShift128::default();
        uut.run_fn(
            |out| {
                if need_reset {
                    need_reset = false;
                    return Some(ResetOrData::Reset);
                }
                let mut input = In::<b32>::dont_care();
                // Downstream reandomly wants to pause
                let want_to_pause = rand::random::<u8>() > 200;
                input.stop_in = want_to_pause;
                // Upstream may have paused
                let want_to_send = rand::random::<u8>() < 200;
                input.void_in = true;
                input.data_in = bits(0);
                if !out.stop_out && want_to_send {
                    // The receiver did not tell us to stop, and
                    // we want to send something
                    input.data_in = bits(source_rng.next().unwrap() as u128);
                    input.void_in = false;
                }
                // Check output
                if !out.void_out && !input.stop_in {
                    // The output will advance on this clock cycle
                    assert_eq!(out.data_out, bits(output_rng.next().unwrap() as u128));
                }
                Some(ResetOrData::Data(input))
            },
            100,
        )
        .take_while(|t| t.time < 100_000)
        .for_each(drop);
    }
}
