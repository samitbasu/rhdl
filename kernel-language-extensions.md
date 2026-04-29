# Kernel Language Extensions — Design Spec

A specification for expanding the subset of Rust accepted inside `#[kernel]` functions. The current subset (documented in `CLAUDE.md` §4) is sufficient for hardware description but leaves significant Rust expressiveness unused. This document proposes specific extensions, organized by implementation cost, with per-feature lowering sketches and acceptance criteria.

This is the language-level companion to `auto-pipelining-plan.md` (compiler-pass-level work) and `widget-roadmap.md` (library-level work). All three are independently shippable parallel work streams.

---

## 1 — Goals and non-goals

**Goal.** Allow RHDL kernels to look more like ordinary Rust without expanding the set of synthesizable concepts. Every extension here must (a) preserve the property that "if it compiles, it's hardware" and (b) lower to the existing RHIF / RTL / NTL opcodes without introducing new fundamental hardware semantics. Extensions are sugar; the underlying model stays cycle-accurate, value-typed, and pure.

**Non-goal.** Closing the gap with full Rust. Heap allocation, dynamic dispatch, references with non-trivial lifetimes, recursive types, true IEEE 754 floats, and unsafe code are excluded by construction (see §6). The kernel will always be a strict subset.

**Why this matters.** A larger accepted subset means LLM-generated kernels need fewer stylistic translations to be valid. State machines written naturally in Rust — with `let-else`, `?`, range patterns, or-patterns — should compile as-is. Every desugaring the user has to do by hand is a place where an LLM (and a human) gets it wrong.

---

## 2 — Tier 1: Pure desugaring extensions

These are syntactic transformations performed in `rhdl-macro-core` or in the MIR-lowering pass. They reduce to constructs the IR already supports. No new RHIF opcodes; no new RTL/NTL passes.

### 2.1 `let-else`

Currently flagged as unsupported. Desugar:

```rust
let Some(x) = optional else { return default; };
// ↓
let x = match optional { Some(v) => v, None => return default };
```

**Lowering.** Pure AST transformation in the macro layer.
**Acceptance.** Existing kernel that uses `match Some/None`-then-bind works identically when rewritten with `let-else`. Test in `crates/rhdl/tests/binding.rs`.

### 2.2 ~~Or-patterns in match arms~~ — shipped 2026-04-29 (PR #3)

```rust
match state {
    State::A | State::B => handle_idle(),
    State::C => handle_busy(),
}
```

**Shipped:** `crates/rhdl-macro-core/src/kernel.rs::match_ex` flat-maps top-level or-patterns into one arm per alternative before the rest of the lowering pipeline runs.  Macro-layer transformation only; no IR change.  Emitted Verilog is byte-identical to the hand-written multi-arm form because RHIF `Case`'s `table: Vec<(CaseArgument, Slot)>` already permits multiple entries pointing at the same Slot.  Nested or-patterns inside tuple/struct/slice patterns (`(A | B, C)`) are caught by `pattern_has_nested_or` and rejected with a specific diagnostic that points at the manual distribution rewrite (`(A, C) | (B, C)`).  See `doc/book/src/kernels/match.md` for the user-facing documentation.

Original entry follows: currently each variant requires a separate arm. The desugaring expands to multiple arms with the same RHS, or — better — lowers to a discriminant-OR check in the existing `Case` opcode.

**Lowering.** RHIF `Case` already supports multiple discriminant values per branch (see `rhdl-core/src/rhif/spec.rs`). The macro layer needs to recognize the syntax and emit the corresponding `Case` arm with a multi-discriminant key. If the IR doesn't yet accept multi-discriminant keys, that's a small, well-isolated extension.
**Acceptance.** A state-machine kernel that previously required N redundant arms collapses to one arm with N alternatives, with byte-identical emitted Verilog.

### 2.3 Range patterns

```rust
match opcode {
    0x00..=0x1F => decode_arithmetic(opcode),
    0x80..=0xFF => decode_extended(opcode),
    _ => decode_other(opcode),
}
```

**Lowering.** Range pattern `lo..=hi` desugars to `x >= lo && x <= hi`. Composes with `Case` arms via guard expressions, or lowers to a chained `Select`.
**Acceptance.** An instruction-decoder kernel using ranges produces fewer NTL nodes than the equivalent multi-arm match, demonstrating that the lowering shares the comparator.

