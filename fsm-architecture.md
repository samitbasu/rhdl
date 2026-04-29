# FSM Architecture for RHDL — Design Plan

A proposal for first-class finite-state-machine support in RHDL: an ergonomic syntax for declaring FSMs, static analyses that catch a meaningful class of bugs at compile time, auto-generated state-diagram visualizations in rustdoc, and a formal-verification path that can prove temporal properties about the machine.

This is the fourth compiler-and-language design plan, alongside `auto-pipelining-plan.md`, `kernel-language-extensions.md`, and `vendor-primitive-architecture.md`. Like those, it is independently shippable in phases; some phases compose, but none blocks the others.

The plan rests on a simple observation: a large fraction of every RHDL widget is an FSM in disguise (the FIFO write-logic, every protocol PHY, every arbiter, every register-mapped peripheral), and the language already has *most* of the right primitives — ADTs with discriminants, exhaustive `match`, kernels as pure functions of `(state, input)`. What's missing is (a) the surface syntax to make the pattern read like an FSM rather than a generic Rust function, (b) the static analyses that exploit the structure, and (c) the formal-verification surface that the kernel-as-pure-fn invariant uniquely makes tractable.

---

## 1 — Motivation

Hardware design has been writing FSMs since 1956 (Mealy) and 1956 (Moore — same year, separately). Every modern HDL has some recognition of the FSM pattern:

- **SystemVerilog** has `unique`/`priority` case keywords plus SVA temporal assertions.
- **Bluespec** schedules guarded atomic actions; the scheduler proves correctness.
- **SpinalHDL** ships a `StateMachine` library with declarative syntax and a graph-export tool.
- **Spade** has a `state` keyword for staged state machines.
- **Chisel** uses match-based FSMs but offloads verification to ChiselTest / SymbiYosys.
- **nMigen / Amaranth** has `m.FSM()` blocks plus `Past`, `Stable`, `Cover` for SVA-style assertions and built-in SymbiYosys integration.

RHDL's current FSM idiom is the bare match-on-state pattern (see `crates/rhdl-fpga/src/fifo/write_logic.rs` for a production example). It is correct, exhaustive, and works — but it is also verbose, scatters the per-state outputs across separate match arms, and provides no leverage for the analyses other HDLs offer for free.

The cost of *not* providing FSM-specific tooling compounds. Widget #18 (UART), #19/20 (SPI), #21 (I²C), #25 (CAN), #28 (LIN), #31 (SENT), #37 (MIDI), #46 (battery management) — every one is an FSM. An ergonomic boost there is a force multiplier across half the roadmap. A static analysis catching unreachable states or deadlocked transitions catches bugs before any cycle is simulated. A formal-verification flow that proves "the FIFO write logic never overflows" is the kind of guarantee that moves RHDL into territory only Bluespec and academic Coq-verified HDLs reach today.

The kernel-as-pure-fn invariant — the same invariant that makes auto-pipelining sound — is exactly what formal verification of FSMs requires. Bounded model checking, symbolic execution, and SVA-style temporal proofs all require a referentially transparent state-transition function. RHDL has that by construction. Whatever tool chain we build, we will not be fighting against the language; we will be exploiting structure that's already there.

---

## 2 — Where FSMs live in RHDL today

The canonical pattern, drawn from the existing code:

```rust
#[derive(PartialEq, Digital, Copy, Clone, Debug, Default)]
pub enum State {
    #[default]
    Idle,
    Running { counter: b8 },
    Done,
}

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
#[rhdl(dq_no_prefix)]
pub struct MyMachine {
    state: DFF<State>,
}

#[kernel]
pub fn my_machine(cr: ClockReset, i: In, q: Q) -> (Out, D) {
    let next = match q.state {
        State::Idle => if i.start {
            State::Running { counter: bits(0) }
        } else {
            State::Idle
        },
        State::Running { counter } => if counter == bits(255) {
            State::Done
        } else {
            State::Running { counter: counter + 1 }
        },
        State::Done => State::Idle,
    };
    let mut o = Out::dont_care();
    o.busy = matches!(q.state, State::Running { .. });
    o.done = matches!(q.state, State::Done);
    if cr.reset.any() {
        return (Out::default(), D { state: State::Idle });
    }
    (o, D { state: next })
}
```

