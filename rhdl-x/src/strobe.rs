use anyhow::bail;
use anyhow::Result;
use petgraph::Direction;
use rhdl_bits::alias::*;
use rhdl_bits::{bits, Bits};
use rhdl_core::diagnostic::dfg::Component;
use rhdl_core::diagnostic::dfg::ComponentKind;
use rhdl_core::note;
use rhdl_core::path::Path;
use rhdl_core::CircuitIO;
use rhdl_core::Digital;
use rhdl_core::Tristate;
use rhdl_macro::{kernel, Digital};

use crate::circuit::root_descriptor;
use crate::circuit::root_hdl;
use crate::circuit::BufZ;
use crate::dff::DFFI;
use crate::trace::trace;
use crate::{circuit::Circuit, clock::Clock, constant::Constant, dff::DFF};
use rhdl_macro::Circuit;

// Build a strobe
#[derive(Clone, Circuit)]
#[rhdl(kernel = strobe::<N>)]
pub struct Strobe<const N: usize> {
    threshold: Constant<Bits<N>>,
    counter: DFF<Bits<N>>,
}

impl<const N: usize> Strobe<N> {
    pub fn new(param: Bits<N>) -> Self {
        Self {
            threshold: param.into(),
            counter: DFF::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct StrobeI {
    pub clock: Clock,
    pub enable: bool,
}

impl<const N: usize> CircuitIO for Strobe<N> {
    type I = StrobeI;
    type O = bool;
}

#[kernel]
pub fn strobe<const N: usize>(i: StrobeI, q: StrobeQ<N>) -> (bool, StrobeD<N>) {
    let mut d = StrobeD::<N>::default();
    note("i", i);
    note("q", q);
    d.counter.clock = i.clock;
    let counter_next = if i.enable { q.counter + 1 } else { q.counter };
    let strobe = i.enable & (q.counter == q.threshold);
    let counter_next = if strobe {
        bits::<{ N }>(1)
    } else {
        counter_next
    };
    d.counter.data = counter_next;
    note("out", strobe);
    note("d", d);
    (strobe, d)
}

#[test]
fn test_strobe_dfg() {
    let strobe = Strobe::<8>::new(bits::<8>(5));
    let descriptor = Strobe::<8>::descriptor(&strobe);
    let total_dfg = descriptor.dfg().unwrap();
    let dot = total_dfg.as_dot();
    std::fs::write("strobe.dot", dot).unwrap();
    // Look for a DFF
    let dff_node = total_dfg
        .graph
        .node_indices()
        .find(|node| {
            matches!(
                total_dfg.graph.node_weight(*node).map(|x| &x.kind),
                Some(ComponentKind::DFF)
            )
        })
        .unwrap();
    eprintln!("dff node is {:?}", dff_node);
    let comp = total_dfg.graph.node_weight(dff_node).unwrap();
    eprintln!("comp is {:?}", comp);
    for edge in total_dfg
        .graph
        .edges_directed(dff_node, Direction::Incoming)
    {
        eprintln!("edge is {:?}", edge);
    }
    let clock_path = Path::default().field("clock");
    trace(&total_dfg, dff_node, &clock_path).unwrap();
}

/*
#[kernel]
pub fn strobe<const N: usize>(
    i: StrobeI,
    (threshold_q, counter_q): (Bits<N>, Bits<N>),
) -> (bool, (Bits<N>, DFFI<Bits<N>>)) {
    let counter_next = if i.enable { counter_q + 1 } else { counter_q };
    let strobe = i.enable & (counter_q == threshold_q);
    let counter_next = if strobe {
        bits::<{ N }>(1)
    } else {
        counter_next
    };
    let dff_next = DFFI::<Bits<{ N }>> {
        clock: i.clock,
        data: counter_next,
    };
    (strobe, (threshold_q, dff_next))
}
*/
