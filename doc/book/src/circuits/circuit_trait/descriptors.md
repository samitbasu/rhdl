# Descriptors

Arguably the most important method in the `Circuit` trait is the one that computes a `Descriptor` for the circuit:

```rust
fn descriptor(&self, scoped_name: ScopedName) -> Result<Descriptor<AsyncKind>, RHDLError>;
```

It is instructive to look at the definition of the `Descriptor` type itself:

```rust
/// Run time description of a circuit.
#[derive(Debug)]
pub struct Descriptor<T> {
    /// The scoped name of the circuit.
    pub name: ScopedName,
    /// The kind of the input type.
    pub input_kind: Kind,
    /// The kind of the output type.
    pub output_kind: Kind,
    /// The kind of the internal feedback type to the inputs of the children.
    pub d_kind: Kind,
    /// The kind of the internal feedback type from the outputs of the children.
    pub q_kind: Kind,
    /// The compiled kernel object.
    pub kernel: Option<rtl::Object>,
    /// The netlist representation of the circuit, if available.
    pub netlist: Option<ntl::Object>,
    /// The HDL (Verilog) description of the circuit, if available.
    pub hdl: Option<HDLDescriptor>,
    /// Phantom data for the marker type.
    pub _phantom: PhantomData<T>,
}
```

There is a lot here, so let's break it down.  First, the idea of a `ScopedName` is equivalent to a `Path` in Rust.  

```rust
    /// The scoped name of the circuit.
    pub name: ScopedName,
```

It is best to think of it as something like `top::frob::baz::widget`, where each segment of the path represents a different scope, and scopes are isolated from each other.  Normally, HDLs used for synthesis are un-scoped (everything sits at a single global namespace), and so RHDL emulates scoping rules by constructing `ScopedName`s for each element, and then later converting these into globally unique names that can sit in the single global namespace.  Thus, every flip flop in a RHDL design will occupy a module named `{path}_dff`, where `{path}` is unique for each DFF.  

Next, we have the `input_kind, output_kind, etc`.  

```rust
    /// The kind of the input type.
    pub input_kind: Kind,
    /// The kind of the output type.
    pub output_kind: Kind,
    /// The kind of the internal feedback type to the inputs of the children.
    pub d_kind: Kind,
    /// The kind of the internal feedback type from the outputs of the children.
    pub q_kind: Kind,
```

These are run time descriptors of the types that are associated with the `CircuitIO` and `CircuitDQ` traits.  Basically, `Kind` represents a run time description of a type that can be manipulated at run time, and for which the layout may be completely different than how `rustc` is representing a type.  We cover `Kind` and the RHDL layout mechanism in detail in the [Digital Types](../../digital/summary.md) chapter.  These can be computed as:

```rust
Descriptor {
    // snip
    input_kind: <<Self as CircuitIO>::I as Digital>::static_kind(),
    output_kind: <<Self as CircuitIO>::O as Digital>::static_kind(),
    d_kind: <<Self as CircuitDQ>::D as Digital>::static_kind(),
    q_kind: <<Self as CircuitDQ>::Q as Digital>::static_kind(),
    // snip
}
```

Next is the kernel function represented as a RHDL Register Transfer Level (RTL) representation:

```rust
    pub kernel: Option<rtl::Object>,
```

Note that this is an `Option`, to allow for the case of circuits that cannot be described by `#[kernel]` functions.  If the function _is_ a `#[kernel]`, then the compiled object code for that function will be contained here.

To get a sneak peek at what this object code looks like (will cover it elsewhere), consider the following sample kernel function:

```rust
    #[kernel]
    fn kernel(x: [b4; 4]) -> b6 {
        let mut accum = b6(0);
        for i in 0..4 {
            accum += x[i].resize::<6>();
        }
        accum
    }
```
 
This simply sums 4 nibbles into a 6 bit accumulator.  It gets translated into the following RTL (at least at the time I wrote this):

