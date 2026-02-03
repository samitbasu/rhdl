#![allow(dead_code)]
use rhdl::{
    core::{ScopedName, SyncKind},
    prelude::*,
};

struct Foo;

impl Foo {
    // ANCHOR: descriptor
    pub fn descriptor(&self, scoped_name: ScopedName) -> Result<Descriptor<SyncKind>, RHDLError>
// ANCHOR_END: descriptor
    {
        let _ = scoped_name;
        todo!()
    }
}

pub mod children {
    use rhdl::{
        core::{ScopedName, SyncKind},
        prelude::*,
    };

    // ANCHOR: children
    pub trait Synchronous: 'static + Sized + SynchronousIO {
        // snip...
        /// Iterate over the child circuits of this circuit.
        fn children(
            &self,
            _parent_scope: &ScopedName,
        ) -> impl Iterator<Item = Result<Descriptor<SyncKind>, RHDLError>> {
            std::iter::empty()
        }
    }
    // ANCHOR_END: children
}

pub mod sim {
    use rhdl::prelude::*;

    // ANCHOR: sim-signature
    pub trait Synchronous: 'static + Sized + SynchronousIO {
        // State storage type
        type S: PartialEq + Clone;
        // snip...
        //                 👇 - extra argument
        fn sim(&self, clock_reset: ClockReset, input: Self::I, state: &mut Self::S) -> Self::O;
        // snip...
    }
    // ANCHOR_END: sim-signature
}

pub mod io {
    use rhdl::prelude::*;
    // ANCHOR: synchronous-io

    pub trait SynchronousIO: 'static + SynchronousDQ {
        type I: Digital;
        type O: Digital;
        type Kernel: DigitalFn
            + DigitalFn3<A0 = ClockReset, A1 = Self::I, A2 = Self::Q, O = (Self::O, Self::D)>;
    }

    // ANCHOR_END: synchronous-io
}

pub mod kernel_def {
    use rhdl::prelude::*;

    pub trait SynchronousIO: 'static + SynchronousDQ {
        type I: Digital;
        type O: Digital;
        // ANCHOR: kernel-def
        // 👇 Kernel def
        type Kernel: DigitalFn
            + DigitalFn3<A0 = ClockReset, A1 = Self::I, A2 = Self::Q, O = (Self::O, Self::D)>;
        // ANCHOR_END: kernel-def
    }
}

pub mod io_focus {
    use rhdl::prelude::*;

    // ANCHOR: synchronous-io-focus
    pub trait SynchronousIO: 'static + SynchronousDQ {
        type I: Digital;
        type O: Digital;
        // snip
    }
    // ANCHOR_END: synchronous-io-focus
}

pub mod xor_example {
    use rhdl::prelude::*;

    #[derive(Circuit, Clone, Copy, CircuitDQ)]
    pub struct XorGate;

    // ANCHOR: xor-io

    impl CircuitIO for XorGate {
        type I = Signal<(bool, bool), Red>;
        type O = Signal<bool, Red>;
        type Kernel = xor_gate;
    }

    // ANCHOR_END: xor-io

    #[kernel]
    pub fn xor_gate(i: Signal<(bool, bool), Red>, _q: XorGateQ) -> (Signal<bool, Red>, XorGateD) {
        let (a, b) = i.val(); // a and b are both bool
        let c = a ^ b; // Exclusive OR
        (signal(c), XorGateD {})
    }
}

pub mod xor_generic {
    use rhdl::prelude::*;

    #[derive(Clone, Copy)]
    pub struct XorGate<D: Domain> {
        _phantom: std::marker::PhantomData<D>,
    }

    impl<D: Domain> Default for XorGate<D> {
        fn default() -> Self {
            Self {
                _phantom: std::marker::PhantomData,
            }
        }
    }

    impl<D: Domain> CircuitDQ for XorGate<D> {
        type D = ();
        type Q = ();
    }

    // ANCHOR: xor-io-generic

    impl<D: Domain> CircuitIO for XorGate<D> {
        type I = Signal<(bool, bool), D>;
        type O = Signal<bool, D>;
        type Kernel = xor_gate_generic<D>;
    }

    // ANCHOR_END: xor-io-generic

    #[kernel]
    pub fn xor_gate_generic<D: Domain>(
        i: Signal<(bool, bool), D>,
        _q: (),
    ) -> (Signal<bool, D>, ()) {
        let (a, b) = i.val(); // a and b are both bool
        let c = a ^ b; // Exclusive OR
        (signal(c), ())
    }
}

