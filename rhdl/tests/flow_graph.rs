use common::exhaustive;
use rhdl::prelude::*;
use stream::reset_pulse;

pub mod common;

pub mod anyer {
    use super::*;

    #[derive(Clone, Debug, Synchronous, Default)]
    #[rhdl(kernel = anyer)]
    pub struct U {}

    impl SynchronousIO for U {
        type I = b4;
        type O = bool;
    }

    impl SynchronousDQ for U {
        type D = ();
        type Q = ();
    }

    #[kernel]
    pub fn anyer(_cr: ClockReset, i: b4, _q: ()) -> (bool, ()) {
        (i.any(), ())
    }
}

pub mod adder {
    use super::*;

    #[derive(Clone, Debug, Synchronous, Default)]
    #[rhdl(kernel = adder)]
    pub struct U {}

    impl SynchronousIO for U {
        type I = (b4, b4);
        type O = b4;
    }

    impl SynchronousDQ for U {
        type D = ();
        type Q = ();
    }

    #[kernel]
    pub fn adder(_cr: ClockReset, i: (b4, b4), _q: ()) -> (b4, ()) {
        let (a, b) = i;
        let sum = a + b;
        (sum, ())
    }
}

pub mod selector {
    use super::*;

    #[derive(Clone, Debug, Synchronous, Default)]
    #[rhdl(kernel = selector)]
    pub struct U {}

    impl SynchronousIO for U {
        type I = (bool, b4, b4);
        type O = b4;
    }

    impl SynchronousDQ for U {
        type D = ();
        type Q = ();
    }

    #[kernel]
    pub fn selector(_cr: ClockReset, i: (bool, b4, b4), _q: ()) -> (b4, ()) {
        let (sel, a, b) = i;
        let out = if sel { a } else { b };
        (out, ())
    }
}

pub mod indexor {
    use super::*;

    #[derive(Clone, Debug, Synchronous, Default)]
    #[rhdl(kernel = indexor)]
    pub struct U {}

    impl SynchronousIO for U {
        type I = (b2, [b4; 4]);
        type O = b4;
    }

    impl SynchronousDQ for U {
        type D = ();
        type Q = ();
    }

    #[kernel]
    pub fn indexor(_cr: ClockReset, i: (b2, [b4; 4]), _q: ()) -> (b4, ()) {
        let (ndx, arr) = i;
        let out = arr[ndx];
        (out, ())
    }
}

pub mod splicer {
    use super::*;

    #[derive(Clone, Debug, Synchronous, Default)]
    #[rhdl(kernel = splicer)]
    pub struct U {}

    impl SynchronousIO for U {
        type I = (b2, [b4; 4], b4);
        type O = [b4; 4];
    }

    impl SynchronousDQ for U {
        type D = ();
        type Q = ();
    }

    #[kernel]
    pub fn splicer(_cr: ClockReset, i: (b2, [b4; 4], b4), _q: ()) -> ([b4; 4], ()) {
        let (ndx, mut arr, val) = i;
        arr[ndx] = val;
        (arr, ())
    }
}

#[test]
fn test_constant_propogation_through_selector_inline() -> miette::Result<()> {
    mod parent {
        use super::*;
        #[derive(Clone, Debug, Synchronous, Default)]
        #[rhdl(kernel = parent)]
        #[rhdl(auto_dq)]
        pub struct Parent {
            selector: selector::U,
        }

        impl SynchronousIO for Parent {
            type I = (b4, b4);
            type O = b4;
        }

        #[kernel]
        pub fn parent(_cr: ClockReset, i: (b4, b4), q: Q) -> (b4, D) {
            let (a, b) = i;
            let mut d = D::init();
            d.selector = (true, a, b);
            let o = q.selector;
            (o, d)
        }
    }

    let uut = parent::Parent::default();
    let inputs = exhaustive::<4>()
        .into_iter()
        .flat_map(|x| exhaustive::<4>().into_iter().map(move |y| (x, y)));
    let inputs = reset_pulse(4).chain(stream(inputs));
    let inputs = clock_pos_edge(inputs, 100);
    test_synchronous_hdl(&uut, inputs)?;
    let fg = uut.flow_graph()?;
    assert!(!fg
        .graph
        .node_weights()
        .any(|w| matches!(w.kind, ComponentKind::Select)));
    Ok(())
}

#[test]
fn test_add_inline() -> miette::Result<()> {
    mod parent {
        use super::*;
        #[derive(Clone, Debug, Synchronous, Default)]
        #[rhdl(kernel = parent)]
        #[rhdl(auto_dq)]
        pub struct Parent {
            adder: adder::U,
        }

        impl SynchronousIO for Parent {
            type I = (b4, b4);
            type O = b4;
        }

        #[kernel]
        pub fn parent(_cr: ClockReset, i: (b4, b4), q: Q) -> (b4, D) {
            let (a, b) = i;
            let mut d = D::init();
            d.adder = (a, b);
            let o = q.adder;
            (o, d)
        }
    }

    let uut = parent::Parent::default();
    let inputs = exhaustive::<4>()
        .into_iter()
        .flat_map(|x| exhaustive::<4>().into_iter().map(move |y| (x, y)));
    let inputs = reset_pulse(4).chain(stream(inputs));
    let inputs = clock_pos_edge(inputs, 100);
    test_synchronous_hdl(&uut, inputs)?;
    Ok(())
}