### 2.4 Match guards

```rust
match (state, count) {
    (State::Run, n) if n < N => keep_running(),
    (State::Run, _)           => stop(),
    _                         => idle(),
}
```

**Lowering.** Append the guard expression as an additional boolean to the arm's predicate. Lowers to `arm_match && guard ? rhs : next_arm`. Already representable as `Select` chains.
**Acceptance.** A guard that references a captured pattern binding compiles and produces correct simulation output. Reset-handling guards are particularly important and should be tested.

### 2.5 `@` bindings in patterns

```rust
match value {
    n @ 0..=15 => use_small(n),
    n          => use_large(n),
}
```

**Lowering.** `@` binding is a let-binding fused with a pattern. Pure macro-level transformation.
**Acceptance.** A kernel that uses `@` binding produces output identical to an equivalent kernel that does the let-binding manually.

### 2.6 Array destructuring patterns

```rust
let [a, b, c] = arr;
```

For fixed-size arrays. Slice rest patterns `[head, tail @ ..]` over fixed-size arrays are also tractable since `tail` has statically-known length.

**Lowering.** Array destructuring is a sequence of indexed loads. Already representable.
**Acceptance.** Equivalent to the corresponding `let a = arr[0]; let b = arr[1]; ...` form in emitted Verilog.

### 2.7 `?` operator on `Option` and `Result`

```rust
fn parse(buf: PacketBuf) -> Result<Header, ParseError> {
    let len = read_len(buf)?;
    let kind = read_kind(buf)?;
    Ok(Header { len, kind })
}
```

`Option`/`Result` already lower to RHIF's `Wrap` opcode. The `?` operator is sugar for `match x { Ok(v) => v, Err(e) => return Err(e.into()) }`.

**Lowering.** Macro-level desugaring. The `From`/`Into` conversion in the `Err` branch needs care — for the kernel subset, restrict `?` to cases where the error types match exactly (no `From::from` call), at least in Phase 1.
**Acceptance.** A kernel using `?` on `Option` produces identical simulation output to an equivalent `match`-and-`return` form. Test the kernel runs correctly when the early-return path is taken.

### 2.8 `for x in array` (value iteration)

```rust
for x in arr {
    sum = sum + x;
}
```

Currently the supported form is `for i in 0..N { ... arr[i] ... }`. Value iteration is a macro-level desugaring.

**Lowering.** Pure AST rewrite to `for i in 0..N { let x = arr[i]; ... }`.
**Acceptance.** Equivalent to indexed iteration in emitted Verilog.

### 2.9 Compile-time `assert!` / `static_assert`

```rust
const fn check_power_of_two(n: usize) -> bool { n != 0 && n & (n - 1) == 0 }

#[kernel]
fn widget<const N: usize>(...)
where rhdl::bits::W<N>: BitWidth,
{
    static_assert!(check_power_of_two(N), "N must be a power of two");
    ...
}
```

**Lowering.** Pure compile-time check; no hardware impact. Implementation reuses the existing `miette` diagnostic infrastructure for clear error messages.
**Acceptance.** A failed `static_assert` produces a compiler error with span pointing at the assertion. Successful asserts produce zero NTL nodes.

---

## 3 — Tier 2: Expressive additions (new IR or new methods)

These need real compiler work but are well-bounded.

### 3.1 More built-in methods on `Bits<N>` and `SignedBits<N>`

| Method | Maps to |
|---|---|
| `count_ones()` | popcount tree (logarithmic depth) |
| `count_zeros()` | popcount of `!self` |
| `leading_zeros()` | priority encoder of `self` reversed |
| `trailing_zeros()` | priority encoder of `self` |
| `reverse_bits()` | static wire permutation (zero gates) |
| `swap_bytes()` | static wire permutation (zero gates) |
| `rotate_left(n)`, `rotate_right(n)` | barrel shifter for runtime `n`; static wire permutation for const `n` |

Each is a self-contained lowering rule in `rhdl-core` plus a method on `rhdl-bits`. Several are dependencies of widgets in `widget-roadmap.md` (popcount for ECC, leading-zero count for floating-point normalization, reverse-bits for serial protocols and CRC reflect). Adding them centrally is strictly better than each widget rolling its own.

