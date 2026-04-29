# `#[derive(Fsm)]` and `#[derive(FsmWidget)]`

The FSM tooling is opt-in via two derive macros. Both are purely additive: they emit metadata trait impls without changing the kernel body or the widget's `Synchronous` impl.

## Why use the FSM macros at all?

Every protocol PHY in `serial_bus/`, every video formatter in `video/`, every register-mapped peripheral, and a substantial fraction of the widgets in `core/` are state machines.  You can write them as bare `match q.state { ... }` kernels — the compiler accepts that just fine.  So why opt into the macro layer?

Five concrete reasons:

### 1. Auto-generated state diagrams in your widget's rustdoc

Once a widget carries `#[derive(FsmWidget)]`, the diagram generator reads its variant table and the author-curated transition list and produces an inline-SVG state diagram embedded in the widget's API docs.  A reader (human or LLM) opens the docs and sees the FSM at a glance — initial state highlighted, terminal states flagged, self-loops drawn separately, the full transition graph visible without scrolling through the kernel body.  Per `CLAUDE.md` §12 rule 14, **every FSM-tagged widget must emit this diagram**; it's not optional decoration.

### 2. Static reachability and deadlock-candidate analysis

The Layer 2 analysis pass walks the transition graph and flags:

- **Unreachable states** — variants that no path from `Idle` can reach.  Catches "I added a variant but forgot to wire it up."
- **Deadlock candidates** — states with no outgoing edges that aren't marked `#[fsm_state(terminal)]`.  Catches "I forgot to wire the transition out of this state."
- **Self-loop saturation** — states whose only edge loops back to themselves.  Same root cause as the deadlock case but with a more specific diagnostic.

These run advisory by default; `#[fsm(strict)]` on the widget escalates them to compile errors.  No bare-`match` widget gets these checks for free.

### 3. SVA-property surface for formal verification

Layer 4's `#[fsm_properties(...)]` attribute lets you declare invariants, liveness goals, coverage points, and environment assumptions next to the kernel function — and `render_property_sva` turns them into SystemVerilog Assertions for a SymbiYosys-driven proof flow:

```rust
#[fsm_properties(
    invariant("state != State::Error", name = "no_error"),
    cover("state == State::Done"),
    liveness("state == State::Done", bound = 1024),
    assume("input.valid"),
)]
#[kernel]
pub fn my_machine(...) -> (Out, D) { /* ... */ }
```

This is the path to *proving* properties about your widget — "the FIFO write-pointer never wraps past the read-pointer absent overflow", "the CAN frame producer eventually reaches `Stop`" — that bare-`match` kernels can't tell the surrounding tooling about.

### 4. Structured metadata for LLM-driven workflows

The diagram generator emits not just SVG but a structured JSON form that an LLM agent can consume programmatically.  When an agent proposes a kernel modification (add a state, refactor transitions, insert a new output), it can:

- read the current FSM structure as JSON,
- propose changes,
- regenerate the JSON,
- diff against the original to verify the structural change is what was intended.

This is materially faster and more reliable than parsing match arms.  For LLM-assisted development of state-machine-shaped widgets, it's the killer feature.

### 5. Consistent vocabulary across the widget library

When every FSM-shaped widget in the tree carries the same metadata layer, contributors learn one pattern and apply it everywhere.  `serial_bus::can_master`, `serial_bus::i2c_master`, `serial_bus::ir_nec_rx`, `serial_bus::one_wire_master` — they all expose the same `FsmDescriptor` surface, the same `FSM_TRANSITIONS` const, the same diagram in the same place in their docs.  Reading a new FSM widget feels familiar even on first encounter.

## When NOT to use the FSM macros

Three cases where opting in is the wrong call:

- **The widget isn't actually a state machine.**  A pure dataflow kernel — say, a CRC accumulator that just shifts and XORs every cycle — has no meaningful "states" to enumerate.  Tagging its `bool` register as a state enum and pretending it's an FSM produces a useless diagram and zero analysis value.  The macros are designed for kernels whose behaviour is *driven by* an enum-typed control register.
- **The state space is unbounded or genuinely runtime.**  If your widget has `state: Bits<N>` for `N >= 8` and the "states" are individual integer values that flow through arithmetic, you don't have a finite state machine in the classical sense — you have a counter or accumulator.  The FSM tooling's value comes from a *small* (≤ ~32) finite enumeration of named states.  A 256-element transition table is neither readable nor useful.
- **The state-update logic doesn't follow the canonical idiom.**  The Layer 2 extraction pass recognises `let next = match q.state { ... }; d.state = next;`.  If the next-state is computed via field-by-field assignment to a `dont_care()`-built struct, or via nested case analysis through multiple variables, the extractor falls back to "could be any variant" and the deadlock check stops working.  You can still use the derive (you'll get the diagram and the formal-verification surface), but the analysis pass becomes less useful.  In that case the cost-benefit shifts toward a bare-`match` kernel with manually-written assertions.