```rust
Object kernel
  fn_id FnID(acf150c5f91134b1)
  arguments [Some(or1)]
  return_register r12
  reg r0 : b4 //  b4
  reg r1 : b16 //  [b4; 4]
  reg r2 : b6 //  b6
  reg r3 : b6 //  b6
  reg r4 : b4 //  b4
  reg r5 : b6 //  b6
  reg r6 : b6 //  b6
  reg r7 : b4 //  b4
  reg r8 : b6 //  b6
  reg r9 : b6 //  b6
  reg r10 : b4 //  b4
  reg r11 : b6 //  b6
  reg r12 : b6 //  b6
  lit l0 : b000000 // b6
  r0 <- r1[0..4] // [[0]]
  r2 <- r0 as x6
  r3 <- l0 + r2
  r4 <- r1[4..8] // [[1]]
  r5 <- r4 as x6
  r6 <- r3 + r5
  r7 <- r1[8..12] // [[2]]
  r8 <- r7 as x6
  r9 <- r6 + r8
  r10 <- r1[12..16] // [[3]]
  r11 <- r10 as x6
  r12 <- r9 + r11
Done
```

It's a bit verbose, but you should be able to follow the logic.  The registers `r0, r4, r7, r10` are used to extract nibbles out of the array input.  These are widened into `r2, r5, r8, r11` respectively to make them 6 bits wide.  Finally, `r3, r6, r9, r12` form an adder chain, with `l0` providing the zero initialization.  

```admonish note
If you notice the missing optimization opportunity here (the register `r3` uses an adder that can be removed), it is a reflection of the fact that the RHDL compiler is still improving.  Once things get stabilized, I can add additional optimizations.  And in any case the output is typically fed to another compiler which will optimize much of these away.
```

The netlist representation of the circuit is also provided by the descriptor.  In most cases, this will be automatically built for a `Circuit` derived using the macro.  We can see the netlist representation for our chained adder as well.  It's even more verbose, since at the netlist level, we work with individual bits

```rust
BTL kernel
   arguments [{r4,r5,r6,r7,r8,r9,r10,r11,r12,r13,r14,r15,r16,r17,r18,r19}]
   return {r74,r75,r76,r77,r78,r79}
    {r26,r27,r28,r29,r30,r31} <- {l0,l0,l0,l0,l0,l0} Add {r4,r5,r6,r7,l0,l0}
    {r42,r43,r44,r45,r46,r47} <- {r26,r27,r28,r29,r30,r31} Add {r8,r9,r10,r11,l0,l0}
    {r58,r59,r60,r61,r62,r63} <- {r42,r43,r44,r45,r46,r47} Add {r12,r13,r14,r15,l0,l0}
    {r74,r75,r76,r77,r78,r79} <- {r58,r59,r60,r61,r62,r63} Add {r16,r17,r18,r19,l0,l0}
Done
```

Again, I realize there are opportunities to simplify this and optimize it, but the current focus is on correctness as opposed to compactness.  The netlist generated by RHDL is primarily used for analysis, and not for simulation or for synthesis.  But that could change in the future.

The last piece of the descriptor is the HDL (Verilog) output.  Because RHDL needs to interface with existing toolchains to produce actual hardware designs, it is important to be able to convert a RHDL design into an existing format.  This function is provided by the HDL method, and any custom HDL work is provided by implementing this method.  The default implementation builds a Verilog function to compute the kernel.  Again, using the example from above, we get

```rust
function [5:0] kernel_kernel(input reg [15:0] arg_0);
      reg [3:0] r0;
      reg [15:0] r1;
      reg [5:0] r2;
      reg [5:0] r3;
      reg [3:0] r4;
      reg [5:0] r5;
      reg [5:0] r6;
      reg [3:0] r7;
      reg [5:0] r8;
      reg [5:0] r9;
      reg [3:0] r10;
      reg [5:0] r11;
      reg [5:0] r12;
      localparam l0 = 6'b000000;
      begin
         r1 = arg_0;
         r0 = r1[3:0];
         r2 = {{2{1'b0}}, r0};
         r3 = l0 + r2;
         r4 = r1[7:4];
         r5 = {{2{1'b0}}, r4};
         r6 = r3 + r5;
         r7 = r1[11:8];
         r8 = {{2{1'b0}}, r7};
         r9 = r6 + r8;
         r10 = r1[15:12];
         r11 = {{2{1'b0}}, r10};
         r12 = r9 + r11;
         kernel_kernel = r12;
      end
endfunction
```

The phantom data member is used to constrain the `Descriptor` type.  `Circuit` produces a `Descriptor<AsyncKind>`, while `Synchronous` produces a `Descriptor<SyncKind>`.  These marker types ensure that you cannot accidentally confuse or mix the two types at run time.