**Lowering.** New unary RHIF opcodes (`Popcount`, `Clz`, `Ctz`) lowering to NTL `Vector` reductions or new specialized NTL ops. `reverse_bits` and `swap_bytes` lower to NTL `Concat` with reordered wires (zero hardware cost — pure renaming).
**Acceptance.** Each method has Tier-1 unit tests in `rhdl-bits` (pure Rust correctness) and Tier-3 HDL snapshot tests in `rhdl-fpga` (verify the lowered Verilog).

### 3.2 Saturating arithmetic methods

```rust
let y = x.saturating_add(b8::MAX);
```

`Bits<N>::saturating_add()`, `saturating_sub()`, `saturating_mul()`. Currently the kernel uses 2's-complement wrapping. Saturation is essential for DSP — clipping samples instead of wrapping. Each lowers to a comparator + mux around the existing add/sub.

**Lowering.** Method on `Bits<N>` returning `Bits<N>`, recognized by the compiler and lowered to: compute the wide-result, check overflow, mux to MAX or MIN on overflow. No new IR opcode needed — composes existing `Binary`, `Select`, and `Cast`.
**Acceptance.** Test all four corner cases: positive overflow, negative overflow (signed), zero crossings.

### 3.3 Custom traits beyond `Digital` (compile-time monomorphized)

Today kernels can be generic over `T: Digital`. Letting users define marker traits — `trait Comparable: Digital` with associated functions — would let widgets be written generically over richer type families.

```rust
trait Comparable: Digital {
    fn compare(a: Self, b: Self) -> Ordering;
}

#[kernel]
fn min<T: Comparable>(a: T, b: T) -> T {
    match T::compare(a, b) { Ordering::Less => a, _ => b }
}
```

**Lowering.** Stays compile-time / monomorphized; never produces a `dyn Trait`. The `#[kernel]` macro performs type-directed dispatch at the same point Rust does.
**Acceptance.** A round-robin arbiter generic over a user's own `RequestPriority` trait compiles and simulates. No `dyn` object code is emitted.

### 3.4 Const arithmetic in generic positions

```rust
pub struct FIFO<const CAP: usize>
where rhdl::bits::W<{ CAP.next_power_of_two().ilog2() }>: BitWidth,
{
    ...
}
```

Stable Rust's `generic_const_exprs` is still nightly. RHDL's macro layer can pre-evaluate const expressions at the macro level and inject the resulting concrete constants into the type position. Very high payoff for any size-parameterized widget.

**Lowering.** `rhdl-macro-core` evaluates the const expression at macro time using a small const evaluator (or a stable subset of `evalexpr`, which is already a dependency). Emits the concrete constant in place. No effect on `rustc`'s view of generics.
**Acceptance.** A widget parameterized by capacity rather than address-bit-width compiles and produces the expected internal RAM size.

### 3.5 Closure desugaring to anonymous kernels

```rust
let mapper = |x: b8| -> b8 { x.wrapping_add(1) };
let mapped = stream.map(mapper);
```

Today you pass a kernel by name as `type Kernel = ...`. Closure-like syntax could be desugared at macro time into a synthetic `#[kernel] fn` and the surrounding type plumbing. Pure sugar — no new expressive power — but it makes `Map<T, S>::try_new::<...>()` ergonomics much friendlier.

**Lowering.** Macro-level rewrite. Restrictions: closure must be non-capturing (no environment), have explicit input and output types. Captured environment would require a state struct, which takes us out of the "pure desugaring" budget into Tier-3 territory.
**Acceptance.** A `stream::map` that previously required a named `#[kernel]` function compiles with an inline closure and produces identical simulation output.

### 3.6 Fixed-point Q-format type

```rust
let a: Q<4, 12> = ...;  // Q4.12 — 4 integer bits, 12 fractional bits
let b: Q<4, 12> = ...;
let c = a + b;          // Q5.12 (one extra integer bit for carry)
let d = a * b;          // Q8.24 (full-precision)
let e: Q<4, 12> = d.truncate();
```

Largely a library addition over `SignedBits<I + F>`, with a smarter `Mul` lowering that produces an extra-width intermediate and a documented `.truncate()` / `.saturate()` step. Mostly exists already in user space; would benefit from being canonicalized.

**Lowering.** The arithmetic operations on `Q<I, F>` lower to existing `SignedBits` arithmetic with a width-tracking type wrapper. The IR sees only the underlying bit operations.
**Acceptance.** A FIR filter widget written using `Q<I, F>` produces bit-identical output to the equivalent hand-written `SignedBits` form, with strictly cleaner source code.

