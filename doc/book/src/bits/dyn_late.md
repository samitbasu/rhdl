# Late Checking

Dynamic operators are checked late in the compilation process, rather than early.  

```shell,rhdl-silent
rm -rf late_check
rm -rf $ROOT_DIR/src/prj/late_check/target
cp -R $ROOT_DIR/src/prj/late_check .
cd late_check
cargo build -q 
```

Consider the following kernel function (I know we haven't covered kernel functions yet, but basically, they are pure Rust functions that are synthesizable...):

```rust
#[kernel]
fn do_stuff(a1: Signal<b4, Red>, a2: Signal<b4, Red>) -> Signal<b5, Red> {
    let a1 = a1.val().dyn_bits(); // 4 bits
    let a2 = a2.val().dyn_bits(); // 4 bits
    let c = a1.xadd(a2); // 5 bits
    let d = c.xadd(b1(1)); // 6 bits
    let e: b5 = d.as_bits(); // Uh oh!
    signal(e)
}
```

This function `do_stuff` will _compile_ just fine, since `rustc` does not know that `d` is a 6 bit quantity.  However, it is not runnable, and RHDL will reject it.  If we try to evaluate the function for some argument like this:

```rust
#[test]
fn test_run_do_stuff() {
    let y = do_stuff(signal(b4(3)), signal(b4(5))).val();
    assert_eq!(y, b5(9));
}
```

We get a straight panic:

```shell,rhdl:late_check
cargo nextest run test_run_do_stuff
```

In this case, RHDL's compiler is actually better at spotting exactly where the problem is.  We don't normally need to compile functions manually, but it's simple enough to do in a test case:

```rust
#[test]
fn test_compile_do_stuff() -> miette::Result<()> {
    let _ = compile_design::<do_stuff>(CompilationMode::Asynchronous)?;
    Ok(())
}
```

```shell,rhdl:late_check
cargo test test_compile_do_stuff
```

In this case, RHDL has inferred that `d` must be 5 bits wide (based on the conversion to `e`), and thus, the assignment is invalid.
