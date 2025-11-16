# Inference

In practice, you will find that `Timed` is really critical in terms of the _interfaces_ of your circuit.   In terms of the _implementation_ of your design, you rarely have to worry about the timing domains except in the very specific places where signals from the two timing domains interact. 

The reason for this is that you can type erase the timing information from your signals within the scope of a function, and then let RHDL's type inference ensure correctness of the signal manipulation within.   For example, consider the following simple kernel that takes two 8-bit values in the `Red` domain, and returns their sum back to the `Red` domain:

```rust
#[kernel]
pub fn add(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<b8, Red> {
    let a = a.val(); // type erasure, result is a plain `b8`
    let b = b.val(); // type erasure, result is a plain `b8`
    let c = a + b;  // b8 + b8 -> b8
    signal(c) // type inference of signal::<Red>(c) from return 
}
```

In this very simple example, the inference is not that complicated.  But in more complicated cases, the use of type inference on the timing domain can be extremely helpful.  

```admonish warning
Once you erase the timing information from the type, `rustc` will no longer be able to determine if you inappropriately mix signals of different time domains.  RHDL will still throw an error if you attempt to use the function/design, but the error is no longer a `rustc` generated compile-time error.  Furthermore, if you test the function as a plain Rust function, it will not panic, as there is no time domain checking in the Rust function.
```

An example is helpful at this point.


```shell,rhdl-silent
rm -rf time_check
rm -rf $ROOT_DIR/src/prj/time_check/target
cp -R $ROOT_DIR/src/prj/time_check .
cd time_check
cargo build -q 
```

Consider the following basic kernel function that violates our invariant that signals from different time domains cannot be mixed:

```rust
#[kernel]
fn do_stuff(a1: Signal<b4, Red>, a2: Signal<b4, Blue>) -> Signal<b4, Red> {
    let a1 = a1.val();
    let a2 = a2.val();
    let c = a1 + a2; // Ack! Red + Blue is not defined!
    signal(c)
}
```

This function will compile and run just fine, since once the type has been erased, there is no runtime checking of the time domains.  

```rust
#[test]
fn test_run_do_stuff() {
    let y = do_stuff(signal(b4(3)), signal(b4(5))).val();
    assert_eq!(y, b4(8));
}
```

```shell,rhdl:time_check
cargo nextest run test_run_do_stuff 
```

If you attempt to compile this design into a working circuit, however, the RHDL compiler will flag the violation

```rust
#[test]
fn test_compile_do_stuff() -> miette::Result<()> {
    env_logger::init();
    let _ = compile_design::<do_stuff>(CompilationMode::Asynchronous)?;
    Ok(())
}
```

```shell,rhdl:time_check
cargo test test_compile_do_stuff 
```




