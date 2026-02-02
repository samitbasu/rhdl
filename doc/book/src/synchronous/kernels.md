# Kernels

In the context of the `Synchronous` trait, the compute kernel is specified in the `SynchronousIO` trait.  Recall the `SynchronousIO` trait

```rust
{{#rustdoc_include ../code/src/synchronous.rs:synchronous-io}}
```

The line of interest is this one:

```rust
{{#rustdoc_include ../code/src/synchronous.rs:kernel-def}}
```

which in words says that the `Kernel` type satisfies the following constraints

- `Kernel : DigitalFn` - this means that that `Kernel` is a synthesizable function that has been decorated with the `#[kernel]` attribute.
- `Kernel` is the type of a function that has the signature `fn(ClockReset, I, Q) -> (O, D)`.


An example of a synchronous kernel might be helpful.  This is the kernel for a simple counter with a variable (compile time determined) bit width:

```rust
{{#rustdoc_include ../code/src/synchronous.rs:counter-kernel}}
```

The `counter` kernel function uses the `ClockReset` argument to force the count to zero.  

Note the important things:

- The signature of the kernel is `fn(ClockReset, I, Q) -> (O, D)`.
- The `#[kernel]` attribute is attached to the definition.