Three things make this nice already: enum-with-payload state types are first-class, exhaustive `match` is enforced by `rustc`, and the kernel is a pure function of `(state, input)`. Three things make it less nice: the next-state logic and the per-state outputs live in separate places (a maintenance hazard), guards aren't supported yet (tracked in `kernel-language-extensions.md` §2.4), and the framework doesn't *know* this is an FSM — it's just a kernel that happens to wrap a `DFF<State>`.

---

## 3 — Design space — five layers

The work splits into five layers, each with different cost and different payoff. They compose: layer N is more useful when layers 1..N-1 are present, but layer N can ship without later layers.

| Layer | What it does | Cost | Payoff |
|---|---|---|---|
| 1 | Ergonomic `#[derive(Fsm)]` macro | low (~2 weeks) | high — better readability, less boilerplate |
| 2 | Static reachability + dead-state analysis | medium (~4 weeks) | high — catches bugs at compile time |
| 3 | Auto-generated state diagrams in rustdoc | low (~2 weeks) | medium-high — visualization, LLM-friendly |
| 4 | Invariant assertions + SymbiYosys integration | medium-high (~6 weeks) | very high — formal proofs of safety properties |
| 5 | Built-in bounded model checker | high (~6 months) | very high — self-contained verification |

The recommended phasing is 1, 2, 3, 4, 5 in order, with 1+2+3 shipping as a single coherent "FSM ergonomics + static analysis" track and 4 shipping as a separate "formal verification" track. Layer 5 is a research-grade follow-on.

---

## 4 — Layer 1: the `#[derive(Fsm)]` macro

### 4.1 Syntax

The principle: stay in idiomatic Rust syntax; do not invent new keywords; do not require rust-analyzer to understand a DSL. Hardware authors are already learning RHDL's kernel subset; learning a separate DSL on top would be a net-negative ergonomic. Prior art that gets this right: Rust's own `serde::{Serialize, Deserialize}` derives.

```rust
#[derive(Fsm, PartialEq, Digital, Copy, Clone, Debug, Default)]
#[fsm(initial = "Idle")]
pub enum State {
    #[default]
    Idle,
    Running { counter: b8 },
    Done,
}

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
#[rhdl(dq_no_prefix)]
#[fsm(state_field = "state")]
pub struct MyMachine {
    state: DFF<State>,
}

#[kernel]
#[fsm_kernel(state_var = "q.state")]
pub fn my_machine(cr: ClockReset, i: In, q: Q) -> (Out, D) {
    /* same body as before — just plain RHDL */
}
```

Three light-weight attributes carry all the metadata:

- `#[derive(Fsm)]` on the state enum marks the type as an FSM state, registering it for analysis. The macro emits a small impl of an `FsmState` trait that records the variant names, default variant, and discriminant layout. Cost: ~50 LOC of macro output per FSM.
- `#[fsm(state_field = "...")]` on the widget struct names the field that holds the state DFF. This is the only field-level annotation needed; everything else is inferred.
- `#[fsm_kernel(state_var = "...")]` on the kernel function names the local expression the kernel matches against. The default — `q.<state_field>` — covers the canonical case and the attribute is usually omittable.

The kernel body itself is **unchanged**. No new syntax inside the kernel. The macro layer just records where the FSM lives; the analysis layer reads the kernel's existing match-on-state structure.

This conservative choice is deliberate. It means:

- Existing widgets can opt into FSM tooling by adding three attribute lines and zero code changes.
- LLM-generated kernels work as-is — the macro is a *declaration* that says "this widget is an FSM," not a transformation.
- The `Synchronous` trait, the `D`/`Q` derive machinery, the existing reset-handling idiom, and every other widget convention compose with FSM tooling without modification.
- If a future agent wants to add a more declarative DSL on top (à la SpinalHDL `StateMachine`), it can lower to this same attribute-annotated form — the underlying contract stays.

### 4.2 What `#[derive(Fsm)]` actually emits

The derive expands to an implementation of:

```rust
pub trait FsmState: Digital {
    const VARIANTS: &'static [FsmVariantDescriptor];
    const INITIAL: usize;  // index into VARIANTS
    fn variant_name(&self) -> &'static str;
    fn variant_index(&self) -> usize;
}

pub struct FsmVariantDescriptor {
    pub name: &'static str,
    pub discriminant: TypedBits,
    pub field_kinds: &'static [Kind],  // for variants with payloads
}
```

This metadata is what the static analyses (Layer 2) and the diagram generator (Layer 3) consume. It costs ~100 LOC per FSM at compile time and produces zero runtime cost — the trait is purely a static reflection surface.

### 4.3 Optional: `#[fsm_state(...)]` per-variant decoration

For richer diagram output, individual variants can carry display hints:

```rust
#[derive(Fsm, ...)]
pub enum State {
    #[fsm_state(label = "idle, waiting for start")]
    #[default]
    Idle,
    #[fsm_state(label = "running, counter = {counter}")]
    Running { counter: b8 },
    #[fsm_state(label = "complete", terminal)]
    Done,
}
```

These are pure display metadata, consumed only by the diagram generator. The `terminal` flag is a hint to the static analysis that a state with no outgoing transitions is *intentional* rather than a deadlock bug.

### 4.4 Acceptance criteria for Layer 1

Per `CLAUDE.md` §11.1:

1. `#[derive(Fsm)]` compiles and produces a valid `FsmState` trait impl for any `Digital`-derived enum.
2. The metadata is correct — variant names, discriminants, payload kinds match the source enum.
3. Three existing FSM-shaped widgets (`fifo::write_logic`, `core::round_robin_arbiter`, `core::crc`) are rewritten to opt into the FSM derives, with byte-identical emitted Verilog (the macro is purely additive metadata).
4. New chapter `doc/book/src/fsm/derive.md` documenting the syntax with one worked example.
5. Tests in `crates/rhdl-macro-core/src/expect/` snapshotting the `#[derive(Fsm)]` expansion.

---

## 5 — Layer 2: static reachability and dead-state analysis

### 5.1 What it analyzes

Once the macro layer has identified a kernel as an FSM, a new RHIF pass walks the kernel's match-on-state opcodes and constructs a directed graph:

- **Nodes**: the variants of the state enum.
- **Edges**: `(source variant, target variant)` pairs derivable from the match arms — for each arm `State::A => /* expr that may return State::B */`, the analysis traces the expression's potential return values and adds an edge `A → B` for each.

The graph is then walked to produce diagnostics:

- **Unreachable states.** Any state that cannot be reached from the initial state via any path. Surfaced as a `miette` warning by default, escalatable to error via attribute. Catches "I added a variant to the enum but forgot to wire any transition into it."
- **Dead transitions.** Any transition whose source is itself unreachable. These are correctness-irrelevant but indicate dead code.
- **Deadlock candidates.** Any state with no outgoing transitions and not marked `terminal` via `#[fsm_state(terminal)]`. The classic "I forgot to add a transition out of this state" bug. False positives if the user *meant* a self-absorbing state, hence the explicit terminal annotation.
- **Non-deterministic transitions.** Any `(state, input)` pair where multiple match arms produce different next-states. With current RHDL this is impossible because match arms are tried in order and the first match wins, but once guards land it becomes a real concern: `State::A if cond1 => B`, `State::A if cond2 => C` — what if both `cond1` and `cond2` hold? Layer 2 reports this as a structural warning, regardless of whether `cond1 ∧ cond2` is actually satisfiable.
- **Self-loop saturation.** A state with no transitions other than self-loops, not marked `terminal`. Often a bug.

### 5.2 Implementation

A new RHIF pass `analyze_fsm_structure` in `crates/rhdl-core/src/compiler/rhif_passes/`. Implements `Pass` per `architecture.md` §3. Reads the `FsmState` trait metadata via the macro-layer registration; walks the RHIF for kernels whose return value is `(O, D)` where `D` contains a state field; builds the transition graph; emits miette diagnostics.

The pass is *advisory only* — it does not transform the IR. Diagnostics surface as warnings unless the user opts into errors via `#[fsm(strict)]` on the widget struct.

### 5.3 Algorithm sketch

