# RHDL Architecture

Structural blueprint for the RHDL workspace. This document captures *where things live and why*, so that future work — particularly AI-driven work — preserves the design boundaries the original author put in place.

This is the structural counterpart to `CLAUDE.md`. CLAUDE.md says how to *write a widget* (definition of done, conventions inside one widget file). This document says how the *workspace and crates* are organized, what depends on what, and what must not change without a compelling reason.

If you are about to:

- add a new crate,
- add a new top-level module inside an existing crate,
- restructure dependencies between crates,
- introduce a new IR layer,
- add a new top-level widget category,
- bypass the proc-macro layer or write Verilog as strings,

read this document first.

---

## 1 — Workspace at a glance

```
rhdl/                                           # Cargo workspace root
├── Cargo.toml                                  # workspace manifest
├── README.md                                   # project intro and roadmap
├── CHANGELOG.md                                # build-narrative log (see CLAUDE.md §16)
├── CLAUDE.md                                   # AI-agent operating contract
├── architecture.md                             # this document
├── widget-roadmap.md                           # prioritized widget list
├── auto-pipelining-plan.md                     # auto-pipelining design plan
├── kernel-language-extensions.md               # #[kernel] subset extensions plan
├── vendor-primitive-architecture.md            # target-provider trait design plan
├── fsm-architecture.md                         # FSM ergonomics + analysis + formal verification
├── rhdl-deep-dive.md                           # narrative architecture walkthrough
├── manifesto.md                                # essay on Rust HDL + LLM-assisted dev
│
├── crates/                                     # all 12 member crates
│   ├── Justfile                                # `just coverage` etc.
│   ├── rhdl/                                   # meta-crate (users depend on this)
│   ├── rhdl-bits/                              # Bits<N> / SignedBits<N>
│   ├── rhdl-core/                              # the compiler (RHIF/RTL/NTL) + types + sim
│   ├── rhdl-macro/                             # proc-macro entry points (thin)
│   ├── rhdl-macro-core/                        # proc-macro implementations
│   ├── rhdl-vlog/                              # Verilog AST, parser, pretty-printer
│   ├── rhdl-span/                              # source-span data type
│   ├── rhdl-trace-type/                        # enriched waveform-trace types
│   ├── rhdl-fpga/                              # *** widget library *** (most new work)
│   ├── rhdl-bsp/                               # board-support packages
│   ├── rhdl-toolchains/                        # external-toolchain glue (iverilog)
│   └── rhdl-surfer-plugin/                     # WASM plugin for Surfer waveform viewer
│
├── doc/                                        # documentation tree
│   ├── book/                                   # mdbook user manual (SUMMARY.md is the index)
│   ├── mdbook-rhdl/                            # custom mdbook preprocessor
│   ├── latte24/                                # LATTE '24 paper LaTeX source
│   ├── latte25/                                # LATTE '25 paper LaTeX source
│   ├── osda2024/                               # OSDA 2024 paper LaTeX source
│   └── references/                             # cited papers and notes
│
└── rhdl-std/                                   # WIP standard library of synthesizable fns
```

The *strategy and design documents at the workspace root* are first-class artifacts. Treat them like the rest of the codebase. Updates go through PR review; design pivots get a CHANGELOG entry. Do not move them into `doc/` or hide them in subfolders — they are deliberately at the root so a fresh contributor sees them on first checkout.

---

## 2 — The crate dependency graph

The dependency graph is *intentional and load-bearing*. Do not introduce new edges casually; do not invert existing edges.