The first case is the most common: just because a widget has a few states doesn't make it an FSM in the sense the macros target.  The two-state `Idle / Running` pattern in `can_master::CanState`, for example, is essentially a boolean — we tag the more interesting `CanField` enum (13 variants, real frame walk) as the FSM instead.  **When choosing which enum to tag, pick the one with the most semantic content.**

## State enum: `#[derive(Fsm)]`

Apply it alongside `#[derive(Digital)]` on the enum that names the FSM's states.

```rust
use rhdl::prelude::*;

#[derive(Fsm, Digital, PartialEq, Copy, Clone, Debug, Default)]
pub enum State {
    #[default]
    Idle,
    Running { counter: b8 },
    Done,
}
```

This emits an `impl FsmState for State` carrying:

- a static `&'static [FsmVariantDescriptor]` listing each variant in source order,
- the index of the initial variant (the one marked `#[default]`, or 0 if no `#[default]` is present),
- a `fsm_variant_index(&self)` method that maps a value back to its variant index.

### Per-variant decoration

Each variant can carry an optional `#[fsm_state(...)]` attribute with two recognised arguments:

- `label = "..."` — a human-readable label that the diagram renderer uses instead of the variant name.
- `terminal` — marks the variant as intentionally absorbing. The static-analysis pass skips it for the deadlock-candidate check.

```rust
#[derive(Fsm, Digital, PartialEq, Copy, Clone, Debug, Default)]
pub enum State {
    #[default]
    #[fsm_state(label = "idle, waiting for start")]
    Idle,
    #[fsm_state(label = "running, counter = {counter}")]
    Running { counter: b8 },
    #[fsm_state(label = "complete", terminal)]
    Done,
}
```

### Overriding the initial variant

If the FSM's initial state isn't the `#[default]` variant, override with `#[fsm(initial = "VariantName")]`:

```rust
#[derive(Fsm, Digital, PartialEq, Copy, Clone, Debug, Default)]
#[fsm(initial = "Running")]
pub enum State {
    #[default]   // serialisation default — but the FSM starts elsewhere
    Idle,
    Running,
    Done,
}
```

The macro verifies the named variant exists; if you typo the name, the macro fails to compile with a precise error.

## Widget struct: `#[derive(FsmWidget)]`

Apply it on the widget struct that owns the state DFF, alongside `#[derive(Synchronous, SynchronousDQ)]`. Two attribute arguments are required:

- `state_field = "..."` — the name of the field holding the state DFF.
- `state_enum = ...` — the enum type that the state DFF carries. Accepts either a path (`State`, `crate::states::State`) or a string literal (`"crate::states::State"`).

```rust
use rhdl::prelude::*;

#[derive(Synchronous, SynchronousDQ, FsmWidget)]
#[fsm(state_field = "state", state_enum = State)]
pub struct MyMachine {
    state: dff::DFF<State>,
    // ...
}
```

This emits an `impl FsmWidget for MyMachine` with a `fsm_descriptor()` associated function returning the widget's compiled [`FsmDescriptor`]. The descriptor is what the static-analysis pass and the diagram generator consume.

### Strict mode

Add `strict` to the attribute to escalate the static-analysis diagnostics from warnings to errors:

```rust
#[derive(Synchronous, SynchronousDQ, FsmWidget)]
#[fsm(state_field = "state", state_enum = State, strict)]
pub struct MyMachine { ... }
```

In strict mode any unreachable state or deadlock candidate becomes a build-breaking error rather than an advisory warning.

## What you don't have to write

Both derives are *metadata only*. They don't:

- generate kernel code,
- modify the widget's `Synchronous` impl,
- introduce any runtime cost beyond a `&'static` slice per FSM,
- require any new `#[kernel]`-internal syntax.

The kernel that walks `match q.state` looks exactly the same whether the FSM tooling is opted in or not. That's deliberate — see `fsm-architecture.md` §4.1 for the rationale.

[`FsmDescriptor`]: ../api/fsm.html
