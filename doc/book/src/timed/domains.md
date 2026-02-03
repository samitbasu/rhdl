# Domains

The `Signal` wrapper type has two generic arguments.  The entirety of the definition is:

```rust
{{#rustdoc_include ../code/src/timed/trait_def.rs:signal_def}}
```

The `Domain` trait is also simple:

```rust
{{#rustdoc_include ../code/src/timed/trait_def.rs:domain_trait}}
```

where `Color` is an enum that uses a color to indicate the uniqueness of the time domain:

```rust
{{#rustdoc_include ../code/src/timed/trait_def.rs:color_enum}}
```

```admonish note
The `Domain` is much like a lifetime in regular Rust - it indicates the validity of the signal and defines a context to which it belongs.  Unfortunately, reusing the lifetime annotation for time domain is not possible given that we want RHDL code to still be valid Rust.
```

When designing reusable components in RHDL, it is often the case that you will need to be generic over the domain (unless designing a synchronous component).  So a widget that may be reusable should probably be generic over `D: Domain`.  Otherwise, you risk building a `Circuit` that assumes its inputs and outputs are tied to the e.g., `Red` domain, and need to use it with `Green` inputs!  If on the other hand, you are working on a design you don't anticipate needing to reuse, I find it's better to just pick some convention for the color to avoid generic-bloat.  