```
                                     ┌──────────────────┐
                                     │  rhdl-bsp        │  L4
                                     └────────┬─────────┘
                                              │ uses widgets, builds boards
                                              ▼
                                     ┌──────────────────┐
                                     │  rhdl-fpga       │  L4
                                     └────────┬─────────┘
                                              │ user-facing widget lib
                                              ▼
              ┌──────────────────────────────────────────────────┐
              │                  rhdl  (meta-crate)              │  L3
              └────┬──────────────┬─────────────┬────────────────┘
                   │              │             │
                   ▼              ▼             ▼
   ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐
   │  rhdl-toolchains │  │   rhdl-macro     │  │   rhdl-core      │  L2
   └────────┬─────────┘  └────────┬─────────┘  └────────┬─────────┘
            │                     │                     │
            │                     ▼                     │
            │           ┌──────────────────┐            │
            │           │ rhdl-macro-core  │            │  L2 (proc-macro)
            │           └────────┬─────────┘            │
            │                    │                      │
            ▼                    ▼                      ▼
   ┌──────────────┐   ┌────────────────────────────────────────────┐
   │ rhdl-vlog    │   │  rhdl-bits  rhdl-span  rhdl-trace-type     │  L0
   └──────────────┘   └────────────────────────────────────────────┘
                                                          ▲
                                                          │
                                            ┌─────────────┴────────┐
                                            │ rhdl-surfer-plugin   │  side (WASM cdylib)
                                            │ (only trace-type)    │
                                            └──────────────────────┘
```

Verified dependency edges (from `crates/*/Cargo.toml`):

| Crate | Depends on (internal only) | Layer |
|---|---|---|
| `rhdl-bits` | — | L0 |
| `rhdl-span` | — | L0 |
| `rhdl-trace-type` | — | L0 |
| `rhdl-vlog` | — | L0 |
| `rhdl-core` | `rhdl-bits`, `rhdl-span`, `rhdl-trace-type`, `rhdl-vlog` | L2 |
| `rhdl-macro-core` | `rhdl-span`, `rhdl-vlog` | L2 (proc-macro support) |
| `rhdl-macro` | `rhdl-macro-core` | L2 (proc-macro entry) |
| `rhdl-toolchains` | `rhdl-core`, `rhdl-vlog` | L2 |
| `rhdl` | `rhdl-bits`, `rhdl-core`, `rhdl-macro`, `rhdl-trace-type`, `rhdl-vlog` | L3 (meta-crate) |
| `rhdl-fpga` | `rhdl` | L4 |
| `rhdl-bsp` | `rhdl`, `rhdl-fpga` | L4 |
| `rhdl-surfer-plugin` | `rhdl-trace-type` only | side (WASM) |

### Rules

**Foundational crates (L0) take no internal dependencies.** They are the substrate. `rhdl-bits`, `rhdl-span`, `rhdl-trace-type`, and `rhdl-vlog` may depend on each other only when there is no alternative — at the time of writing, they are independent. Adding an internal dependency between L0 crates is a structural change requiring justification.

**`rhdl-macro-core` deliberately does not depend on `rhdl-core`.** This is not an oversight. Procedural macros run *at compile time*, in the host toolchain's process. A proc-macro crate that depends on the runtime crate it generates code for creates build-time cycles and slows incremental compilation dramatically. The macro layer uses only `rhdl-vlog` (for embedded Verilog snippets in macro output) and `rhdl-span` (for source spans). All runtime semantics live in `rhdl-core`.

**`rhdl` is a thin meta-crate.** Its entire purpose is the `prelude` re-export tree. It contains no logic. If you find yourself wanting to add a function to `rhdl/src/`, the function probably belongs in one of the underlying crates with a re-export added to `rhdl/src/prelude.rs`.

**`rhdl-fpga` depends only on `rhdl`.** Widgets pull everything they need through the meta-crate's prelude. Do not add direct dependencies from `rhdl-fpga` on `rhdl-core` or other internal crates — it bypasses the public API surface.

**`rhdl-bsp` depends on `rhdl-fpga`.** Board-support code instantiates and composes widgets. Do not invert this. A widget that is too specific to one board belongs in the BSP, not in `rhdl-fpga`.

