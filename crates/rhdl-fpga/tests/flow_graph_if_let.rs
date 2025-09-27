use expect_test::expect;
use rhdl::prelude::*;
use rhdl_fpga::core::dff;

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U {
    reg: dff::DFF<b8>,
}

impl SynchronousIO for U {
    type I = Option<(bool, b8)>;
    type O = bool;
    type Kernel = test_kernel;
}

#[kernel]
pub fn test_kernel(_cr: ClockReset, i: Option<(bool, b8)>, q: Q) -> (bool, D) {
    let mut d = D::dont_care();
    d.reg = q.reg;
    let mut o = false;
    if let Some((_x, y)) = i {
        d.reg = y;
        o = y.any();
    }
    (o, d)
}

fn stream() -> impl Iterator<Item = TimedSample<(ClockReset, Option<(bool, b8)>)>> {
    vec![
        None,
        Some((false, b8(3))),
        Some((true, b8(1))),
        None,
        Some((false, b8(0))),
        Some((true, b8(5))),
        None,
    ]
    .into_iter()
    .with_reset(1)
    .clock_pos_edge(100)
}

#[test]
fn test_trace() -> miette::Result<()> {
    let uut = U::default();
    let input = stream();
    let vcd = uut.run(input).collect::<Vcd>();
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("vcd")
        .join("flow_graph_if_let");
    std::fs::create_dir_all(&root).unwrap();
    let expect = expect!["4a38bb60748b7bf521c5f8e862822ec7f09f26244c7c34c6cdf59ff13bba1123"];
    let digest = vcd
        .dump_to_file(root.join("flow_graph_if_let.vcd"))
        .unwrap();
    expect.assert_eq(&digest);
    Ok(())
}

#[test]
fn test_hdl() -> miette::Result<()> {
    let uut = U::default();
    let input = stream();
    let tb = uut.run(input).collect::<SynchronousTestBench<_, _>>();
    let tm = tb.rtl(&uut, &Default::default())?;
    tm.run_iverilog()?;
    let tm = tb.ntl(&uut, &Default::default())?;
    tm.run_iverilog()?;
    Ok(())
}
