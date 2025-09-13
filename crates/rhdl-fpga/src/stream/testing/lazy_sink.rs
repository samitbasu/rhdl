//! A Lazy Random Stream Sink
//!
//! This source sink generates a sequence of
//! 32-bit pseudo-random numbers with a probability
//! of "sleeping" after generating each one.
//! For each number, the user provided function is
//! applied to the random number, and the result
//! compared with the incoming data.  A valid flag
//! is maintained as long as the sequence matches.
//!
//!# Schematic Symbol
//!
//! The lazy sink is easy to use:
#![doc = badascii_formal!(r"
       ++LazySink++     
  ?T   |          |     
+----->|data      |     
       |     valid+---->
<------+ready     |     
       |          |     
       +----------+     
")]
//!
//!
//!# Internal Details
//!
//! The [LazySink] core includes a
//! [XorShift] core to generate the
//! random numbers.  These numbers then
//! drive both the probability that the core
//! sleeps, and are passed to a use provided
//! synthesizable function to map to `T`.
//!
//! The internals look roughly like this:
//!
#![doc = badascii!(r"
                                            +------------+        
         +Strm2FIFO+                        v            |        
   ?T    |         | ?T    +-+func+-+ bool        +----+ |        
+------->|    data +------>|        +-----> & +-->|d  q+-+-> valid
 ready   |         |       +--------+             +----+          
<--------+    next |<-----+                                       
         |         |      +--------------------------+            
         +---------+                                 |            
                                  +-+XorRng+-+    +--+---+        
                                  |          |b32 |      |        
                           1 +--->|next   out+--->| > x? |        
                                  |          |    |      |        
                                  +----------+    +------+        
")]
//!
//! A [Stream2FIFO] buffer is used to collect incoming data and
//! hold it in a shallow FIFO.  The output of the FIFO is passed
//! to the user function to validate.  The validation flag is
//! latched when it goes low, indicating a problem with the incoming
//! data.  The [XorShift] random number generator is used to decide
//! when to possibly advance the FIFO and accept another data element.
//! As the probability is decreased, the sink becomes increasingly
//! "reluctant" to be ready, increasing backpressure on the upstream.
use badascii_doc::{badascii, badascii_formal};
