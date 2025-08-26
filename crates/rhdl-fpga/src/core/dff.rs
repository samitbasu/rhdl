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
use rhdl::{
    core::{
        hdl::ast::{
            always, assign, bit_string, id, if_statement, index_bit, initial,
            non_blocking_assignment, port, signed_width, unsigned_width, Declaration, Direction,
            Events, HDLKind, Module,
        },
        types::bit_string::BitString,
    },
    prelude::*,
};

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
        let mut module = Module {
            name: name.into(),
            ..Default::default()
        };
        let output_bits = T::bits();
        let init: BitString = self.reset.typed_bits().into();
        let data_width = if T::static_kind().is_signed() {
            signed_width(output_bits)
        } else {
            unsigned_width(output_bits)
        };
        module.ports = vec![
            port(
                "clock_reset",
                Direction::Input,
                HDLKind::Wire,
                unsigned_width(2),
            ),
            port("i", Direction::Input, HDLKind::Wire, data_width),
            port("o", Direction::Output, HDLKind::Reg, data_width),
        ];
        module.declarations.push(Declaration {
            kind: HDLKind::Wire,
            name: "clock".into(),
            width: unsigned_width(1),
            alias: None,
        });
        module.declarations.push(Declaration {
            kind: HDLKind::Wire,
            name: "reset".into(),
            width: unsigned_width(1),
            alias: None,
        });
        module
            .statements
            .push(initial(vec![assign("o", bit_string(&init))]));
        module
            .statements
            .push(continuous_assignment("clock", index_bit("clock_reset", 0)));
        module
            .statements
            .push(continuous_assignment("reset", index_bit("clock_reset", 1)));
        let dff = if_statement(
            id("reset"),
            vec![non_blocking_assignment("o", bit_string(&init))],
            vec![non_blocking_assignment("o", id("i"))],
        );
        let events = vec![Events::Posedge("clock".into())];
        module.statements.push(always(events, vec![dff]));
        Ok(HDLDescriptor {
            name: name.into(),
            body: module,
            children: Default::default(),
        })
    }
}
