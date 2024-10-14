use common::exhaustive;
use rhdl::prelude::*;
use rhdl_core::test_module::test_flowgraph_for_synchronous;

pub mod common;

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
    let inputs = stream(inputs);
    let inputs = clock_pos_edge(inputs, 100);
    let tm = test_flowgraph_for_synchronous(&uut, inputs)?;
    tm.run_iverilog()?;
    Ok(())
}
