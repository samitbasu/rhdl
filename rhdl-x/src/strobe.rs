use std::collections::HashMap;
use std::ops::Range;

use anyhow::bail;
use anyhow::Result;
use miette::LabeledSpan;
use miette::MietteError;
use miette::NamedSource;
use miette::SourceCode;
use miette::SourceSpan;
use miette::SpanContents;
use petgraph::Direction;
use rhdl_bits::alias::*;
use rhdl_bits::{bits, Bits};
use rhdl_core::ast::ast_impl::FunctionId;
use rhdl_core::compile_design;
use rhdl_core::crusty::index::IndexedSchematic;
use rhdl_core::diagnostic::SpannedSource;
use rhdl_core::note;
use rhdl_core::path::Path;
use rhdl_core::rhif::object::SourceLocation;
use rhdl_core::rhif::spec::FuncId;
use rhdl_core::schematic::components::ComponentKind;
use rhdl_core::schematic::schematic_impl::PinPath;
use rhdl_core::schematic::schematic_impl::Trace;
use rhdl_core::Circuit;
use rhdl_core::CircuitIO;
use rhdl_core::Digital;
use rhdl_core::KernelFnKind;
use rhdl_core::Tristate;
use rhdl_macro::{kernel, Digital};

use crate::dff::DFFI;
use crate::{clock::Clock, constant::Constant, dff::DFF};
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
fn child_one(i: b4) -> b4 {
    i
}

#[kernel]
fn child_two(i: b8) -> b8 {
    i
}

#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct DemoQ {
    pub child_one: b4,
    pub child_two: b8,
}

#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct DemoD {
    pub child_one: b4,
    pub child_two: b8,
}

#[kernel]
fn update_me(i: b1, q: DemoQ) -> (b1, DemoD) {
    (
        i,
        DemoD {
            child_one: child_one(q.child_one),
            child_two: child_two(q.child_two),
        },
    )
}

#[kernel]
fn circuit(i: b1) -> b1 {
    let mut q = DemoQ::default();
    let (o, d) = update_me(i, q);
    q.child_one = child_one(d.child_one);
    q.child_two = child_two(d.child_two);
    o
}

#[kernel]
pub fn add_one<const N: usize>(count: Bits<N>) -> Bits<N> {
    count + 1
}

#[kernel]
pub fn add_enabled<const N: usize>(enable: bool, count: Bits<N>) -> Bits<N> {
    if enable {
        add_one::<{ N }>(count)
    } else {
        count
    }
}

#[kernel]
pub fn strobe<const N: usize>(i: StrobeI, q: StrobeQ<N>) -> (bool, StrobeD<N>) {
    let mut d = StrobeD::<N>::default();
    note("i", i);
    note("q", q);
    //d.counter.clock = i.clock;
    //    let counter_next = if i.enable { q.counter + 1 } else { q.counter };
    let counter_next = add_enabled::<{ N }>(i.enable, q.counter);
    let strobe = i.enable & (q.counter == q.threshold);
    let counter_next = if strobe {
        bits::<{ N }>(1)
    } else {
        counter_next
    };
    let jnk = add_enabled::<{ N }>(i.enable, q.counter);
    let hoo = add_one::<{ N }>(jnk);
    let jaz = if strobe { counter_next } else { counter_next };
    d.counter.data = counter_next;
    note("out", strobe);
    note("d", d);
    (strobe, d)
}

#[test]
fn test_circuit_schematic() {
    use rhdl_core::DigitalFn;
    let Some(KernelFnKind::Kernel(kernel)) = circuit::kernel_fn() else {
        panic!("No kernel function");
    };
    let module = compile_design(kernel).unwrap();
    let schematic = rhdl_core::schematic::builder::build_schematic(&module, module.top)
        .unwrap()
        .inlined();
    let mut dot = std::fs::File::create("circuit_schematic.dot").unwrap();
    rhdl_core::schematic::dot::write_dot(&schematic, None, &mut dot).unwrap();
}

#[test]
fn test_schematic() {
    use rhdl_core::DigitalFn;

    let Some(KernelFnKind::Kernel(kernel)) = strobe::<8>::kernel_fn() else {
        panic!("No kernel function");
    };
    let design = compile_design(kernel).unwrap();
    let schematic = rhdl_core::schematic::builder::build_schematic(&design, design.top).unwrap();
    let mut dot = std::fs::File::create("strobe_schematic.dot").unwrap();
    rhdl_core::schematic::dot::write_dot(&schematic, None, &mut dot).unwrap();
}