**`rhdl-surfer-plugin` is a sealed island.** It is a `cdylib` for Extism (WASM) and depends only on `rhdl-trace-type`. The trace-type crate exists in part to be the narrow API surface this plugin can target without dragging in the whole compiler.

**`rhdl-toolchains` is the only crate allowed to invoke external tools.** All `iverilog`, `yosys`, `nextpnr`, vendor-tool integration goes here. Do not invoke external processes from `rhdl-core` or `rhdl-fpga`.

---

## 3 — Inside `rhdl-core` — the compiler architecture

`rhdl-core` is by far the largest crate (~36k LOC). Its module structure encodes the multi-stage compiler's stage boundaries. Preserve them.

```
crates/rhdl-core/src/
├── lib.rs                              # public re-exports (thin)
├── error.rs                            # RHDLError, miette-decorated
├── util.rs                             # generic helpers (id, hashing)
├── clock_details.rs                    # clock metadata
│
├── ast/                                # captured Rust AST (post-#[kernel])
│   ├── ast_impl.rs                     # KernelFn, Expr, Pattern, Stmt
│   ├── builder.rs                      # constructors used by the macro
│   ├── visit.rs                        # AST visitor trait
│   └── spanned_source.rs
│
├── types/                              # the type system
│   ├── digital.rs                      # Digital trait
│   ├── timed.rs                        # Timed trait
│   ├── domain.rs                       # Domain (clock-domain phantom)
│   ├── signal.rs                       # Signal<T, Domain>
│   ├── clock.rs / reset.rs / clock_reset.rs / reset_n.rs
│   ├── kernel.rs                       # KernelFn marker traits
│   ├── digital_fn.rs                   # DigitalFn{1,2,...,6}
│   ├── kind.rs                         # runtime type descriptor (Kind)
│   ├── path.rs                         # field-access paths
│   ├── typed_bits.rs                   # bit-vector with Kind
│   ├── timed_sample.rs                 # simulator sample type
│   └── ...
│
├── bitx/                               # 4-state bit (BitX = 0/1/X/Z)
│
├── circuit/                            # Circuit / Synchronous traits + descriptors
│   ├── circuit_impl.rs                 # Circuit / CircuitIO / CircuitDQ traits
│   ├── synchronous.rs                  # Synchronous / SynchronousIO / SynchronousDQ
│   ├── descriptor.rs                   # Descriptor<AsyncKind | SyncKind>
│   ├── hdl_descriptor.rs               # HDLDescriptor (Verilog output bundle)
│   ├── adapter.rs                      # async/sync bridging
│   ├── chain.rs                        # composition of sub-circuits
│   ├── fixture.rs                      # top-level simulation/synthesis fixture
│   ├── drc.rs                          # design-rule checks
│   ├── scoped_name.rs                  # hierarchical name plumbing
│   ├── phantom.rs                      # phantom-marker types
│   ├── array/                          # array-of-circuits helper
│   ├── function/                       # Func wrapper for kernel-as-circuit
│   └── hdl/                            # HDL emission helpers per circuit family
│
├── compiler/                           # the multi-stage compiler
│   ├── mod.rs                          # exports compile_design entry point
│   ├── driver.rs                       # compile_design / compile_design_stage1
│   ├── stage1.rs                       # AST → MIR → RHIF (with passes)
│   ├── stage2.rs                       # RHIF → RTL (with passes)
│   ├── stage3.rs                       # RTL → NTL → optimize_ntl
│   ├── lower_rhif_to_rtl.rs            # the cross-IR lowering
│   ├── interner.rs                     # symbol/path interning
│   ├── error.rs                        # compiler-specific errors
│   ├── utils.rs
│   ├── mir/                            # MIR with type inference
│   │   ├── compiler.rs                 # compile_mir
│   │   ├── infer.rs                    # type inference (ena union-find)
│   │   ├── mir_impl.rs / ty.rs / lit.rs
│   │   └── error.rs
│   ├── rhif_passes/                    # Stage 1 transformations
│   │   ├── pass.rs                     # Pass trait
│   │   ├── check_clock_domain.rs       # CDC enforcement
│   │   ├── check_rhif_type.rs
│   │   ├── check_rhif_flow.rs          # use-before-write
│   │   ├── partial_initialization_check.rs
│   │   ├── constant_propagation.rs
│   │   ├── dead_code_elimination.rs
│   │   ├── lower_inferred_casts.rs
│   │   ├── lower_inferred_retimes.rs
│   │   ├── propagate_literals.rs
│   │   ├── precompute_discriminants.rs
│   │   ├── remove_unused_*.rs / remove_unneeded_muxes.rs / ...
│   │   └── symbol_table_is_complete.rs # invariant check between passes
│   ├── rtl_passes/                     # Stage 2 transformations
│   │   └── (similar Pass-trait pattern)
│   └── ntl_passes/                     # Stage 3 transformations
│       └── (similar Pass-trait pattern)
│
├── rhif/                               # RHDL Hardware Intermediate Form
│   ├── spec.rs                         # OpCode, Slot, Binary, Select, ...
│   ├── object.rs                       # Object (a compiled kernel)
│   ├── rhif_builder.rs                 # construct RHIF programmatically
│   ├── vm.rs                           # interpreter for kernel testing
│   ├── visit.rs / display_rhif.rs / remap.rs / runtime_ops.rs
│
├── rtl/                                # Register-Transfer Level IR (untyped SSA)
│   ├── spec.rs / object.rs / vm.rs / ...
│   └── symbols.rs                      # symbol-table extension for RTL
│
├── ntl/                                # Net-Transfer Layer (netlist IR)
│   ├── spec.rs                         # PrimitiveRequest, BlackBox, Wire ops
│   ├── object.rs / from_rtl.rs / hdl.rs / graph.rs
│   ├── builder.rs                      # constant, circuit_black_box, etc.
│   └── error.rs
│
├── hdl/                                # high-level HDL emission shim
│   ├── ast.rs / builder.rs / formatter.rs
│   └── (uses rhdl-vlog as the AST)
│
├── sim/                                # simulation framework
│   ├── iter/                           # iterator combinators (with_reset, clock_pos_edge, ...)
│   ├── probe/                          # observation probes (glitch_check, edge_time)
│   ├── run/                            # run extensions (RunExt, RunSynchronousExt)
│   ├── testbench/                      # SynchronousTestBench, TestBench
│   ├── test_module.rs                  # iverilog test-module wrapper
│   └── vcd.rs / mod.rs
│
├── trace/                              # waveform tracing
│   ├── session.rs / page.rs / record.rs
│   ├── container/                      # VcdFile, SvgFile, ...
│   ├── svg/                            # SVG renderer
│   └── vcd.rs / trace_sample.rs / key.rs
│
└── common/                             # shared utilities
    └── (symtab, etc.)
```