---

## 4 — Tier 3: Research-grade

These are not quick wins but are worth scoping.

### 4.1 Minifloat / bfloat16 with hardware semantics

Not full IEEE 754. A fixed-precision variant with explicit hardware semantics: subnormal handling defined to flush-to-zero, NaN propagation defined as a single sentinel, round-to-nearest-even mandatory. Useful for ML inference accelerators on FPGAs. Reference: XLS (Google) has a minifloat library worth studying [3].

### 4.2 Slice patterns over fixed-size arrays beyond destructuring

```rust
match buf {
    [hdr_kind, len_lo, len_hi, payload @ ..] => parse(hdr_kind, len_lo, len_hi, payload),
    _ => malformed(),
}
```

Today the workaround is explicit indexing. The challenge is that `payload @ ..` needs a statically-known length, which is true for fixed-size arrays but the macro layer must compute it. Useful for stream-processing kernels.

### 4.3 Bounded `while-let`

```rust
let mut buf = packet.payload();
while let Some(byte) = buf.next() {
    process(byte);
}
```

A `while let` pattern with a compile-time-known iteration cap (the array length, in this case) would generalize the current `for i in 0..N` constraint. Useful for protocol parsers that want early termination. Adds compiler complexity around loop-bound inference and `break` semantics.

### 4.4 Try-into-error conversions

The Phase-1 `?` extension restricts to identical-error-type cases. A future extension would allow `From`/`Into` conversion in the `?`-error-path, matching standard Rust idioms. This requires letting trait-method calls participate in kernel lowering, which intersects with §3.3.

### 4.5 Iterator combinators on fixed-size arrays

`array.iter().fold(...)`, `array.iter().sum()`, `array.iter().enumerate()`, `array.iter().zip(other)`. Each desugars to a const-bounded `for` loop, but the rules for what counts as a synthesizable iterator chain need careful definition. Could be a compelling demo of "RHDL kernels read like ordinary Rust."

---

## 5 — Phasing

### Phase 1 — Pattern desugarings (one PR, ~2 weeks)

Bundle: `let-else`, or-patterns, range patterns, match guards, `@` bindings, array destructuring, `for x in array`, compile-time `assert`. All are macro-layer transformations; one mid-sized PR. Unblocks readable state-machine kernels for UART/SPI/I2C.

### Phase 2 — `Bits<N>` method library (one PR, ~2 weeks)

Bundle the bit-manipulation methods: `count_ones`, `count_zeros`, `leading_zeros`, `trailing_zeros`, `reverse_bits`, `swap_bytes`, `rotate_left`, `rotate_right`. Plus the saturating-arithmetic methods. Each is independently testable. Direct dependency of widgets in `widget-roadmap.md`.

### Phase 3 — `?` on Option / Result (one PR, ~1 week)

Restricted to identical-error-type cases. Builds on the existing `Wrap` opcode in RHIF.

### Phase 4 — Custom traits and const generic arithmetic (one PR each, ~3 weeks total)

Larger scope; touches the macro layer's type-directed dispatch and adds a const evaluator. High value for widget genericity.

### Phase 5 — Closure desugaring (~1 week)

Strictly non-capturing closures only. Ergonomic win for `stream::map`-style APIs.

### Phase 6+ — Tier 3 research items

Fixed-point type as a library before being a compiler feature. Minifloat after `Bits<N>` math is mature. Slice patterns and `while-let` after `for x in array` is in practice.

---

## 6 — What stays forbidden

These are not items deferred to a later phase; they are intentionally outside the language for hardware-modelling reasons. Document them clearly so users (and LLMs) don't waste time trying.

**References (`&`, `&mut`).** The kernel's value-only model is what makes pipelining and retiming sound; introducing aliasing would entangle the IR with lifetime analysis and break the "kernel is a pure function" invariant. Auto-pipelining (per `auto-pipelining-plan.md`) depends on this.

**Heap allocation, `Vec`, `Box`, `String`.** No hardware story; would require dynamic resource allocation that doesn't exist on FPGAs. Fixed-size arrays cover the legitimate use cases.

**`dyn Trait` / trait objects.** Dynamic dispatch needs runtime vtables. Static dispatch via monomorphization (which the Tier-2 custom traits would use) is fine.