pub mod xor_sync {
    use rhdl::prelude::*;

    #[derive(Synchronous, SynchronousDQ, Clone, Copy)]
    pub struct XorGate;

    // ANCHOR: xor-sync-io

    impl SynchronousIO for XorGate {
        type I = (bool, bool);
        type O = bool;
        type Kernel = xor_gate_sync;
    }

    // ANCHOR_END: xor-sync-io

    #[kernel]
    pub fn xor_gate_sync(
        _clock_reset: ClockReset,
        i: (bool, bool),
        _q: XorGateQ,
    ) -> (bool, XorGateD) {
        let (a, b) = i; // a and b are both bool
        let c = a ^ b; // Exclusive OR
        (c, XorGateD {})
    }
}

pub mod dq {
    use rhdl::prelude::*;

    // ANCHOR: synchronous-dq

    pub trait SynchronousDQ: 'static {
        type Q: Digital;
        type D: Digital;
    }

    // ANCHOR_END: synchronous-dq
}

pub mod dq_example {
    use rhdl::prelude::*;

    type A = std::marker::PhantomData<()>;
    type B = std::marker::PhantomData<()>;
    type C = std::marker::PhantomData<()>;

    // ANCHOR: xd-def

    #[derive(Digital, Copy, Clone, PartialEq)]
    pub struct XD {
        pub child_1: <A as SynchronousIO>::I,
        pub child_2: <B as SynchronousIO>::I,
        pub child_3: <C as SynchronousIO>::I,
    }

    // ANCHOR_END: xd-def

    // ANCHOR: xq-def

    #[derive(Digital, Copy, Clone, PartialEq)]
    pub struct XQ {
        pub child_1: <A as SynchronousIO>::O,
        pub child_2: <B as SynchronousIO>::O,
        pub child_3: <C as SynchronousIO>::O,
    }

    // ANCHOR_END: xq-def

    // ANCHOR: x-sync-def

    #[derive(Synchronous)]
    pub struct X {
        pub child_1: A,
        pub child_2: B,
        pub child_3: C,
    }

    // ANCHOR_END: x-sync-def

    impl SynchronousDQ for X {
        type Q = XQ;
        type D = XD;
    }

    impl SynchronousIO for X {
        type I = ();
        type O = ();
        type Kernel = NoSynchronousKernel<ClockReset, (), XQ, ((), XD)>;
    }
}

pub mod dq_derive_example {
    use rhdl::prelude::*;

    type A = std::marker::PhantomData<()>;
    type B = std::marker::PhantomData<()>;
    type C = std::marker::PhantomData<()>;

    // ANCHOR: x-sync-derive-def

    //                       👇 new!
    #[derive(Synchronous, SynchronousDQ)]
    pub struct X {
        pub child_1: A,
        pub child_2: B,
        pub child_3: C,
    }

    // ANCHOR_END: x-sync-derive-def

    impl SynchronousIO for X {
        type I = ();
        type O = ();
        type Kernel = NoSynchronousKernel<ClockReset, (), XQ, ((), XD)>;
    }
}

pub mod sim_trait {
    use rhdl::prelude::*;

    // ANCHOR: synchronous-sim-trait
    pub trait Synchronous: 'static + Sized + SynchronousIO {
        type S: PartialEq + Clone;
        fn init(&self) -> Self::S;
        fn sim(&self, clock_reset: ClockReset, input: Self::I, state: &mut Self::S) -> Self::O;
        // snip
    }
    // ANCHOR_END: synchronous-sim-trait
}

pub mod sim_example {
    use rhdl::prelude::*;

    #[derive(Synchronous, SynchronousDQ, Clone, Copy)]
    pub struct MyCircuit;

    impl SynchronousIO for MyCircuit {
        type I = (bool, bool);
        type O = bool;
        type Kernel = or_kernel;
    }

    #[kernel]
    pub fn or_kernel(_cr: ClockReset, i: (bool, bool), _q: MyCircuitQ) -> (bool, MyCircuitD) {
        let (a, b) = i;
        (a | b, MyCircuitD {})
    }

    #[test]
    fn test_sim() {
        let mut inputs = [(true, false), (false, true), (true, true), (false, false)].into_iter();
        let clock = [clock(false), clock(true)].into_iter().cycle();
        let mut cr = clock.map(|c| clock_reset(c, reset(false)));
        // ANCHOR: synchronous-sim

        // Get an instance of the circuit you want to simulate
        let uut = MyCircuit;
        // Get the initial state
        let mut state = uut.init();
        // Loop until done
        loop {
            // Update the clock and reset signal
            let clock_reset = cr.next().unwrap();
            // Get the next input
            let Some(input) = inputs.next() else {
                break;
            };
            // Simulate the output of the circuit
            let output = uut.sim(clock_reset, input, &mut state);
            // Do something with it
            assert_eq!(output, input.0 | input.1);
        }

        // ANCHOR_END: synchronous-sim
    }
}