```
input: kernel K with declared state-DFF field S of type E
output: set of diagnostics

1. extract the state-update expression from K's kernel body
   (the expression assigned to D.S, after the match-on-state)
2. for each match arm "State::Variant_i => expr_i":
   2a. compute the set of state values that expr_i can produce,
       by recursive case analysis through nested match/if/let
   2b. for each producible state value State::Variant_j:
       add edge (Variant_i, Variant_j) to graph G
3. run BFS from State::INITIAL on G; mark visited variants
4. for each unvisited variant:
   emit "unreachable state" diagnostic
5. for each variant V with no outgoing edges and not terminal:
   emit "deadlock candidate" diagnostic
6. for each pair of arms in the match producing different targets
   under the same pattern but different guards:
   emit "potentially non-deterministic transition" diagnostic
```

Step 2a is the only non-trivial part — it requires walking arbitrary kernel expressions to compute the set of state values they can produce. For the canonical idiom (`if-else` and nested `match`) this is straightforward. For pathological cases (computing a state via field assignment to a `dont_care()`), the analysis falls back to "could be any variant" and warns conservatively. Worst case: false positives reported; never false negatives that hide real bugs.

### 5.4 Acceptance criteria for Layer 2

1. Pass produces zero false negatives — every unreachable state is reported.
2. Pass produces zero false positives on the existing widget corpus — no spurious "unreachable" warnings for any committed FSM widget.
3. Pass handles the core idioms: `match` on state, nested `if/else` returning state values, struct-field assignment via `dont_care()` then field-set.
4. Each diagnostic includes a span pointing at the offending source line.
5. New chapter `doc/book/src/fsm/static_analysis.md` documenting the pass and its diagnostics.
6. CHANGELOG entry per §16.

---

## 6 — Layer 3: auto-generated state diagrams

### 6.1 What it produces

For every widget with `#[derive(Fsm)]` in scope, the rustdoc gets an auto-generated state diagram. The same metadata that drives Layer 2's analysis drives the diagram: nodes are variants, edges are transitions, labels are derived from `#[fsm_state(label = "...")]` attributes.

The renderer emits two formats:

- **Inline SVG** for rustdoc, via the existing `badascii_doc` machinery or a small new svg emitter that takes the transition graph and lays it out using a tractable algorithm (Sugiyama for hierarchical graphs; force-directed for cyclic ones). The SVG is embedded directly in the doc, so it survives offline rustdoc usage without external Graphviz dependency.
- **Graphviz `dot`** for the user, via `cargo run --example fsm_dump --package rhdl-fpga`. Useful for piping into `dot`, `xdot`, or external graph-analysis tools.

### 6.2 Implementation

A new helper crate `rhdl-fsm-diagram` (or a module inside `rhdl-fpga::doc`) that takes a `FsmState`-implementing type plus the transition graph from Layer 2 and produces SVG. The graph is computed at compile time by the Layer-2 pass and stashed in the widget's `Descriptor`; the rustdoc tooling reads it from there.

For the diagram itself, layout is the only meaningful complexity. Two passes:

1. Compute strongly-connected components of the transition graph. Render SCCs as boxes; non-SCC components as standard nodes.
2. Lay out using a layered algorithm (Sugiyama) for the DAG of SCCs and a circular layout inside each SCC.

This avoids the dependency on Graphviz at build time while producing diagrams that look approximately like Graphviz's `dot` output.

### 6.3 LLM-friendliness

A specifically interesting property: the generated diagram is a *structural* representation of the FSM that an LLM agent can read alongside the source. When an agent modifies the kernel (adds a transition, splits a state), the regenerated diagram shows the structural diff. This is a faster feedback loop than reading match arms — pattern matching against a graph is something LLMs do well.

The diagrams should be embedded in the rustdoc as both SVG and as a structured representation (JSON or a Rust constant) so that an LLM-driven workflow can consume the structured form when generating or modifying widgets.

### 6.4 Acceptance criteria for Layer 3

