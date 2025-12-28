use expect_test::expect;
use rhdl::{core::circuit::chain::Chain, prelude::*};

mod auto_counter {
    use rhdl::prelude::*;
    use rhdl_fpga::core::dff;

    #[derive(Debug, Clone, Default, SynchronousDQ, Synchronous)]
    pub struct U<const N: usize>
    where
        rhdl::bits::W<N>: BitWidth,
    {
        count: dff::DFF<Bits<N>>,
    }

    impl<const N: usize> SynchronousIO for U<N>
    where
        rhdl::bits::W<N>: BitWidth,
    {
        type I = ();
        type O = Bits<N>;
        type Kernel = auto_counter_kernel<N>;
    }

    #[kernel]
    pub fn auto_counter_kernel<const N: usize>(_cr: ClockReset, _i: (), q: Q<N>) -> (Bits<N>, D<N>)
    where
        rhdl::bits::W<N>: BitWidth,
    {
        (q.count, D::<N> { count: q.count + 1 })
    }
}

mod doubler {
    use rhdl::prelude::*;

    #[kernel]
    pub fn doubler<const N: usize>(_cr: ClockReset, i: Bits<N>) -> Bits<N>
    where
        rhdl::bits::W<N>: BitWidth,
    {
        i << 1
    }
}

#[test]
fn test_auto_counter_counts() -> miette::Result<()> {
    let input = std::iter::repeat_n((), 100)
        .with_reset(1)
        .clock_pos_edge(100);
    let uut = auto_counter::U::<4>::default();
    let vcd = uut.run(input).collect::<Vcd>();
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("vcd")
        .join("chain_func");
    std::fs::create_dir_all(&root).unwrap();
    let expect = expect!["a00439c6689e90a5fe2f8ec7812ed70dcac787261d99e747d4bfc2d80d7aa1a5"];
    let digest = vcd.dump_to_file(root.join("auto_counter.vcd")).unwrap();
    expect.assert_eq(&digest);
    Ok(())
}

#[test]
fn test_auto_counter_is_correct() -> miette::Result<()> {
    let input = std::iter::repeat_n((), 100)
        .with_reset(1)
        .clock_pos_edge(100);
    let uut = auto_counter::U::<4>::default();
    let output = uut
        .run(input)
        .synchronous_sample()
        .map(|x| x.output)
        .skip(1)
        .collect::<Vec<_>>();
    let expected = (0..100).map(|x| bits(x % 16)).collect::<Vec<_>>();
    assert_eq!(output, expected);
    Ok(())
}

#[test]
fn test_chain_auto_counter() -> miette::Result<()> {
    let input = std::iter::repeat_n((), 100)
        .with_reset(1)
        .clock_pos_edge(100);
    let c1 = auto_counter::U::<4>::default();
    let c2 = Func::try_new::<doubler::doubler<4>>()?;
    let uut = Chain::new(c1, c2);
    let output = uut
        .run(input)
        .synchronous_sample()
        .map(|x| x.output)
        .skip(1)
        .collect::<Vec<_>>();
    let expected = (0..100)
        .map(|x| bits(((x % 16) << 1) % 16))
        .collect::<Vec<_>>();
    assert_eq!(output, expected);
    Ok(())
}
