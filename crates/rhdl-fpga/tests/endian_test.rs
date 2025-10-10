use expect_test::expect;
use rhdl::prelude::*;

mod sub {
    use rhdl::prelude::*;
    use rhdl_fpga::core::dff;

    #[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
    pub struct U {
        data: dff::DFF<b2>,
    }

    #[derive(PartialEq, Clone, Digital)]
    pub struct I {
        pub data: Option<(bool, b2)>,
        pub ready: bool,
    }

    #[derive(PartialEq, Clone, Digital)]
    pub struct O {
        pub done: bool,
        pub data: b2,
    }

    impl SynchronousIO for U {
        type I = I;
        type O = O;
        type Kernel = bottom_kernel;
    }

    #[kernel]
    pub fn bottom_kernel(cr: ClockReset, i: I, q: Q) -> (O, D) {
        let mut d = D::dont_care();
        let mut o = O::dont_care();
        d.data = q.data;
        o.done = false;
        o.data = q.data;
        if let Some((p, q)) = i.data {
            d.data = q;
            o.done = p;
        }
        if cr.reset.any() {
            o.done = false;
        }
        (o, d)
    }
}

mod master {
    use rhdl::prelude::*;

    #[derive(Clone, Debug, Synchronous, Default)]
    pub struct U {}

    #[derive(PartialEq, Clone, Digital)]
    pub struct I {
        pub write: Option<(bool, b2)>,
        pub done: bool,
    }

    #[derive(PartialEq, Clone, Digital)]
    pub struct O {
        pub ready: bool,
        pub data: Option<(b2, bool)>,
    }

    impl SynchronousDQ for U {
        type D = ();
        type Q = ();
    }

    impl SynchronousIO for U {
        type I = I;
        type O = O;
        type Kernel = master_kernel;
    }

    #[kernel]
    pub fn master_kernel(cr: ClockReset, i: I, _q: ()) -> (O, ()) {
        let mut o = O::dont_care();
        o.ready = true;
        o.data = None;
        if let Some((addr, data)) = i.write {
            o.data = Some((data, addr));
        }
        if cr.reset.any() {
            o.data = None;
            o.ready = false;
        }
        (o, ())
    }
}

#[derive(Synchronous, SynchronousDQ, Clone, Debug, Default)]
struct U {
    sub: sub::U,
    master: master::U,
}

impl SynchronousIO for U {
    type I = Option<(bool, b2)>;
    type O = b2;
    type Kernel = kernel;
}

#[kernel]
pub fn kernel(cr: ClockReset, i: Option<(bool, b2)>, q: Q) -> (b2, D) {
    let mut d = D::dont_care();
    d.sub.data = None;
    if let Some((f, a)) = q.master.data {
        d.sub.data = Some((a, f));
    }
    d.sub.ready = q.master.ready;
    d.master.done = q.sub.done;
    d.master.write = i;
    let mut o = q.sub.data;
    if cr.reset.any() {
        o = bits(0)
    }
    (o, d)
}

fn test_input_stream() -> impl Iterator<Item = TimedSample<(ClockReset, Option<(bool, b2)>)>> {
    vec![
        None,
        Some((false, bits(1))),
        Some((true, bits(2))),
        None,
        Some((false, bits(3))),
        Some((true, bits(1))),
        None,
        None,
        None,
    ]
    .into_iter()
    .with_reset(1)
    .clock_pos_edge(100)
}

#[test]
fn test_trace() -> miette::Result<()> {
    let uut = U::default();
    let input = test_input_stream();
    let vcd = uut.run(input).collect::<Vcd>();
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("vcd")
        .join("lid");
    std::fs::create_dir_all(&root).unwrap();
    let expect = expect!["5092aeca697d257a93249ad48b5b8fe84f92ac4f168c50bc074241e0cfd0b006"];
    let digest = vcd.dump_to_file(root.join("twist.vcd")).unwrap();
    expect.assert_eq(&digest);
    Ok(())
}

#[test]
fn test_hdl_generation() -> miette::Result<()> {
    let uut = U::default();
    let input = test_input_stream();
    let tb = uut.run(input).collect::<SynchronousTestBench<_, _>>();
    let tm = tb.rtl(&uut, &Default::default())?;
    tm.run_iverilog()?;
    let tm = tb.ntl(&uut, &Default::default())?;
    tm.run_iverilog()?;
    Ok(())
}