#[test]
fn test_simple_schematic_inlined() {
    use rhdl_core::DigitalFn;
    let Some(KernelFnKind::Kernel(kernel)) = add_enabled::<8>::kernel_fn() else {
        panic!("No kernel function");
    };
    let module = compile_design(kernel).unwrap();
    let schematic = rhdl_core::schematic::builder::build_schematic(&module, module.top)
        .unwrap()
        .inlined();
    schematic
        .components
        .iter()
        .enumerate()
        .for_each(|(ndx, c)| {
            eprintln!("component {} kind {:?}", ndx, c.kind);
        });
    schematic.wires.iter().for_each(|w| {
        eprintln!("wire {:?}", w);
    });
    let schematic = schematic.inlined();
    schematic
        .components
        .iter()
        .enumerate()
        .for_each(|(ndx, c)| {
            eprintln!("component {} kind {:?}", ndx, c.kind);
        });
    schematic.wires.iter().for_each(|w| {
        eprintln!("wire {:?}", w);
    });
    let mut dot = std::fs::File::create("add_enabled_schematic.dot").unwrap();
    rhdl_core::schematic::dot::write_dot(&schematic, None, &mut dot).unwrap();
}

#[test]
fn test_schematic_inlined() {
    use rhdl_core::DigitalFn;

    let Some(KernelFnKind::Kernel(kernel)) = strobe::<8>::kernel_fn() else {
        panic!("No kernel function");
    };
    let design = compile_design(kernel).unwrap();
    let schematic = rhdl_core::schematic::builder::build_schematic(&design, design.top)
        .unwrap()
        .inlined();
    schematic
        .components
        .iter()
        .enumerate()
        .for_each(|(ndx, c)| {
            eprintln!("component {} path {:?}", ndx, c.path);
        });
    let mut dot = std::fs::File::create("strobe_schematic.dot").unwrap();
    rhdl_core::schematic::dot::write_dot(&schematic, None, &mut dot).unwrap();
}

#[test]
fn test_strobe_schematic() {
    let strobe = Strobe::<8>::new(bits::<8>(5));
    let descriptor = Strobe::<8>::descriptor(&strobe);
    let schematic = descriptor.schematic().unwrap();
    let clock_pin_path = PinPath {
        pin: schematic.inputs[0],
        path: Path::default().field("clock"),
    };
    let schematic = schematic.inlined();
    for sink in rhdl_core::crusty::downstream::follow_pin_downstream(
        &schematic.clone().into(),
        clock_pin_path,
    )
    .unwrap()
    .sinks
    {
        eprintln!("sink is {:?}", sink);
    }
    let dff = schematic
        .components
        .iter()
        .find_map(|c| match &c.kind {
            ComponentKind::DigitalFlipFlop(d) => Some(d),
            _ => None,
        })
        .unwrap();
    eprintln!("dff is {:?}", dff);
    let dff_clock_pin = dff.clock;
    let dff_source_path = rhdl_core::crusty::upstream::follow_pin_upstream(
        &schematic.clone().into(),
        PinPath {
            pin: dff_clock_pin,
            path: Path::default(),
        },
    )
    .unwrap();
    let report = trace_diagnostic(&schematic.clone().into(), &dff_source_path);
    eprintln!("report is {:?}", report);
    for segment in &dff_source_path.paths {
        eprintln!("segment is {:?}", segment);
    }
    let mut dot = std::fs::File::create("strobe_inlined.dot").unwrap();
    rhdl_core::schematic::dot::write_dot(&schematic, Some(&dff_source_path), &mut dot).unwrap();
}

// Notes to think about:
// 1. Make schematic inlined always.
// 2. Generate trace items using the operators as well.
// So something like this:
//   source -> pin -> "from here"
//              -> op "via this op"
//   dest -> pin -> "to here"

fn trace_diagnostic(is: &IndexedSchematic, trace: &Trace) -> miette::Report {
    let pool = SourcePool::new(is.schematic.source.clone());
    let source_locations = trace
        .paths
        .iter()
        .filter_map(|p| is.schematic.pin(p.source).location)
        .collect::<Vec<_>>();
    for loc in source_locations {
        let func = pool.source.get(&loc.func).unwrap();
        eprintln!("Location: {:?}", loc);
        eprintln!("span: {:?}", func.span(loc.node));
        eprintln!("Text: {}", func.text(loc.node));
    }

    let labels = trace.paths.iter().filter_map(|p| {
        is.schematic
            .pin(p.source)
            .location
            .inspect(|x| eprintln!("location is {:?}", x))
            .and_then(|l| pool.get_range_from_location(l))
            .inspect(|x| eprintln!("range is {:?}", x))
            .map(|range| {
                LabeledSpan::new(
                    Some(format!("{}", p.source)),
                    range.start,
                    range.end - range.start,
                )
            })
            .inspect(|x| eprintln!("label is {:?}", x))
    });
    let diagnostic = miette::MietteDiagnostic::new("Clock error")
        .with_help("Check that the clock is connected to a clock source")
        .with_code("clock_error")
        .with_severity(miette::Severity::Warning)
        .with_labels(labels);
    miette::Report::new(diagnostic).with_source_code(pool)
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