Stage boundary rules:

- **Each IR's `spec.rs` is the source of truth for that IR's opcodes.** A new opcode at any IR level requires (a) an entry in `spec.rs`, (b) a lowering rule from the previous IR if applicable, (c) a lowering rule to the next IR if applicable, and (d) test coverage for both lowerings.
- **Passes implement `trait Pass { fn run(Object) -> Result<Object> }`.** Live in `rhif_passes/`, `rtl_passes/`, or `ntl_passes/` matching their stage. New passes register in the appropriate `stage1.rs`/`stage2.rs`/`stage3.rs` driver.
- **`SymbolTableIsComplete` runs between passes as an invariant check.** Do not skip this; it catches half-applied transformations.
- **The MIR is private to Stage 1.** Do not expose MIR types from `rhdl-core`'s public API.

---

## 4 — Inside `rhdl-fpga` — the widget library

```
crates/rhdl-fpga/
├── Cargo.toml
├── src/
│   ├── lib.rs                          # registers each widget category
│   ├── doc.rs                          # write_svg_as_markdown helper for examples
│   ├── core/                           # foundation primitives
│   │   ├── mod.rs                      # `pub mod counter; pub mod dff; ...`
│   │   ├── counter.rs / dff.rs / delay.rs / option.rs / slice.rs / constant.rs
│   │   ├── edge_detector.rs / pulse_stretcher.rs / priority_encoder.rs / ...
│   │   ├── pwm.rs / crc.rs / mac.rs / divider.rs / barrel_shifter.rs / ...
│   │   └── ram/                        # RAM family (sync, async, option-wrapped)
│   ├── audio/                          # audio output / audio-protocol widgets
│   │   └── audio_pwm.rs                # PWM / sigma-delta stereo audio
│   ├── serial_bus/                     # protocol-PHY and serial-bus widgets
│   │   ├── uart.rs / uart_tx.rs / uart_rx.rs / uart_16550.rs
│   │   ├── spi_master.rs / spi_slave.rs / half_spi_master.rs
│   │   ├── i2c_master.rs / can_master.rs / lin_master.rs
│   │   ├── one_wire_master.rs / dht22.rs / sent_rx.rs / ir_nec_rx.rs
│   │   ├── midi.rs / ws2812.rs
│   │   └── ...
│   ├── video/                          # raster-display timing + format encoders
│   │   ├── video_timing.rs             # generic H/V counters + sync generator
│   │   ├── cga_rgbi.rs                 # IBM CGA digital RGBI
│   │   └── ntsc_composite.rs           # NTSC monochrome composite encoder
│   ├── cdc/                            # clock-domain crossing
│   ├── fifo/                           # synchronous and asynchronous FIFOs
│   ├── gray/                           # gray-code encoders/decoders
│   ├── pipe/                           # pipelined map/filter/...
│   ├── stream/                         # backpressure-aware stream cores
│   ├── axi4lite/                       # AXI4-Lite protocol family
│   ├── reset/                          # reset conditioners
│   ├── rng/                            # XorShift, etc.
│   ├── dsp/                            # DSP building blocks
│   ├── tristate/                       # tristate buffers
│   └── lid/                            # latency-insensitive design (Carloni)
│
├── examples/                           # one runnable example per widget
│   └── <widget>.rs                     # produces an SVG/VCD trace via doc::write_svg_as_markdown
│
├── doc/                                # committed waveform output
│   └── <widget>.md                     # generated by the example, embedded by include_str!
│
├── vcd/                                # committed reference VCDs
│   └── <widget>/<widget>.vcd           # SHA256-digest-checked
│
└── tests/                              # cross-widget integration tests (rare)
```

