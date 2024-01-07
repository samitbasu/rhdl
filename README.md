# rhdl
A Hardware Description Language based on the Rust Programming Language

# Status

This turned out to be a complete rewrite of `rust-hdl` from the ground up.  The goal is
to satisfy four key capabilities, all of which were missing in `rust-hdl`:

- High performance simulation
- "It's just Rust" syntax
- Trivial reusability
- Support for enums with payloads

## High performance simulation

The high performance simulation issue is quite important.  As you build larger and
more complicated/complex designs, the simulation time becomes a limiting factor for
writing comprehensive test benches and running designs through thorough software 
validation.  To first order, RHDL is roughly 1 to 2 orders of magnitude faster than
RustHDL.

## "It's Just Rust" syntax

RustHDL was essentially a set of structured patterns to help you write hierarchical
strongly typed Verilog.  But it resulted in several strange "rules" that you needed
to remember such as
- signals need to be read from `.val()` and written to `.next`
- local signals are needed for all temporary variables/bindings
- connecting signals and data from multiple modules was fairly complicated

RHDL takes a completely different approach.  Here, the ideas are:
- The code is _just_ Rust.
- The supported subset of Rust is broad, and includes things like
  matches, if-expressions, let bindings, type inference and generics,
  early returns, etc.  References/pointers and lambdas are not supported.
- Data and state are handled in a transparent manner with larger state machines
encapsulating the state of smaller ones.  
- State advancement is handled at the top level, where things like clock crossings are 
also dealt with.

This design encourages a highly functional style of coding.  Testing is much more trivial,
since the individual functions can be tested using standard Rust test practice.

## Trivial reusability

The composability of the state and the functional nature of the state machine descriptions means
that reusing components (which are defined as a tuple of state, constants, types and a computer kernel)
is completely trivial.  

## Support for enums with payloads

The hardware world is a good one for using Rust enum's to model various data structures.  Packets,
opcodes, and data elements frequently make sense to be modelled as Rust enums.  RustHDL only supported
C style enums with no payloads.  RHDL, by contrast, supports and encourages the use of enums.

# Risks

There are several risks in the RHDL design that RustHDL either did not have, or sidestepped.
- The infrastructure for writing a DSL based on rustlang is nacent.  `rustc` itself is not 
targetted at reuse and repurposing.  `rust-analyzer` includes only the front end of a Rust
compiler, and not the middle parts.  
- RustHDL used procedural macros to try and convert the AST into something that could be
synthesized into Verilog.  Procedural macros have some severe limitations, including the
inability to share state between them and the lack of context for the processing.
- Time and effort - RustHDL was an out-growth of my commercially sponsored work at the time, and
there is fielded production quality firmware which was built with it.  Unfortunately, I no
longer get paid to write firmware.  So progress is slower.  I consider this more a schedule
risk than a technical one.

# The Plan

- [x] Import and improve the finite-width bits classes from `rust-hdl`
- [x] Add full fledged signed bit support
- [x] Create a high-performance logging infrastructure to log at speed
- [x] Add support for aggregate data types based on bits (e.g., structs, tuples, arrays)
- [x] Add support for enums that include bits
- [x] Allow for enums to have customized layouts for the discriminant
- [x] Provide a visualization tool for enums, and structs
- [x] Develop a hardware-compatible intermediate representation (RHIF)
- [x] Write a bootstrap compiler that can convert AST from Rust to RHIF
- [x] Write a type inference engine that can perform run time type inference
- [x] Provide support for calling other HDL kernels from within another one
- [x] Import undefined bindings from the calling scope
- [x] Move to a standard library of functions that can be synthesized and used
- [x] Build a test infrastructure to compare Verilog with Rust results
- [x] Write a compiler capable of handling multiple kernels
- [x] Write a RHIF -> Verilog assembler
- [ ] Port the `RustHDL` widget library to `RHDL`
- [ ] Port the various FPGA BSPs to `RustHDL` from `RHDL`.

Some other topics I'm thinking about

- [ ] Build a verilator bridge so that the Verilog code can be tested from Rust, using Verilator to simulate the Verilog
- [x] Add support for zero-sized signals.  This is needed for black box modules and synchronous automatons (no non-clock inputs)
- [-] Make logging pure.  The global approach feels "weird".
- [ ] Clean up the generated verilog, or clean up the RHIF so that the verilog is easier to read/understand.


# Logging - Rethought

So one challenge with the current logging infrastructure is that it was designed before I realized
that `rhdl` was going evolve the way it did.  So it needs a revisit.  If we lean more heavily towards
software logging and the `log` facade instead of hardware logging and the `vcd` concept, then we want
something as simple as:

```rust
vcd!("target", <value>)
```
where `value: Digital`.  Key ideas:

1. The "target" string can be combined with the module/file source location to get a unique 
  string that serves as a key for the log site.
2. The timestamp is held in the global logger state.
3. Thread local storage can be used in the future to allow for multiple loggers.
4. The `vcd` macro records a log of name/value pairs into a global db.  Delta compression is
applied somehow (which means it needs a way to compare values).
5. Later, the DB can be post-processed to generate VCD files.

We can also support the idea of logging/tracing levels.  Something like:

```rust
vcd!(level, "target", <value>)
```

The kernel proc-macro can just omit `vcd!` calls.



The core idea is still good.
The primary points of the new logging design are:

- Logging inside the "hot loop" needs to be super quick.
- Filters on the logging level can still be set on the log context.
- Context based logging - i.e., hierarchical names to identify log points

The base of storing data is to keep logs organized into streams of a given type.  Let's say we have a
function that serializes a Digital type into some Bytes.  I.e., assume that for any Digital data structure,
we can call

```rust
let y = x.log(); // <-- converts x into a log friendly representation
```

The idea is that we can then stash this byte slice into a bucket that is indexed by a simple integer.  So that
a log database looks something like (without worrying too much about performance just yet).

```rust
// New type to wrap the log index
struct LogIndex(usize);

// Log ID if allocated.  If not, it will have index: None.
struct LogID<T: Digital> {
  index: Option<LogIndex>,
  level: LogLevel,
  _kind: Phantom<T>,
}

// A log record, consisting of the serialized bytes and the timestamp.
struct LogRecord {
  data: Bytes,
  time: u64,
}

// For a log stream, the kind of the log data, 
struct LogDetails {
  name: String,
  kind: Kind,
}

// The log database is then just a list of data streams
struct LogDatabase {
  records: HashMap<LogIndex, LogRecord>,
  index: HashMap<LogIndex, LogDetails>,
}
```

Logging within a kernel then looks something like:

```rust

struct S {
    inputs_log: LogID<I>,
}

fn foo_state(inputs: I, state: S) -> S {
  log.record(state.inputs_log, inputs);
}
```

We could also split out the log context from the inputs and the
state. But that means you have to maintain a separate call 
