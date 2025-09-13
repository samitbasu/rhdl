use rhdl::prelude::*;

#[cfg(test)]
mod common;
#[cfg(test)]
use common::*;
use rhdl_core::circuit::{async_func::AsyncFunc, fixture::Fixture};
use rhdl_vlog::formatter::Pretty;

// Unified macro for binding inputs and outputs to a fixture

#[test]
fn test_simple_fixture_example() -> miette::Result<()> {
    let expect = expect_test::expect_file!["fixture_adder.expect"];
    #[kernel]
    fn adder(a: Signal<(b4, b4), Red>) -> Signal<b4, Red> {
        let (a, b) = a.val();
        signal(a + b) // Return signal with value
    }

    let adder = AsyncFunc::new::<adder>()?;
    let mut fixture = Fixture::new("adder_top", adder);
    bind!(fixture, a -> input.val().0);
    bind!(fixture, b -> input.val().1);
    bind!(fixture, sum -> output.val());
    let vlog = fixture.module()?;
    let vlog_str = vlog.pretty();
    expect.assert_eq(&vlog_str);
    Ok(())
}

#[test]
fn test_bind_macro_with_expressions() -> miette::Result<()> {
    #[kernel]
    fn simple_passthrough(a: Signal<b4, Red>) -> Signal<b4, Red> {
        a
    }

    let circuit = AsyncFunc::new::<simple_passthrough>()?;

    // Test with array indexing expression
    let mut fixtures = vec![
        Fixture::new("test0", circuit.clone()),
        Fixture::new("test1", circuit.clone()),
    ];

    // These should work now with expression instead of just identifier
    bind!(fixtures[0], input_signal -> input.val());
    bind!(fixtures[0], output_signal -> output.val());

    bind!(fixtures[1], input_signal -> input.val());
    bind!(fixtures[1], output_signal -> output.val());

    // Test with a new fixture to demonstrate method call expressions
    let mut another_fixture = Fixture::new("test2", circuit.clone());
    bind!(another_fixture, test_input -> input.val());
    bind!(another_fixture, test_output -> output.val());

    // Basic test - should not panic and should compile
    assert!(fixtures[0].module().is_ok());
    assert!(fixtures[1].module().is_ok());
    assert!(another_fixture.module().is_ok());

    Ok(())
}