Conventions enforced:

- **Categories already exist; reuse them.** `core`, `audio`, `serial_bus`, `video`, `cdc`, `fifo`, `gray`, `lid`, `pipe`, `reset`, `rng`, `dsp`, `stream`, `axi4lite`, `tristate`. A new category requires a strong reason — the widget doesn't fit any existing category, and at least two related widgets motivate the new top-level module.  Foundation primitives (registers, RAMs, counters, arithmetic, control widgets) live in `core/`; off-chip-facing protocol PHYs live in `serial_bus/`; raster-display widgets live in `video/`; audio-output / audio-protocol widgets live in `audio/`.  When a `serial_bus/` or `video/` widget needs a `dff` / `constant` / `pwm` / `edge_detector` / `pulse_stretcher` from `core/`, it imports via `use crate::core::{dff, constant};` rather than `use super::*;` (sibling-only `super::` references are reserved for intra-category composition such as `serial_bus::midi → serial_bus::uart::Uart`).
- **One widget per file.** Group related widgets under a `mod.rs` (see `src/fifo/`, `src/stream/`).
- **Every widget has the four companion artifacts** in their canonical locations: `src/<cat>/<name>.rs`, `examples/<name>.rs`, `doc/<name>.md`, `vcd/<name>/<name>.vcd`. See CLAUDE.md §3 for the file-anatomy template.
- **Internal sub-modules in a category get registered in the category's `mod.rs` and do not become public unless they have a separate user-facing surface.** `fifo::write_logic` and `fifo::read_logic` are public because users may want to compose them; `fifo::testing::*` is a `#[cfg(test)]`-friendly helper module.