1. Every widget with `#[derive(Fsm)]` in scope produces a state diagram in its rustdoc, with no extra annotations needed by the widget author.
2. The diagram is up-to-date with the source by virtue of being derived from it — no `cargo run --example` step required for rustdoc.
3. Three existing FSM widgets have their rustdoc inspected and the diagrams reviewed for accuracy.
4. The structured (JSON) representation is also available for LLM-tool consumption.

---

## 7 — Layer 4: invariant assertions and SymbiYosys integration

### 7.1 Syntax

Properties live as attributes on the kernel function:

```rust
#[kernel]
#[fsm_invariant(after_reset = "state != State::Error")]
#[fsm_invariant("(state == State::Running) implies output.busy")]
#[fsm_liveness(eventually = "state == State::Done", from = "State::Initialized", within = 10000)]
#[fsm_cover(reachable = "state == State::Running { counter: bits(100) }")]
pub fn my_machine(cr: ClockReset, i: In, q: Q) -> (Out, D) { /* ... */ }
```

Four kinds of properties:

- `fsm_invariant`: a Boolean expression that must hold every cycle. Lowers to `assert property (@(posedge clock) (cond));` in SVA.
- `fsm_liveness`: a property that must eventually hold, optionally bounded. Lowers to `assert property (@(posedge clock) eventually cond);` (unbounded) or `assert property (@(posedge clock) ##[1:N] cond);` (bounded).
- `fsm_cover`: a coverage point — does the design ever reach this state? Lowers to `cover property (@(posedge clock) cond);`. Useful as both verification and dead-code detection.
- `fsm_assume`: an assumption about the environment that the proof can rely on. Lowers to `assume property (@(posedge clock) cond);`.

The expression language is a strict subset of the kernel-accepted expression language: equality, comparison, Boolean ops, field access, `matches!`, no calls (to keep symbolic execution tractable in Layer 5). The lowering parses the expression at compile time and emits the corresponding SVA expression directly.

### 7.2 SymbiYosys flow

The user runs:

```sh
cargo rhdl prove --package rhdl-fpga --widget MyMachine --engine smtbmc:z3 --depth 64
```

