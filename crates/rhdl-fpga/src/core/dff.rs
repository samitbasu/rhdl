//! Digital Flip Flop
//!
//! The [DFF] is the building block for nearly all synchronous
//! logic.  The [DFF] is generic over the type `T` it holds,
//! and can store a custom value for the reset value to take.
//! The [DFF] is positive edge triggered, with an active
//! high reset.  It is a [Synchronous] component, and meant
//! to be used in synchronous circuits.  
//!
//! Here is the schematic symbol
//!
#![doc = badascii_doc::badascii_formal!(r"
    +--+DFF+----+   
  T |           | T 
+-->|input  out +-->
    | (d)   (q) |   
    |           |   
    +-----------+   
")]
//!
//!# Timing
//!
//! Good resources are available online to understand how digital
//! flip flops work, and what they do.  The simplest explanation is
//! that they operate like 1-tap delay lines, in which the data
//! presented at a clock edge appears on the output at the next
//! clock edge.  
//!
//! Roughly, this looks like this:
//!
#![doc = badascii_doc::badascii!(r"
          +---+   +---+   +---+   +---+
 clk  +---+   +---+   +---+   +---+    
                                       
      +-+D1+--+-+D2+--+-+D3+--+-+D4+--+
 input+---+---+-------+-------+-------+
          +---------+                  
                    v                  
      +-+XX+--+-+D1+--+-+D2+--+-+D3+--+
output+-------+-------+-------+-------+
")]
//! One of the primary purposes of [DFF]s in designs is to decouple
//! the input side of the flip flop from the output side.  This allows
//! the output side of the FF to only depend on a fixed quantity (the
//! last recorded value), instead of the potentially changing input
//! side.
//!
//! [DFF]s are used in state machines, as memory storage elements,
//! and to break up pipelines.
//!
//!# Example
//!
//! Here is a simple example of a state machine recognizing a sequence
//! using a [DFF].
//!
//!```
#![doc = include_str!("../../examples/dff.rs")]
//!```
//!
//! The trace shows the FSM working.
#![doc = include_str!("../../doc/dff.md")]
use quote::format_ident;
use rhdl::prelude::*;
use syn::parse_quote;

#[derive(PartialEq, Debug, Clone)]
/// Basic Digital Flip Flop
///
/// Carries type `T`, with a given
/// reset value.  Is positive edge
/// triggered on the synchronous clock.
pub struct DFF<T: Digital> {
    reset: T,
}

impl<T: Digital> DFF<T> {
    /// Create a new [DFF] with the
    /// provided reset value.
    pub fn new(reset: T) -> Self {
        Self { reset }
    }
}

impl<T: Digital + Default> Default for DFF<T> {
    fn default() -> Self {
        Self {
            reset: T::default(),
        }
    }
}

impl<T: Digital> SynchronousIO for DFF<T> {
    type I = T;
    type O = T;
    type Kernel = NoKernel3<ClockReset, T, (), (T, ())>;
}

impl<T: Digital> SynchronousDQ for DFF<T> {
    type D = ();
    type Q = ();
}

#[derive(PartialEq, Debug, Digital)]
#[doc(hidden)]
pub struct S<T: Digital> {
    cr: ClockReset,
    reset: Reset,
    current: T,
    next: T,
}

impl<T: Digital> Synchronous for DFF<T> {
    type S = S<T>;

    fn init(&self) -> Self::S {
        Self::S::dont_care()
    }

    fn sim(&self, clock_reset: ClockReset, input: Self::I, state: &mut Self::S) -> Self::O {
        trace_push_path("dff");
        trace("input", &input);
        let clock = clock_reset.clock;
        let reset = clock_reset.reset;
        if !clock.raw() {
            state.next = input;
            state.reset = reset;
        }
        if clock.raw() && !state.cr.clock.raw() {
            if state.reset.raw() {
                state.current = self.reset;
            } else {
                state.current = state.next;
            }
        }
        state.cr = clock_reset;
        trace("output", &state.current);
        trace_pop_path();
        state.current
    }

    fn description(&self) -> String {
        format!(
            "Positive edge triggered DFF holding value of type {:?}, with reset value of {:?}",
            T::static_kind(),
            self.reset.typed_bits()
        )
    }

    fn hdl(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        self.as_verilog(name)
    }

    fn descriptor(&self, name: &str) -> Result<CircuitDescriptor, RHDLError> {
        let ntl = rhdl::core::ntl::builder::synchronous_black_box(self, name)?;
        Ok(CircuitDescriptor {
            unique_name: name.to_string(),
            input_kind: Self::I::static_kind(),
            output_kind: Self::O::static_kind(),
            d_kind: Kind::Empty,
            q_kind: Kind::Empty,
            children: Default::default(),
            ntl,
            rtl: None,
        })
    }
}

impl<T: Digital> DFF<T> {
    fn as_verilog(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        let module_name = format_ident!("{}", name);
        let init: vlog::LitVerilog = self.reset.typed_bits().into();
        let data_width: vlog::BitRange = (0..T::static_kind().bits()).into();
        let reset_index = bit_range(ClockReset::static_kind(), &path!(.reset))?;
        let reset_index = syn::Index::from(reset_index.0.start);
        let clock_index = bit_range(ClockReset::static_kind(), &path!(.clock))?;
        let clock_index = syn::Index::from(clock_index.0.start);
        let module: vlog::ModuleDef = parse_quote! {
            module #module_name(
                input wire [1:0] clock_reset,
                input wire [#data_width] i,
                output reg [#data_width] o
            );
                wire clock;
                wire reset;
                assign clock = clock_reset[#clock_index];
                assign reset = clock_reset[#reset_index];
                initial begin
                    o = #init;
                end
                always @(posedge clock) begin
                    if (reset) begin
                        o <= #init;
                    end else begin
                        o <= i;
                    end
                end
            endmodule
        };
        Ok(HDLDescriptor {
            name: name.into(),
            body: module,
            children: Default::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_hdl_output() -> miette::Result<()> {
        let expect = expect_test::expect![[r#"
            module top(input wire [1:0] clock_reset, input wire [3:0] i, output reg [3:0] o);
               wire  clock;
               wire  reset;
               assign clock = clock_reset[0];
               assign reset = clock_reset[1];
               initial begin
                  o = 4'b1010;
               end
               always @(posedge clock) begin
                  if (reset) begin
                     o <= 4'b1010;
                  end else begin
                     o <= i;
                  end
               end
            endmodule
        "#]];
        let uut: DFF<b4> = DFF::new(bits(0b1010));
        let hdl = uut.hdl("top")?.as_module().pretty();
        expect.assert_eq(&hdl);
        Ok(())
    }
}