---

## 5 — Cross-crate mechanisms

### 5.1 Proc-macro layer

The macro layer is split deliberately:

- **`rhdl-macro`** (`proc-macro = true`). Thin wrapper. Each entry point is a single function delegating to `rhdl-macro-core`. Live entries: `#[derive(Digital)]`, `#[derive(Timed)]`, `#[derive(Circuit)]`, `#[derive(CircuitDQ)]`, `#[derive(Synchronous)]`, `#[derive(SynchronousDQ)]`, `#[kernel]`, `path!`, `export!`, `bind!` (re-exported from `rhdl::prelude`).
- **`rhdl-macro-core`** (regular library, NOT `proc-macro = true`). Holds the actual code-generation logic. Tests for the macro logic live here, not in `rhdl-macro`.

This split lets the macro logic itself be unit-tested. Do not collapse the two crates.

### 5.2 Trait dispatch into the framework

A widget integrates with the framework via three trait pairs, all in `rhdl-core::circuit`:

- `Synchronous` + `SynchronousIO` + `SynchronousDQ` — the single-clock-domain family.
- `Circuit` + `CircuitIO` + `CircuitDQ` — the multi-domain family.
- `Func<T, S>` — kernel-as-circuit wrapper (used by `stream::map`, etc.).

The `#[derive(Synchronous)]` macro generates the `Synchronous` impl from the struct's fields. The user supplies the `SynchronousIO` impl (which names the kernel) and gets `SynchronousDQ` either by `#[derive(SynchronousDQ)]` (auto, recommended) or by hand for unusual cases like `DFF` where `D = Q = ()`.

Do not bypass these traits to plug a hand-written widget directly into the simulator or HDL emitter. The traits are how the framework knows how to fan out clock and reset, how to compute the Q/D bundle, and how to run the kernel.

### 5.3 Verilog AST (`rhdl-vlog`), not strings

All Verilog output flows through `rhdl-vlog` as a typed AST. The `parse_quote!` macro from `syn` produces an AST literal; `Pretty::pretty()` formats it. **Never** assemble Verilog as a `String` or `format!`. Doing so produces unmaintainable output and silently strips type information that the AST exposes to downstream passes.

The decision to replace string templating with a proper AST is recorded in the README plan as a completed milestone. Do not regress it.

### 5.4 Kernel-as-pure-fn invariant

A `#[kernel]` function is, semantically, a pure function over `Digital` types. The framework relies on this for:

- IR lowering (RHIF assumes value-only semantics).
- Iterator-based simulation (kernels can be called directly from `cargo test`).
- Auto-pipelining (a pure kernel is a DAG, which retiming requires).
- Functional-equivalence testing (original vs. transformed kernel).

The forbidden list in CLAUDE.md §4 enforces this — no references, no heap, no closures with captures. These restrictions are not stylistic; they are the foundation of every transformation in the compiler. Do not relax them without first updating the relevant compiler invariants.

---

## 6 — What must not change without a compelling reason

The following are *architectural decisions*. Changing any of them is a structural pivot that requires a CHANGELOG entry, a design-doc update, and reviewer sign-off.

