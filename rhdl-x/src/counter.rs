use rhdl::prelude::*;
use rhdl_fpga::core::dff;

mod kernel_host {
    use rhdl::prelude::*;

    #[derive(Clone, Debug, Default, Synchronous)]
    pub struct U {}

    impl SynchronousIO for U {
        type I = b8;
        type O = b16;
        type Kernel = my_kernel;
    }

    impl SynchronousDQ for U {
        type D = ();
        type Q = ();
    }

    #[kernel]
    fn sub(i: b8) -> b16 {
        i.resize()
    }

    #[kernel]
    pub fn my_kernel(cr: ClockReset, i: b8, _q: ()) -> (b16, ()) {
        if cr.reset.any() {
            (b16::default(), ())
        } else {
            (sub(i), ())
        }
    }
}

mod comb_adder {
    use rhdl::prelude::*;

    #[derive(Clone, Debug, Default, Synchronous)]
    pub struct U<const N: usize> {}

    impl<const N: usize> SynchronousIO for U<N> {
        type I = (Bits<N>, Bits<N>);
        type O = Bits<N>;
        type Kernel = adder<{ N }>;
    }

    impl<const N: usize> SynchronousDQ for U<N> {
        type D = ();
        type Q = ();
    }

    #[kernel]
    pub fn adder<const N: usize>(_cr: ClockReset, i: (Bits<N>, Bits<N>), _q: ()) -> (Bits<N>, ()) {
        let a = i;
        (a.0 + a.1, ())
    }
}

#[derive(PartialEq, Clone, Copy, Debug, Digital)]
pub struct I {
    pub enable: bool,
}

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
pub struct U<const N: usize> {
    count: dff::U<Bits<N>>,
    adder: comb_adder::U<{ N }>,
}

impl<const N: usize> Default for U<N> {
    fn default() -> Self {
        Self {
            count: dff::U::new(Bits::ZERO),
            adder: Default::default(),
        }
    }
}

impl<const N: usize> SynchronousIO for U<N> {
    type I = I;
    type O = Bits<N>;
    type Kernel = counter<{ N }>;
}

#[kernel]
pub fn counter<const N: usize>(cr: ClockReset, i: I, q: Q<N>) -> (Bits<N>, D<N>) {
    let next_count = if i.enable { q.adder } else { q.count };
    let mut output = Bits::<{ N }>::maybe_init();
    let mut d = D::<{ N }>::maybe_init();
    if !cr.reset.any() {
        output = q.count;
        d.count = next_count;
        d.adder = (q.count, bits(1));
    }
    (output, d)
}

#[cfg(test)]
mod tests {

    use petgraph::algo::is_cyclic_directed;

    use super::*;

    #[test]
    fn test_verilog_generation() -> miette::Result<()> {
        let uut: U<4> = U::default();
        let hdl = uut.hdl("uut")?;
        std::fs::write("counter.v", format!("{}", hdl.as_module())).unwrap();
        Ok(())
    }

    #[test]
    fn test_counter_timing_root() -> miette::Result<()> {
        use core::hash::Hasher;
        let uut: U<4> = U::default();
        let rtl = uut.descriptor("top")?.rtl.unwrap();
        eprintln!("rtl: {:?}", rtl);
        let fg = build_rtl_flow_graph(&rtl);
        let mut dot = std::fs::File::create("counter.dot").unwrap();
        write_dot(&fg, &mut dot).unwrap();
        let counter_uut = &uut.descriptor("uut")?.flow_graph;
        let mut dot = vec![0_u8; 0];
        write_dot(counter_uut, &mut dot).unwrap();
        let mut hasher = fnv::FnvHasher::default();
        hasher.write(&dot);
        let hash = hasher.finish();
        eprintln!("Dot hash: {:x}", hash);
        let mut dot = std::fs::File::create(format!("counter_{hash:x}.dot")).unwrap();
        write_dot(counter_uut, &mut dot).unwrap();
        assert!(!is_cyclic_directed(&counter_uut.graph));
        eprintln!("rtl: {:?}", rtl);
        Ok(())
    }
}
/*
The function to compute timing needs to look something like this:

fn timing(path: &Path, computer: &'dyn CostEstimator) -> Result<CostGraph, RHDLError> {
    let module = compile_design::<<Self as Synchronous>::Update>(CompilationMode::Synchronous)?;
    let top = &module.objects[&module.top];
    let timing = compute_timing_graph(&module, module.top, path, computer)?;
    // Check for inputs to the timing graph that come via the Q path
    let target_argument = top.arguments[2]; // The arguments are Reset, Input, Q
    for input in timing.inputs {
        if input.slot == target_argument {
            if Path::default().field("child1").is_prefix_of(&input.path) {
                let path = path.strip_prefix("child1");
                let child_timing = <Child1 as Synchronous>::timing(&path, computer)?
            }
        }
    }
}

*/