mod kernel_example {
    use rhdl::prelude::*;
    use rhdl_fpga::core::dff;

    #[derive(Clone, Debug, Synchronous, SynchronousDQ)]
    #[rhdl(dq_no_prefix)]
    /// The counter core
    ///   `N` is the bitwidth of the counter
    pub struct Counter<const N: usize>
    where
        rhdl::bits::W<N>: BitWidth,
    {
        count: dff::DFF<Bits<N>>,
    }

    impl<const N: usize> Default for Counter<N>
    where
        rhdl::bits::W<N>: BitWidth,
    {
        fn default() -> Self {
            Self {
                count: dff::DFF::new(Bits::<N>::default()),
            }
        }
    }

    impl<const N: usize> SynchronousIO for Counter<N>
    where
        rhdl::bits::W<N>: BitWidth,
    {
        type I = bool;
        type O = Bits<N>;
        type Kernel = counter<N>;
    }

    // ANCHOR: counter-kernel

    #[kernel]
    /// Counter kernel function
    pub fn counter<const N: usize>(cr: ClockReset, enable: bool, q: Q<N>) -> (Bits<N>, D<N>)
    where
        rhdl::bits::W<N>: BitWidth,
    {
        let next_count = if enable { q.count + 1 } else { q.count };
        let next_count = if cr.reset.any() { bits(0) } else { next_count };
        (q.count, D::<N> { count: next_count })
    }

    // ANCHOR_END: counter-kernel
}

pub mod sync_trait {
    use rhdl::{
        core::{ScopedName, SyncKind},
        prelude::*,
    };

    // ANCHOR: synchronous_trait
    pub trait Synchronous: 'static + Sized + SynchronousIO {
        type S: PartialEq + Clone;

        fn init(&self) -> Self::S;

        fn sim(&self, clock_reset: ClockReset, input: Self::I, state: &mut Self::S) -> Self::O;

        fn descriptor(&self, scoped_name: ScopedName) -> Result<Descriptor<SyncKind>, RHDLError>;

        fn children(
            &self,
            _parent_scope: &ScopedName,
        ) -> impl Iterator<Item = Result<Descriptor<SyncKind>, RHDLError>>;
    }
    // ANCHOR_END: synchronous_trait
}

pub mod synth {
    use rhdl::{
        core::{ScopedName, SyncKind},
        prelude::*,
    };

    // ANCHOR: synchronous_trait_descriptor

    pub trait Synchronous: 'static + Sized + SynchronousIO {
        // snip
        fn descriptor(&self, scoped_name: ScopedName) -> Result<Descriptor<SyncKind>, RHDLError>;
        // snip
    }

    // ANCHOR_END: synchronous_trait_descriptor
}

pub mod descriptor {
    use std::marker::PhantomData;

    use rhdl::prelude::*;

    // ANCHOR: descriptor

    /// Run time description of a circuit.
    #[derive(Debug)]
    pub struct Descriptor<T> {
        /// snip
        pub hdl: Option<HDLDescriptor>,
        /// Phantom data for the marker type.
        pub _phantom: PhantomData<T>,
    }

    // ANCHOR_END: descriptor
}

pub mod hdl_descriptor {
    use rhdl::prelude::*;

    // ANCHOR: hdl-descriptor

    #[derive(Clone, Hash, Debug)]
    pub struct HDLDescriptor {
        /// The unique name of the circuit.
        pub name: String,
        /// The list of modules that make up this circuit.
        pub modules: vlog::ModuleList,
    }

    // ANCHOR_END: hdl-descriptor
}

pub mod verilog_example {
    #[test]
    fn test_verilog_syntax() -> miette::Result<()> {
        use rhdl::prelude::*;
        // ANCHOR: verilog

        // Make a 4 bit counter
        let uut = rhdl_fpga::core::counter::Counter::<4>::default();
        // Get the run time descriptor
        let desc = uut.descriptor(ScopedName::top())?;
        // Get a reference to the checked HDL descriptor
        let hdl = desc.hdl()?;
        // Write it out to a file
        std::fs::write("counter.v", hdl.modules.pretty()).unwrap();

        // ANCHOR_END: verilog
        Ok(())
    }
}