1. **The four-crate L0 substrate.** `rhdl-bits`, `rhdl-span`, `rhdl-trace-type`, `rhdl-vlog` are the foundation. Adding internal dependencies between them, or adding new L0 crates, requires written justification.
2. **The `rhdl-macro` / `rhdl-macro-core` split.** Do not collapse. Do not let `rhdl-macro-core` depend on `rhdl-core`.
3. **The IR layering: RHIF → RTL → NTL.** Three IRs, in this order, with lowering between consecutive levels and a pass registry per stage. Adding a fourth IR is a major architectural change. Skipping or reordering levels is forbidden.
4. **The `Pass` trait in `rhdl-core::compiler::*_passes`.** Every transformation is a `Pass`. Do not write inline transforms inside the stage drivers.
5. **Verilog through `rhdl-vlog` AST, not strings.** No regressions on this.
6. **Kernel as pure `fn` over `Digital`.** No references, no heap, no captured closures.
7. **The `Synchronous` / `Circuit` trait families dispatch sub-circuits through `D` and `Q` aggregates derived by macro.** Do not write custom sub-circuit composition that bypasses the auto-derived `D`/`Q`.
8. **One widget per file, with the four artifacts** (source, example, doc/.md, vcd/.vcd). Per CLAUDE.md TL;DR.
9. **Strategy and design documents at the workspace root.** Their location is part of the contract for new contributors.
10. **The vendor-primitive architecture: target as a parameter to `Descriptor::hdl_for(&target)`, not as a generic on widgets.** Per `vendor-primitive-architecture.md`. Widgets stay target-agnostic.

---

## 7 — How to evolve the architecture

When new structural needs arise, follow these patterns rather than improvising.

**New widget category.** Add a new top-level module under `crates/rhdl-fpga/src/`, register it in `lib.rs`, populate it with at least two widgets that share the category's concern, and update `widget-roadmap.md` to mention it. If only one widget motivates the category, put the widget in an existing category and reconsider when the second arrives.

**New IR pass.** Pick the stage (RHIF / RTL / NTL), drop the file in the matching `*_passes/` directory, implement `Pass`, register in the stage driver, write before/after `expect_test` snapshots. Update `auto-pipelining-plan.md` if the pass is timing-relevant.

**New language feature inside `#[kernel]`.** Update `kernel-language-extensions.md` first (proposal + lowering sketch), then implement. The macro layer and possibly `rhdl-core` MIR/RHIF are involved. Update CLAUDE.md §4 (allowed/forbidden list) when shipped.

**New target / vendor primitive.** Per `vendor-primitive-architecture.md`. Add a `Target` impl, register primitives, supply simulation models if needed. The widget API does not change.

**New crate at the workspace level.** Justify why the existing twelve are insufficient. Place it in the dependency graph at §2 and update this document. Strong default is *no* — most new functionality belongs inside an existing crate.

**Restructuring `rhdl-core`'s internal modules.** Update §3 of this document. The IR-stage boundaries (`rhif/`, `rtl/`, `ntl/`, `compiler/*_passes/`) are part of the architecture and require especially careful review.

---

## 8 — How this document relates to the others

| Document | Scope | Mutability |
|---|---|---|
| `architecture.md` (this) | Workspace structure, crate dep graph, IR layering, where things go | Stable; updates require reviewer sign-off |
| `CLAUDE.md` | How to write a widget; the contract for "done" | Updated with each new convention |
| `widget-roadmap.md` | Which widgets to build next | Updated as widgets ship |
| `auto-pipelining-plan.md` | Future auto-pipelining feature | Updated as the feature is built |
| `kernel-language-extensions.md` | Future kernel-language extensions | Updated as extensions ship |
| `vendor-primitive-architecture.md` | Future target-provider system | Updated as primitives are added |
| `fsm-architecture.md` | Future FSM ergonomics + analysis + formal verification | Updated as phases ship |
| `rhdl-deep-dive.md` | Narrative architecture walkthrough (descriptive) | Updated when major architectural shifts happen |
| `manifesto.md` | Why Rust HDL + LLM-assisted dev | Reference document; rarely changes |
| `CHANGELOG.md` | Build narrative — every shipped change | Append-only |
| `README.md` | Project introduction and roadmap checklist | Updated as roadmap items ship |

