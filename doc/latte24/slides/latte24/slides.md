---
# try also 'default' to start simple
theme: seriph
# random image from a curated Unsplash collection by Anthony
# like them? see https://unsplash.com/collections/94734566/slidev
background: https://cover.sli.dev
# some information about your slides, markdown enabled
title: Rust as a Hardware Description Language
info: |
  ## Slidev Starter Template
  Presentation slides for developers.

  Learn more at [Sli.dev](https://sli.dev)
# apply any unocss classes to the current slide
class: text-center
# https://sli.dev/custom/highlighters.html
highlighter: shiki
# https://sli.dev/guide/drawing
drawings:
  persist: false
# slide transition: https://sli.dev/guide/animations#slide-transitions
transition: slide-left
# enable MDC Syntax: https://sli.dev/guide/syntax#mdc-syntax
mdc: true
---

# Rust as a Hardware Description Language

Samit Basu

Latte 24 Conference

San Diego, CA 2024

<!--
The last comment block of each slide will be treated as slide notes. It will be visible and editable in Presenter Mode along with the slide. [Read more in the docs](https://sli.dev/guide/syntax.html#notes)
-->

---
---

# Why Rust?

- **Static typing** - capture errors at the compilation stage instead of on the bench.
- **Functional** - use functional programming features to improve testing and express design intent.
- **Macros** - extend and reuse designs safely
- **Cargo** - included package manager to ease reuse and modularity
- **Crates.io** - established open-source ecosystem to share code and designs
- **Testing** - built-in testing framework makes automated testing painless
- **Tooling** - IDE support from `rust-analyzer` provides code completion, 
type hints, etc.
- **Generics** - reusable code with type safety

<br>
<br>

Official [website](https://www.rust-hdl.org/)

---
layout: two-cols
---

# RustHDL (first attempt)

- Entirely open source core and libraries
- Takes a subset of Rust and compiles it to Verilog
- Includes an event-based simulator
- No formal compiler - just a basic transpiler/code-rewriter
- Medium scale designs developed 
- Commercially deployed in a high speed data acquisition system
- Moving from Xilinx --> Lattice ECP5 took ~2 weeks
- Including time to build a soft-core SDRAM controller

::right::

```rust
#[derive(LogicBlock)]
// v--- Modules use simple composition of structs
pub struct SPIMaster<const N: usize> {
    // Clocks are a type ---v
    pub clock: Signal<In, Clock>,
    // Signal has direction --v 
    pub data_outbound: Signal<In, Bits<N>>,
    // Signal has type ---------v
    pub start_send: Signal<In, Bit>,
    //v--- pub visibility control
    pub data_inbound: Signal<Out, Bits<N>>,
    // Bus ------v
    pub wires: SPIWiresMaster,
    // Local scratchpad ---v
    local_signal: Signal<Local, Bit>,
    // D Flip Flop --v
    state: DFF<SPIState>,
    //           ^-- With enumerated value
    cs_off: Constant<Bit>,
    //           ^-- (Rust) Run time initialized constant
}
```

---
layout: two-cols
---

# Interfaces

- Logical grouping of signals
- Can be nested
- Allow signals in both directions
- Think "wiring harness"


::right::

```rust 
//        v-- indicates its an interface
#[derive(LogicInterface, Clone, Debug, Default)]
//        v-- "mating" interface
#[join = "SDRAMDriver"]
pub struct SDRAMDevice<const D: usize> {
  // Interfaces can be generic --^
  pub clk: Signal<In, Clock>,
  pub we_not: Signal<In, Bit>,
  pub read_data: Signal<Out, Bits<D>>,
  pub write_enable: Signal<In, Bit>,
}
```

Can `join` interfaces with a single line of code:
```rust
fn update(&mut self) {
    I2CBusDriver::join(&mut self.controller.i2c, 
      &mut self.test_bus.endpoints[0]);
}
```

---
layout: two-cols
---

# Mental Model

- Signals
  - Represent wires in the design
  - Read from `.val()`
  - Write to `.next` member
- State encapsulated in D Flip Flops, BRAMs, etc.
- **Code is valid Rust** 
  - removing the `#[hdl_gen]` attribute does 
  not change the code's meaning
  - Simulations run the Rust directly
  - You must make `rustc` happy!
- Latch prevention done via `yosys` pass




::right::

```rust
// Design is parametric over N - the size of the counter
impl<const N: usize> Logic for Strobe<N> {
  // v-- Attribute to generate HDL
  #[hdl_gen]
  // v-- Update function is attached to any logic circuit
  fn update(&mut self) {
    // v-- latch prevention
    self.counter.d.next = self.counter.q.val();
    // v-- mux control signal
    if self.enable.val() {
      //  v-- value assigned if mux control is true
      self.counter.d.next = self.counter.q.val() + 1;
    }
    // v-- combinatorial logic
    self.strobe.next = self.enable.val() & 
      (self.counter.q.val() == self.threshold.val());
    // v-- higher priority mux for previous mux output
    if self.strobe.val() {
      self.counter.d.next = 1.into();
    }
  }
}
```


---
---

# Simulation And Testing

- RustHDL includes an event-based simulator built-in
- Combinatorial intra-module logic and multi-clock domains supported

![SDRAM_FIFO](/hls_sdram_fifo.png)


---
---

# Reuse

- Rust is highly composable
- Sub-modules can be tested in isolation
- FPGA specific code split out into `board support packages`
- Manages toolchain and constraint generation
- Third party BSPs do exist on `crates.io`!

---
---

# Shortcomings and Feedback - Not Rusty Enough

Early user feedback on RustHDL

- Need more language features to make it feel more `Rusty`
  - Local variables
  - Type inference
  - Match/if expressions
  - Early returns

```rust
fn update() {
  //  v--- local variable with inferred type
  let a = match self.state {
    //     ^--- match expression
    State::Idle => return(3);
    // Early returns ---^
    State::Busy => 1,
  };
}
```

---
layout: two-cols
---

# Wish list
Function and data composition

- Rich enums, structs, arrays, etc.

```rust
enum OpCode {
  Noop,
  Jump(b24),
  Load{dest: Register, src: Register},
  Save([b56; 8])
}
```

- Function composition

```rust
fn add_em(a: u8, b: u8) -> u8 {
  a + b
}

fn do_stuff(a: u8, b: u8) -> u8 {
  add_em(a, b)
}
```

::right::

- Fewer footguns
  - Testbenches are too hard to write
  - Prevent timing collisions
  - Better error messages

- More backends
  - Support for VHDL, FIRRTL, etc.

---
---

# RHDL - The Next Generation

- Under development since 2023
- Includes a co-compiler
- Compiler includes
  - Type inference
  - Type checking
  - SSA transformation
  - Lowering passes for ifs, loops, etc.
  - Intermediate representation form
- VM to run the IR
- Generation of Verilog (other languages to be added)
- Automated detection of timing collisions, etc.
- Much more `Rusty`!


