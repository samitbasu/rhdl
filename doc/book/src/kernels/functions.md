# Functions

Ultimately, any circuit `kernel` must be a function, but you can also create other functions, annotate them with `kernel` and then call from from other kernels.  This form of composition allows you to factor your code into reusable pieces that can be individually and rigorously tested and then easily composed.

In general functions used in `kernel` must satisfy the following:

1. They must be `pub`.  This requirement is due to how the generated types get exported.
2. They cannot return a zero-sized type.  I.e., a synthesizable function must produce _something_.  The argument list can be empty, but not the return type.
3. Each argument must `impl Digital`.  The return type must also `impl Digital`.
4. There is a limit to the number of arguments the function can take.  Currently about 6.

Let's work through some examples so you can get a sense of what kinds of function signatures are allowed, and which ones are not.

The first is a simple function with a couple of arguments, a couple of returns.  Nothing too exciting.

```rust
{{#rustdoc_include ../code/src/kernels/functions.rs:step_0}}
```

The following is _not_ allowed, because it returns a zero-sized type

```rust
{{#rustdoc_include ../code/src/kernels/functions.rs:step_1}}
```

This `kernel` does not cause a `rustc` compilation error, but it will fail when you attempt to use it in a RHDL design.  To see this, we can explicitly compile the kernel using the `compile_design` function:

```rust
{{#rustdoc_include ../code/src/kernels/functions.rs:step_1_test}}
```

This results in the following error:

<!-- cmdrun to-html "cd ../code && cargo test --lib -- kernels::functions::step_1::test_empty_return_kernel --exact --nocapture --ignored 2>&1" -->

```admonish note
RHDL does not allow functions with empty return types since a pure function with an empty return would not correspond to any hardware.  While I guess you could use such a function to log or trace messages, RHDL requires you to put those `trace` calls directly into a kernel that does something.
```

## Infallible Pattern Matching Arguments

You can use infallible pattern matching in the arguments for tuples.  Struct and enum patterns are currently unsupported.

```rust
{{#rustdoc_include ../code/src/kernels/functions.rs:step_2}}
```

You can make your function generic and add bounds as needed.

```rust
{{#rustdoc_include ../code/src/kernels/functions.rs:step_3}}
```

```admonish note
When calling a generic function from within a kernel, due to limitations of how the `#[kernel]` attribute works, you will need to **explicitly** supply all of the generic types.  Type inference across function calls is not great in RHDL at the moment.  This can be annoying, but it may improve in the future as the RHDL compiler improves.
```

Of course, you can pass `Digital` values into functions, and return `Digital` values out of the functions:

```rust
{{#rustdoc_include ../code/src/kernels/functions.rs:step_4}}
```

```admonish warning
You cannot use `impl Digital` in return position with `kernel` functions.  The result can be generic, but it must be declared as a generic parameter.
```

You can _call_ functions just like you normally would in rust, provided the named function is available at the call site, and that the referenced function is annotated with `#[kernel]`.  So, for example:

```rust
{{#rustdoc_include ../code/src/kernels/functions.rs:step_4}}
```

When the function you are calling is generic, you must provide the generic parameters explicitly at the call site.  This is due to a limitation on how the RHDL compiler works.  So if we had instead:

```rust
{{#rustdoc_include ../code/src/kernels/functions.rs:step_5}}
```

Getting type inference to work at these sites would involve some pretty substantial changes to the way the RHDL compiler works.  It's not impossible, so maybe in a future version.