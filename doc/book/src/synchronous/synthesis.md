# Synthesis

Synthesis for `Synchronous` circuits is also completely analogous to that for `Circuit` and was covered [here](../circuits/synthesis.md).  In summary, we call `descriptor` on an instance of the circuit, with a scoped name, and collect the resulting struct.  The relevant part of the `Synchronous` trait is:

```rust
pub trait Synchronous: 'static + Sized + SynchronousIO {
    // snip
    fn descriptor(&self, scoped_name: ScopedName) -> Result<Descriptor<SyncKind>, RHDLError>;
    // snip
}
```

If we are at the top of the hierarchy, we can use `ScopedName::top` to generate a hierarchy root (the resulting modules will have `top` as the root module).  Constructing the descriptor can fail at run time as `Synchronous` blocks may not be synthesizable.  This can occur when building circuits for testing that include behaviors that are useful for analysis, but cannot be physically synthesized.  

Given a `Descriptor<SyncKind>`, we can retrieve the HDL (if it is defined) from the `.hdl` field:

```rust
pub struct Descriptor<T> {
    // snip
    pub hdl: Option<HDLDescriptor>,
    // snip
}
```

If this field is `Some`, then the underlying `HDLDescriptor` contains a synthesizable translation of the RHDL design into Verilog.  The `HDLDescriptor` is also quite simple:

```rust
#[derive(Clone, Hash, Debug)]
pub struct HDLDescriptor {
    /// The unique name of the circuit.
    pub name: String,
    /// The list of modules that make up this circuit.
    pub modules: rhdl_vlog::ModuleList,
}
```

You can think of `name` as the "top" element of the circuit.  The `modules` are data structures that define the Verilog code used to describe the circuit.  You can convert these into a pretty printed string using the `.pretty()` method.  The `ModuleList` struct also has a method to check the syntax of the enclosed Verilog using [icarus](https://github.com/steveicarus/iverilog).  So, for a synthesizable circuit `T`, we can do something akin to:

```rust
let uut = T::new(); // 👈 or whatever
let desc = uut.descriptor()?; // Get the run time descriptor
let hdl = desc.hdl()?; // Gets a reference to the checked HDL descriptor
std::fs::write("my_verilog.v", hdl.modules.pretty())?; // Do something with it
```

This code is identical to the one presented for `Circuit` [here](../circuits/synthesis.md).
