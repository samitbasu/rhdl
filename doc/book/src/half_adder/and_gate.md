# The And Gate

To make a half-adder, we will need an AND gate as well as our XOR gate.  For completeness, here is our XOR gate from the [previous chapter](../xor_gate/the_gate.md):


```rust
{{#rustdoc_include ../code/src/half_adder.rs:adder-step-1}}
```

And here is the equivalent for an And gate.  I don't recommend building actual designs this way - it is very very low level, but it is illustrative.

```rust
{{#rustdoc_include ../code/src/half_adder.rs:adder-step-2}}
```

Given that our AND gate is trivial by construction, we won't bother testing it, but you may want to repeat the exercises from the `Xor Gate` chapter yourself.  Let's move on to building our half-adder.
