## Basic Testing

We probably want to test all possible inputs of our `XorGate`, and since there are only four inputs, it shouldn't be too hard.  We can start by testing our kernel itself.  Just as a plain Rust function (which it still is...)

```shell,rhdl:xor
mkdir tests
```

```rust,write:xor/tests/test_inputs.rs
use rhdl::prelude::*;
use xor::*;

#[test]
fn test_all_inputs() {
    let inputs = [(false, false), (false, true), (true, false), (true, true)];
    let outputs = [false, true, true, false];
    inputs.iter().zip(outputs.iter()).for_each(|(inp, outp)| {
        let (y, _) = xor_gate(signal(*inp), ());
        assert_eq!(y.val(), *outp);
    });
}
```

```shell,rhdl:xor
cargo build -q
cargo test 
```

Ok - that was easy enough.  But that just tests that our logic was correct, right?  What about testing more of the things?  How do I know the generated hardware will work as intended?  And what does the generated hardware look like, anyway?  The simpleset way to get a view on the generated HDL is to use the `.hdl` method on any struct that `impl Circuit`.  The result can be converted into a module and then a string.   The following test does exactly that.

```rust,write:xor/tests/show_verilog.rs
use rhdl::prelude::*;

#[test]
fn show_verilog() -> miette::Result<()> {
     let gate = xor::XorGate;
     let hdl = gate.hdl("xor_gate")?.as_module();
     eprintln!("{hdl}");
     Ok(())
}
```

```shell,rhdl:xor
cargo build -q
cargo test --test show_verilog -- --nocapture
```

While not required, it is often handy to check that the output of an HDL generation step has not changed from the last time you reviewed or tested it.  As such, a crate like [expect-test](https://github.com/rust-analyzer/expect-test) can be used to check that the output is still correct.  We can add it as a `dev` dependency to our project

```shell,rhdl:xor
cargo add --dev expect-test
```

A test using `expect-test` can write the expected Verilog code to a file and, then verify it later.

```rust,write:xor/tests/expect_verilog.rs
use rhdl::prelude::*;

#[test]
fn test_verilog_output() -> miette::Result<()> {
     let gate = xor::XorGate;
     let hdl = gate.hdl("xor_gate")?.as_module();
     let expect = expect_test::expect_file!["xor.v.expect"];
     expect.assert_eq(&hdl.to_string());
     Ok(())
}
```

You can run the test with an `UPDATE_EXPECT=1` to get the expected output to be written to a file.
```shell,rhdl:xor
cargo build -q
UPDATE_EXPECT=1 cargo test --test expect_verilog
cat tests/xor.v.expect
```

Then in the future, you can run the test, and it will compare the generated code against the template file stored.

```shell,rhdl:xor
cargo test --test expect_verilog
```
