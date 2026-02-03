# Hardware Testing

We need to fixture our ones counter so that it looks like this:

```badascii
             +-----+Fixture+----+           
 dip0..dip7  |  ++OnesCounter++ | led0..led3
+------------+->|             +-+---->      
             |  +-------------+ |           
             +------------------+           
```

We will then use the constraints file to bind the `dip` switches and `led` pins to the FPGA.  Again, we use the `bind!` macro to connect top level named ports to the inputs and outputs of our circuit:

```rust
{{#rustdoc_include ../code/src/count_ones.rs:ones-step-10}}
```

Let's now create an integration test to build and flash the FPGA

```rust
{{#rustdoc_include ../code/src/count_ones.rs:ones-step-11}}
```

If you plug in your board, and run the test, you should have a functioning ones counter!