**Recursive types.** `enum Tree { Leaf, Node(Box<Tree>) }` cannot have a finite hardware representation. Should produce a clear compiler error pointing at the recursion site.

**`unsafe`.** No legitimate use case in the kernel subset. Hardware-level type punning (`mem::transmute`) is unnecessary because the type system already exposes bit-level views via `Digital::bin()`.

**Floating-point IEEE 754.** The full IEEE spec — denormals, NaN propagation rules, rounding modes — requires hundreds of gates per op and dedicated FPU-like hardware. Worth doing eventually as a black-boxed library widget rather than as a kernel primitive. Tier-3 minifloat (§4.1) is the realistic compromise.

**Lambdas / closures with captures.** Captured environments would require closure structs. Non-capturing closures (Tier-2 §3.5) are pure sugar; capturing closures break the value-only model.

**Runtime-bounded iteration (`while`, `loop`).** Hardware needs known upper bounds on cycle counts to fit in finite gates. Const-bounded `for` and the proposed bounded `while-let` (§4.3) are the supported forms.

---

## 7 — Acceptance criteria for the spec as a whole

Each individual extension shipping satisfies CLAUDE.md's contract — code, tests at every applicable tier, documentation, validation artifacts, and a CHANGELOG entry — for both the language feature itself and at least one widget that uses it.

A Tier-1 / Tier-2 extension is "done" when:

1. The kernel-language change compiles and all existing kernel tests still pass.
2. New tests in `crates/rhdl/tests/` cover the new syntax — typically a positive test (well-formed kernel) and a negative test (kernel mis-using the feature, expecting a `miette` diagnostic).
3. At least one existing widget is rewritten to use the new feature, with the resulting Verilog identical to the pre-extension version (sanity check that no semantics changed).
4. The book gets a new `kernels/<feature>.md` chapter referenced from `doc/book/src/SUMMARY.md`.
5. `CLAUDE.md` §4 is updated to reflect the new allowed/forbidden state.

---

## 8 — References

[1] Basu, Samit. "RHDL: Rust as a Hardware Description Language." LATTE '25, March 2025. (`doc/latte25/latte.tex`.)

[2] *The Rust Reference* — pattern matching, range patterns, or-patterns, slice patterns. https://doc.rust-lang.org/reference/patterns.html

[3] XLS Project (Google). "Accelerated HW Synthesis." https://google.github.io/xls/ . The `xls/dslx/stdlib/float.x` minifloat library is a relevant reference for hardware-friendly floating-point semantics.

[4] Rust RFC 3137 — `let-else` statements. https://rust-lang.github.io/rfcs/3137-let-else.html

[5] Rust RFC 1492 — `..=` range patterns. https://rust-lang.github.io/rfcs/1492-dotdoteq-range-patterns.html

[6] Rust Tracking Issue 95228 — `generic_const_exprs`. https://github.com/rust-lang/rust/issues/76560 . The unstable feature whose stable equivalent we approximate with macro-time const evaluation.

[7] Skarman, F., Gustafsson, O. "Spade: An Expression-Based HDL With Pipelines." OSDA 2023. — Spade also takes a "Rust-like syntax, hardware-restricted subset" approach; a useful comparison for design choices around what to admit and what to forbid.

[8] Bluespec System Verilog. Arvind, R.S. Nikhil. "Bluespec System Verilog: Efficient, Correct RTL from High-Level Specifications." MEMOCODE 2004. — The atomic-action / scheduler approach to hardware DSL design, contrasted with RHDL's value-functional approach.

---

## 9 — Open questions

- **Granularity of CHANGELOG entries.** Is a per-extension CHANGELOG entry enough, or does each acceptance-test widget rewrite warrant its own?
- **Stability story.** Should new kernel-language features be opt-in via a `#[rhdl(features(...))]` attribute, with stabilization criteria, or accepted as base language?
- **Diagnostic quality.** A poorly-implemented `?` operator that produces "expected `Result`, found `_`" diagnostics is worse than not having `?` at all. Per-extension testing must include negative-path diagnostic snapshots via `expect_test`.
- **LLM eval harness.** Does the kernel-language extension move the needle on the LLM-assisted evaluation harness proposed in `rhdl-deep-dive.md` §3? Run a corpus pre- and post-extension to measure.
