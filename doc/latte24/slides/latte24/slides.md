---
# try also 'default' to start simple
theme: default
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

- **Static typing** - capture errors at the compilation stage instead of on the bench
- **Functional** - use functional programming features to improve testing and express design intent
- **Macros** - extend and reuse designs safely
- **Cargo** - included package manager to ease reuse and modularity
- **Crates.io** - established open-source ecosystem to share code and designs
- **Testing** - built-in testing framework makes automated testing painless
- **Tooling** - IDE support from `rust-analyzer` provides code completion, 
type hints, etc.
- **Generics** - reusable code with type safety

<br>
<br>

Official website [www.rust-hdl.org](https://www.rust-hdl.org/)

---
layout: two-cols
---

# RustHDL (a first attempt)

- Entirely open source core and libraries
- Takes a subset of Rust and transpiles it to Verilog
- Includes an event-based simulator
- Uses AST transformations to generate HDL
- Medium scale designs developed, tested, and deployed
- Commercial use in a high speed data acquisition system
- Moving from Xilinx --> Lattice ECP5 took ~2 weeks
  - Including time to build a soft-core SDRAM controller
  - And a new PC interface for data transfer

::right::

```rust
#[derive(LogicBlock)]
pub struct Blinky {
  pulser: Pulser,
  clock: Signal<In, Clock>,
  leds: Signal<Out, Bits<8>>,
}

impl Logic for Blinky {
  #[hdl_gen]
  fn update(&mut self) {
    self.pulser.enable.next = true;
    self.pulser.clock.next = self.clock.val();
    self.leds.next = 0x00.into();
    if self.pulser.pulse.val() {
      self.leds.next = 0xAA.into();
    }
  }
}

impl Default for Blinky {
  fn default() -> Self {
    let pulser = Pulser::new(CLOCK_SPEED_100MHZ.into(), 1.0, Duration::from_millis(250));
    Blinky {
      pulser,
      clock: pins::clock(),
      leds: pins::leds(),
    }
  }
}

fn main() {
    let uut = Blinky::default();
    synth::generate_bitstream(uut, "firmware/blinky")
    // v--- build a simple simulation (1 testbench, single clock)
    let mut sim = simple_sim!(Blinky, clock, CLOCK_SPEED_HZ, ep, {
        let mut x = ep.init()?;
        wait_clock_cycles!(ep, clock, x, 4*CLOCK_SPEED_HZ);
        ep.done(x)
    });
    // v--- construct the circuit
    let uut = Blinky::default();
    // v--- run the simulation, with the output traced to a .vcd file
    sim.run_to_file(Box::new(uut), 5 * sim_time::ONE_SEC, "blinky.vcd").unwrap();
}
```

---
layout: two-cols
---

# Composition for Hardware


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
---

# Blinky!


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
- Scopes and visibility are Rust-like


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
---

# Simulation And Testing

- RustHDL includes an event-based simulator built-in
- Combinatorial intra-module logic and multi-clock domains supported

![SDRAM_FIFO](/hls_sdram_fifo.png)


---
layout: two-cols
---

# Reuse

- Rust is highly composable
- Sub-modules can be tested in isolation
- FPGA specific code split out into `board support packages`
- Manages toolchain and constraint generation
- Third party BSPs do exist on `crates.io`!
- Reuse via structural composition of modules
- Internal logic is encapsulated and hidden
- Very similar to how schematics are built

::right::

```rust
#[derive(LogicBlock)]
pub struct SDRAMFIFOController<
    const R: usize, // Number of rows in the SDRAM
    const C: usize, // Number of columns in the SDRAM
    const L: u32,   // Line size (multiple of the SDRAM interface width) - rem(2^C, L) = 0
    const D: usize, // Number of bits in the SDRAM interface width
    const A: usize, // Number of address bits in the SDRAM (should be C + R + B)
> {
    // FPGA interface Clock
    pub clock: Signal<In, Clock>,
    // SDRAM interface - 10 signals
    pub sdram: SDRAMDriver<D>,
    // Clock for SDRAM
    pub ram_clock: Signal<In, Clock>,
    // FIFO interface
    pub data_in: Signal<In, Bits<D>>,
    pub write: Signal<In, Bit>,
    pub full: Signal<Out, Bit>,
    // ... snip ...
    // SDRAM Bursty controller
    controller: SDRAMBurstController<R, C, L, D>,
    // Front and back porch FIFOs
    fp: AsynchronousFIFO<Bits<D>, 5, 6, L>,
    bp: AsynchronousFIFO<Bits<D>, 5, 6, L>,
    // Read pointer for SDRAM
    read_pointer: DFF<Bits<A>>,
}
```

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