When this document and another disagree, this document wins for structural concerns (where things go, what depends on what). CLAUDE.md wins for widget-authoring concerns (how a single widget is built and tested). Design plans win for forward-looking feature concerns (how a planned feature should work). Where the boundaries blur, file an issue rather than guessing.

---

## 9 — Why these constraints exist

A few of the rules look arbitrary at first reading. Recording the rationale here so it is not re-litigated.

**Why a separate `rhdl-vlog` crate.** Verilog AST manipulation is heavy on `syn` and `quote`, with a non-trivial pretty-printer. Isolating it from `rhdl-core` keeps `rhdl-core` build times reasonable and lets non-RHDL projects use the Verilog AST library independently.

**Why `rhdl-trace-type` is its own crate.** The Surfer waveform-viewer plugin (`rhdl-surfer-plugin`) is compiled as a `cdylib` for WebAssembly via Extism. It must avoid pulling in `rhdl-core` or anything large. `rhdl-trace-type` is a tiny crate with just the trace-type definitions, and the Surfer plugin depends on only that. Without this split, the WASM plugin balloons.

**Why `rhdl-macro` and `rhdl-macro-core` are split.** Procedural macros must live in a `proc-macro = true` crate, which has the constraint that it can only export proc-macros. Moving the *implementation* into a regular library crate (`rhdl-macro-core`) lets the macro logic itself be unit-tested without going through the proc-macro test harness.

**Why three IRs.** RHIF preserves Rust types (essential for type-checked optimizations and good error messages). RTL drops type info but stays SSA at bit-level (cleaner for arithmetic optimizations and operand-width-aware passes). NTL is a true netlist (the right granularity for timing analysis, retiming, vendor primitives). A two-IR design (à la FIRRTL alone) loses the Rust-type-aware optimization phase; a four-IR design buys nothing observed yet. Three is the empirically-justified count.

**Why widgets depend on `rhdl` and not `rhdl-core` directly.** The meta-crate's `prelude` is the *public API surface*. If a widget reaches around the prelude into `rhdl-core` directly, every API churn breaks the widget. Going through the prelude means the prelude is the contract — and breaking changes are visible in one file.

**Why the doc/ tree is separate from crates/.** The mdbook user manual, the LATTE papers, and the strategy docs at root are not code artifacts but they are first-class deliverables. Keeping them at workspace level (rather than buried in a crate) makes them discoverable. Keeping them out of `crates/` keeps `cargo build` fast and `cargo doc` focused on the API.

---

## 10 — Final word for AI agents

If you are an AI agent reading this — and you very likely are — and you are about to do *any* of the following, stop and re-read the relevant section of this document:

- Adding a `pub mod` to `crates/rhdl/src/lib.rs`. The meta-crate is thin; new modules belong in an underlying crate.
- Adding `rhdl-core` as a dependency of `rhdl-macro-core`. Do not. See §2 and §9.
- Writing Verilog as a `format!`-ed string. Do not. Use `rhdl-vlog`'s `parse_quote!` and the `Pretty` formatter.
- Importing `Vec`, `String`, or `Box` inside a `#[kernel]` function. The kernel subset is heap-free.
- Creating a new top-level widget category in `rhdl-fpga` because your new widget "doesn't fit." Three out of four times, it fits in `core` or `dsp`.
- Skipping the `examples/<name>.rs` or `doc/<name>.md` for a new widget because "it's just a small one." Per CLAUDE.md, no it isn't.
- Creating a new IR or a fourth pass stage. Don't. The three-IR pipeline is the architecture.
- Threading a `Target` generic through every widget for vendor-primitive support. Don't. Targets are a codegen-time argument; see `vendor-primitive-architecture.md`.

When in doubt: search this document for the structural concept; search CLAUDE.md for the conventions; consult the relevant design plan if it's a forward-looking feature; ask a human if you're still unsure. Architectural integrity is cheaper to preserve than to recover.