(`cargo rhdl` is a new cargo subcommand we'd add in `rhdl-toolchains`; alternative is a standalone `rhdl-prove` binary.)

This:

1. Compiles the widget to Verilog with the SVA properties embedded.
2. Generates a `.sby` config file declaring the SVA properties as `assert` mode (for invariants and liveness) or `cover` mode (for coverage points).
3. Invokes `sby` (SymbiYosys) on the generated config.
4. Parses the result; on counterexample, presents the witness as a structured trace (cycle-by-cycle inputs and resulting state evolution), with line numbers pointing into the kernel source.

The user-visible workflow is: write the property, run one command, get either "PROVED" or a concrete failing trace. Same UX as `cargo test` but for formal proofs.

### 7.3 Why SymbiYosys?

It's the canonical open-source formal-verification frontend for Verilog/SystemVerilog. Driven by Yosys; supports multiple proof engines (smtbmc, abc-pdr, z3, boolector, yices, cvc4); handles BMC, k-induction, and unbounded proofs; and is what nMigen/Amaranth, Spade, and SymbiFlow all use. Free, mature, and well-documented. Replicating it in pure RHDL would be Layer 5; integrating with it is Layer 4.

### 7.4 Acceptance criteria for Layer 4

1. Three widgets get a property suite that proves a meaningful invariant. Recommended initial corpus: `fifo::write_logic` (proves: write_address never wraps past read_address in the absence of an overflow), `core::crc` (proves: result matches reference for known test vectors), `core::round_robin_arbiter` (proves: liveness — every requesting client eventually grants).
2. The `cargo rhdl prove` subcommand is documented in the book.
3. CI runs `sby` on the verified widgets if `iverilog` is available; otherwise skips with a documented reason.
4. The Verilog-emission path supports SVA properties without breaking the existing `iverilog` round-trip — assertions are guarded behind a compilation flag so iverilog (which doesn't natively support all SVA) still simulates cleanly.

---

## 8 — Layer 5: built-in bounded model checker

The aspirational research-grade follow-on. RHDL has the kernel as a pure Rust function; symbolic execution of that function over `(state, input)` for K cycles, using a SAT/SMT solver, gives bounded model checking inside the language's own toolchain — no external Verilog flow required.

The implementation would integrate `z3` via its Rust bindings or `boolector` via FFI. Each kernel becomes a transition function `T: (State, Input) → State × Output`. BMC unrolls T for K steps and asserts that all configured invariants hold; the solver searches for a counterexample.

This is a 6+ month project on its own, but the payoff is significant: an LLM agent can propose a kernel and *prove* a property in seconds without invoking external tools. For the LLM-assisted-development thesis, this is the killer feature — agents become formally verifiable in a closed loop.

### 8.1 Why this is uniquely tractable in RHDL

- **Kernel-as-pure-fn.** No closures, no I/O, no heap. Each kernel call is a pure mathematical function. Symbolic execution is sound.
- **ADTs with finite discriminants.** State spaces are statically bounded.
- **The IR is already amenable.** RHIF is typed SSA — exactly the IR shape symbolic-execution engines want to consume.
- **Validation infrastructure already exists.** The iterator-based simulator, the existing `expect_test` machinery, and the `cargo test`-as-eval-harness all generalize to "verify that the BMC's counterexamples are real failures."

The principal risk is solver performance — 16-bit-wide arithmetic in a state struct can blow up the SMT formula. Mitigations: bitblasting helpers in the lowering, decomposition of multi-field states into independent sub-FSMs where possible, opt-in proof-engine selection (k-induction for invariants, BMC with depth bounds for liveness).

This layer is out of scope for an initial implementation. It belongs in the document as a destination, not as a near-term task.

---

## 9 — Validation requirements

Per `CLAUDE.md` §11.1, every layer is a compiler-level change with the full PR contract: one feature per PR, tests at every level, Justification section in the PR description, documentation in code + book + this design plan, CHANGELOG entry naming the guarantee preserved.

Specific test requirements per layer:

- **Layer 1.** Macro-expansion snapshot tests in `crates/rhdl-macro-core/src/expect/`. Three real-widget integration tests (FIFO write logic, round-robin arbiter, CRC) each rewriting an existing kernel with `#[derive(Fsm)]` and verifying the emitted Verilog is byte-identical.
- **Layer 2.** Pass-level unit tests with `expect_test` snapshots of the diagnostic output on hand-crafted FSMs (one with an unreachable state, one with a deadlock, one with a non-deterministic transition under guards). Negative test that an FSM with no issues produces no warnings.
- **Layer 3.** Snapshot test of the SVG output for a known FSM. Snapshot of the structured JSON representation. Manual review of three real widgets' diagrams.
- **Layer 4.** Each of the three corpus widgets gets a property suite proved end-to-end via `sby`. Negative test: a deliberately-broken FIFO with an off-by-one error in `write_logic` produces a counterexample trace pointing at the bug.
- **Layer 5.** Out of scope for this design plan; addressed in a future plan if and when the layer is undertaken.

---

## 10 — Risks and open questions

**State-update extraction is structural pattern matching.** Layer 2's transition-graph extraction relies on recognizing the canonical `match q.state { ... } -> next_state` shape. Kernels that compute next state through a non-canonical path (e.g., assign each field of `D.state` separately based on different conditions) will be invisible to the analysis or will produce conservative "unknown target" edges. We need to either restrict the FSM idiom to the canonical pattern, or invest in a more sophisticated analysis. Initial recommendation: restrict, document, and produce a clear diagnostic when the analysis can't determine the FSM structure.

**Output computation analysis.** Outputs (Mealy vs. Moore distinction) are derived from `(state, input)` in the same kernel. Layer 2's reachability analysis only covers state-to-state transitions; output verification belongs in Layer 4. There's a useful intermediate layer — "this output is a pure function of state alone (Moore) vs. depends on input (Mealy)" — that we could surface as a diagnostic but is deferred until Layer 4 brings in the expression-level analysis machinery.

**State explosion in BMC (Layer 5).** A state struct with two 16-bit fields has 4 billion states. Bitblasted SMT formulas grow accordingly. This is the standard model-checking risk. Spec-level mitigations: state abstraction (collapsing payload values to a small symbolic alphabet for proof purposes), decomposition into sub-FSMs, k-induction for invariants that don't need the full state space.

**Liveness properties without fairness assumptions.** Pure liveness ("eventually X") is undecidable without bounded depth or fairness assumptions about the environment. SymbiYosys handles this via bounded liveness or k-induction with explicit `fairness` properties on inputs. Layer 4 must require either a bound or a fairness assumption — open question whether to default to one or require explicit choice.

**Diagram layout quality.** Auto-layout of state graphs is hard. Sugiyama handles DAGs nicely; cyclic FSMs (most of them) need force-directed or other techniques and the result can be ugly. Initial implementation aims for "good enough"; if quality is bad, fall back to dumping `dot` and asking the user to render with Graphviz manually.

**Composition with kernel-language-extensions.** Match guards (kernel-language-extensions §2.4) interact with Layer 2's non-determinism analysis: once guards land, "potentially non-deterministic transition" diagnostics become more important. The two design plans should ship in coordination — Layer 2 should be designed assuming guards exist, even if guards land first.

**Composition with auto-pipelining.** Pipelined FSMs are an active research area in HLS. Auto-pipelining (per `auto-pipelining-plan.md`) will eventually want to retime FSMs, but the recurrence on the state-DFF feedback edge bounds how much retiming is possible. The FSM analysis in Layer 2 should produce the transition graph in a form auto-pipelining can consume to compute recurrence-bounded II for FSMs that include compute on state transitions (e.g., a pipeline-stage-fronted FSM controlling a multi-cycle multiply).

**Naming convention for FSM-aware widgets.** Should there be a separate widget category in `rhdl-fpga` for FSM-shaped widgets, or should they continue to live in `core`/`fifo`/`stream`/etc. as today? Recommendation: keep them where they are. The FSM derive is metadata; it's not a structural reorganization.

**LLM eval harness for FSMs.** A specific evaluation: take a corpus of N widget kernels with known FSM structure, ask an LLM agent to propose modifications (add a state, refactor transitions, add an output), and measure whether the modifications preserve the static-analysis diagnostics and the formal-verification proofs. This is the "FSM-aware LLM workflow" that the layered tooling enables. Out of scope for this design plan; worth flagging as a measurement target once Layer 4 ships.

---

## 11 — Phasing summary

| Phase | Deliverable | Status | Depends on |
|---|---|---|---|
| 1 | `#[derive(Fsm)]` macro + `#[derive(FsmWidget)]` | **shipped 2026-04-29 (PR #2)** | Nothing |
| 2 | Static reachability + dead-state pass | **shipped 2026-04-29 (PR #2)** | Phase 1 |
| 3 | Auto-generated state diagrams (SVG + dot + JSON) | **shipped 2026-04-29 (PR #2)** | Phase 1, 2 |
| 4 | `#[fsm_properties(...)]` + SVA emission | **shipped 2026-04-29 (PR #2)** | Phase 1, 2 |
| 4b | `cargo rhdl prove` SymbiYosys driver + corpus proofs | not yet shipped | Phase 4 |
| 5 | Built-in bounded model checker | not yet shipped (research-grade) | Phase 4b (validates approach) |

Phases 1–4 shipped together as a single PR (`feat/fsm-architecture`).  The metadata surface is in place; the cargo subcommand that drives SymbiYosys end-to-end (Phase 4b) and the in-house BMC (Phase 5) remain as future tracks.

The first widget rewrites that opt into the FSM derives ship in `refactor/use-fsm-and-or-patterns` (post-merge of PR #2 and PR #3): `core::can_master` (CanField as the FSM enum, plus or-pattern collapse of three field-classification matches) and `core::one_wire_master` (OneWireState).  See the relevant CHANGELOG entries for the detailed before/after.

---

## 12 — References

[1] Mealy, G.H. *A Method for Synthesizing Sequential Circuits.* Bell System Technical Journal, 1955. — The foundational Mealy machine paper. Outputs depend on state and input.

[2] Moore, E.F. *Gedanken-Experiments on Sequential Machines.* Automata Studies, Princeton University Press, 1956. — The Moore-machine companion. Outputs depend on state alone.

[3] Cleaveland, R., and Hennessy, M. *Testing Equivalence as a Bisimulation Equivalence.* Formal Aspects of Computing, 1993. — Theoretical underpinning for FSM equivalence checking.

[4] IEEE 1800-2017. *SystemVerilog Language Reference Manual,* §16: SystemVerilog Assertions (SVA). — The canonical specification of `assert property`, `cover property`, `assume property`, `eventually`, etc.

[5] Wolf, C. *Yosys Open Synthesis Suite.* https://yosyshq.net/yosys/ — The Yosys ecosystem; SymbiYosys is the formal-verification frontend.

[6] Wolf, C., et al. *SymbiYosys.* https://symbiyosys.readthedocs.io/ — The formal-verification driver Layer 4 integrates with. Supports smtbmc, abc-pdr, multiple SMT solvers.

[7] Amaranth HDL Project. *Formal Verification.* https://amaranth-lang.org/docs/amaranth/latest/stdlib/formal.html — Python-language precedent for the same architectural pattern (declarative FSMs + SymbiYosys integration).

[8] SpinalHDL. *State Machine Library.* https://spinalhdl.github.io/SpinalDoc-RTD/master/SpinalHDL/Libraries/fsm.html — Scala-language precedent. Worth studying for the syntactic design choices we're *not* adopting (declarative DSL keywords).

[9] Skarman, F., and Gustafsson, O. *Spade: An Expression-Based HDL With Pipelines.* OSDA 2023. — Spade's `state` keyword is a closer cousin to RHDL's design philosophy than SpinalHDL's DSL approach.

[10] Bluespec System Verilog. Arvind, R.S. Nikhil. *Bluespec System Verilog: Efficient, Correct RTL from High-Level Specifications.* MEMOCODE 2004. — The atomic-action / scheduler approach as an alternative to explicit FSMs.

[11] de Moura, L., and Bjørner, N. *Z3: An Efficient SMT Solver.* TACAS 2008. — The SMT solver Layer 5 would integrate with.

[12] Niemetz, A., and Preiner, M. *Boolector: A Bit-Vector Decision Procedure.* SAT 2017. — Alternative SMT solver for bit-vector-heavy hardware.

[13] Cimatti, A., Clarke, E., Giunchiglia, F., and Roveri, M. *NUSMV: A New Symbolic Model Verifier.* CAV 1999. — Foundational symbolic model checker; useful reference for the BMC algorithm in Layer 5.

[14] Sugiyama, K., Tagawa, S., and Toda, M. *Methods for Visual Understanding of Hierarchical System Structures.* IEEE Transactions on Systems, Man, and Cybernetics, 1981. — The layered graph drawing algorithm Layer 3 would use for state diagrams.

[15] Basu, Samit. *RHDL: Rust as a Hardware Description Language.* LATTE '25, March 2025. — In-tree at `doc/latte25/latte.tex`. The kernel-as-pure-fn invariant that this design plan exploits is established here.

---

## 13 — Decisions captured

For the record (also reflected in `architecture.md` and `CLAUDE.md` once shipped):

- **The FSM surface is metadata, not new syntax.** `#[derive(Fsm)]` plus two attribute hints, not a DSL with new keywords. Keeps rust-analyzer working and LLM-friendly.
- **The state enum is the source of truth.** All metadata flows from the `Digital`-derived state enum; the widget struct and kernel just declare where it lives.
- **Reset handling stays where it is.** The existing "reset comes last" idiom in the kernel is unchanged. The FSM macro doesn't reorganize reset logic.
- **Static analysis is advisory by default.** Diagnostics are warnings unless the user opts into errors. Layer 2 must not break the existing widget corpus.
- **Formal verification ships via SymbiYosys (Layer 4) before any in-house BMC (Layer 5).** Reuse the open-source formal-verification ecosystem first; only build our own when we have empirical data on what's missing.
- **The SVA expression sublanguage is a strict subset of the kernel-accepted expression language.** No calls, no payloads-of-payloads, no recursion. Keeps Layer 5's symbolic execution tractable when it lands.
- **Diagrams are auto-generated from metadata.** The state-diagram artifact is a *consequence* of the FSM derive, not a separately-maintained file. Diagrams stay in sync with code by construction.