#[test]
fn test_constant_propagates_through_unary() -> miette::Result<()> {
    mod parent {
        use super::*;

        #[derive(Clone, Debug, Synchronous, Default)]
        #[rhdl(kernel = parent)]
        #[rhdl(auto_dq)]
        pub struct Parent {
            anyer: anyer::U,
        }

        impl SynchronousIO for Parent {
            type I = ();
            type O = bool;
        }

        #[kernel]
        pub fn parent(_cr: ClockReset, _i: (), q: Q) -> (bool, D) {
            let mut d = D::init();
            d.anyer = bits(3);
            let o = q.anyer;
            (o, d)
        }
    }

    let uut = parent::Parent::default();
    let inputs = reset_pulse(4).chain(stream(std::iter::once(())));
    let inputs = clock_pos_edge(inputs, 100);
    test_synchronous_hdl(&uut, inputs)?;
    let fg = uut.flow_graph()?;
    assert!(!fg
        .graph
        .node_weights()
        .any(|w| matches!(w.kind, ComponentKind::Unary(_))));
    Ok(())
}

#[test]
fn test_async_add() -> miette::Result<()> {
    #[derive(Clone, Debug, Circuit, Default)]
    #[rhdl(kernel = async_add)]
    pub struct U {}

    impl CircuitIO for U {
        type I = Signal<(b8, b8), Red>;
        type O = Signal<b8, Red>;
    }

    impl CircuitDQ for U {
        type D = Signal<(), Red>;
        type Q = Signal<(), Red>;
    }

    #[kernel]
    pub fn async_add(
        i: Signal<(b8, b8), Red>,
        q: Signal<(), Red>,
    ) -> (Signal<b8, Red>, Signal<(), Red>) {
        let (a, b) = i.val();
        (signal(a + b), q)
    }

    let uut = U::default();
    let inputs = exhaustive::<8>()
        .into_iter()
        .flat_map(|x| exhaustive::<8>().into_iter().map(move |y| (x, y)))
        .map(signal::<_, Red>)
        .enumerate()
        .map(|(ndx, val)| timed_sample(val, (ndx * 100) as u64));
    test_asynchronous_hdl(&uut, inputs)?;
    Ok(())
}

#[test]
fn test_constant_propagates_through_adder() -> miette::Result<()> {
    mod parent {
        use super::*;

        #[derive(Clone, Debug, Synchronous, Default)]
        #[rhdl(kernel = parent)]
        #[rhdl(auto_dq)]
        pub struct Parent {
            adder: adder::U,
        }

        impl SynchronousIO for Parent {
            type I = ();
            type O = b4;
        }

        #[kernel]
        pub fn parent(_cr: ClockReset, _i: (), q: Q) -> (b4, D) {
            let (a, b) = (bits(3), bits(4));
            let mut d = D::init();
            d.adder = (a, b);
            let o = q.adder;
            (o, d)
        }
    }

    let uut = parent::Parent::default();
    let inputs = reset_pulse(4).chain(stream(std::iter::once(())));
    let inputs = clock_pos_edge(inputs, 100);
    test_synchronous_hdl(&uut, inputs)?;
    let fg = uut.flow_graph()?;
    assert!(!fg
        .graph
        .node_weights()
        .any(|w| matches!(w.kind, ComponentKind::Binary(_))));
    Ok(())
}

#[test]
fn test_constant_propagates_through_indexing() -> miette::Result<()> {
    mod parent {
        use super::*;

        #[derive(Clone, Debug, Synchronous, Default)]
        #[rhdl(kernel = parent)]
        #[rhdl(auto_dq)]
        pub struct Parent {
            indexor: indexor::U,
        }

        impl SynchronousIO for Parent {
            type I = bool;
            type O = b4;
        }

        #[kernel]
        pub fn parent(_cr: ClockReset, i: bool, q: Q) -> (b4, D) {
            let mut d = D::init();
            let index = b2(3);
            d.indexor = (index, [bits(1), bits(2), bits(3), bits(4)]);
            let o = if i { q.indexor } else { bits(3) };
            (o, d)
        }
    }

    let uut = parent::Parent::default();
    let inputs = reset_pulse(4).chain(stream([false, true].iter().copied()));
    let inputs = clock_pos_edge(inputs, 100);
    test_synchronous_hdl(&uut, inputs)?;
    let fg = uut.flow_graph()?;
    assert!(!fg
        .graph
        .node_weights()
        .any(|w| matches!(w.kind, ComponentKind::DynamicIndex(_))));
    Ok(())
}

#[test]
fn test_constant_propagates_through_splicing() -> miette::Result<()> {
    mod parent {
        use super::*;

        #[derive(Clone, Debug, Synchronous, Default)]
        #[rhdl(kernel = parent)]
        #[rhdl(auto_dq)]
        pub struct Parent {
            splicer: splicer::U,
        }

        impl SynchronousIO for Parent {
            type I = bool;
            type O = [b4; 4];
        }

        #[kernel]
        pub fn parent(_cr: ClockReset, i: bool, q: Q) -> ([b4; 4], D) {
            let mut d = D::init();
            let index = b2(3);
            let orig = [bits(1), bits(2), bits(3), bits(4)];
            d.splicer = (index, orig, bits(5));
            let o = if i { q.splicer } else { orig };
            (o, d)
        }
    }

    let uut = parent::Parent::default();
    let inputs = reset_pulse(4).chain(stream([false, true].iter().copied()));
    let inputs = clock_pos_edge(inputs, 100);
    test_synchronous_hdl(&uut, inputs)?;
    let fg = uut.flow_graph()?;
    assert!(!fg
        .graph
        .node_weights()
        .any(|w| matches!(w.kind, ComponentKind::DynamicSplice(_))));
    Ok(())
}
