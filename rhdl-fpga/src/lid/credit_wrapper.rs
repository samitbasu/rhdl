//! A Credit-based Pipeline Wrapper
//!
//!# Purpose
//!
//! The original Latency Insensitive Design work focused on stallable pipelines.  That
//! is to say, that if the `ready` signal was taken away (or equivalently, if the
//! downstream process asserted `stop`), then the entire pipeline would stall
//! until the `ready` signal was reasserted.  In the original papers, this was done via
//! a gated clock or a clock-enable signal that was used to either advance a given
//! stage in the pipeline or hold it in it's current state.  Roughly, something like
//! this:
#![doc = badascii!(r"
      +--+Pipeline+--+          
  ?S  |              | ?T        
+---->|in        out +---->     
      |              |          
      |     clk_en   |          
      +--------------+          
              ^                 
              |          ready  
              +----------------+
")]
//!
//! However, the idea of a `clk_en` line doesn't always fit with a pipeline.  For example,
//! a DRAM controller can be seen as a pipeline (where `S` is the address to read from and
//! `T` are the data elements read back, for example).  The DRAM controller is generally
//! not stallable.  And it is fair to assume that the controller requires you to read out the data
//! elements from the output once you have committed a certain transaction.  
//!
//! Furthermore, suppose that each item `S` injected into the pipeline produces `N` items of
//! type `T` on the output of the pipeline. Then when can a new item be injected?  If the pipeline
//! is opaque, then we can only keep track of how many pending items need to be written to the
//! output.
//!
//! The obvious answer is to include an output FIFO at the end of the pipeline to hold the
//! items as they are produced by the pipeline.  These can then be served to the downstream
//! process as it manages the `ready` signal.
//!
#![doc = badascii!(r"
      +--+Pipeline+--+     +--+FIFO+---+      
  ?S  |              | ?T  |           | ?T   
+---->|in        out +---->| in    out +----> 
      |              |     |           | ready
      |              |     |       next|<----+
      +--------------+     +-----------+      
")]
//!
//! Ignoring, temporarily the problem of underflow of the output FIFO, the bigger problem is the lack
//! of backpressure handling by the pipeline.  If the output FIFO is full, how do we stall the pipeline?
//! If it has no clock enable or other means of stalling, we are still in the same situation as before.
//!
//! The proposed solution in this core is to introduce a credit-based system.  A control core that
//! tracks the number of open slots in the output FIFO, and only dispatches as many items `S` such
//! that the output is guaranteed to fit in the output FIFO.  Each clock for which the `ready` signal
//! is asserted will release an additional credit to the controller, and each `S` item that is consumed
//! will require `N` credits to be available, where `N` is the number of `T` items produced by each `S`.
//!
//! Thus, backpressure is moved upstream of the pipeline.  The pipeline itself does not need to support
//! backpressure, since the controller will stop the inflow of data when there is insufficient credit
//! in the FIFO to start processing more data elements.
//!
//! Furthermore this design is invariant to the latency introduced by the pipeline.  It is irrelevant.  The
//! latency may even be variable.  Each output slot in the FIFO is reserved for a pending computation, and
//! credit tracking makes no assumptions about how long those reservations are held for.
//!
#![doc = badascii!(r"
            +                                                       
            |\         +--+Pipeline+--+     +--+FIFO+---+           
     None+->|0+    ?S  |              | ?T  |           | ?T  data  
        ?S  | +------->|in        out +---->| in    out +---------->
data +----->|1|        |              |     |           |     ready 
            | +        |              |     |       next|<----+----+
            |/         +--------------+     +-----------+     |     
            +^                                                |     
             |         +--------------+                       |     
ready        +-------->|   Control    |<----------------------+     
<----------------------+              |                             
                       +--------------+                             
")]
//!
//!# Schematic Symbol
//!
//! Here is the schematic symbol for the [CreditPipelineWrapper].
//!
#![doc = badascii_formal!("
      +-+CreditWrapper+----+       
  ?S  |                    |  ?T   
+---->| data         data  +------>
      |                    |       
      |                    |       
<-----+ ready        ready |<-----+
      |                    |       
   ?S |                    |  ?T   
<-----+ to_pipe  from_pipe |<-----+
      |                    |       
      +--------------------+       
")]
//!
//! It is understood that the pipline will start when fed `Some(S)` data
//! element, and will produce multiple [Option<T>] output elements.  The
//! number of outputs per input is a const generic argument to the core.
//! Also, the internal FIFO size is exposed, since knowledge of how big the
//! output FIFO will need to be is a design decision.
use badascii_doc::{badascii, badascii_formal};
