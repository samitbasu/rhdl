# RHDL Build Narrative

This file is the *story* of how RHDL — and especially its widget library in `crates/rhdl-fpga` — has evolved. It is not a `git log`. It is the record of what was built, **why**, what we learned along the way, and what followed up that wasn't obvious from the diff.

If `git log` answers *what changed and when*, this CHANGELOG answers *what we were trying to do and what we discovered*. New widgets, design pivots, gotchas hit during development, and follow-up debt all belong here. PRs and routine refactors that don't change a load-bearing decision do not.

## How to use this file

- **Adding an entry is mandatory.** See `CLAUDE.md` §16 — every widget, fix, or design pivot must land with a CHANGELOG entry in the same commit.
- Entries are grouped by date (newest at the top) and organized as discrete stories. One widget = one entry, even if it took several commits.
- Each entry follows the template below. Skip a section only if it's genuinely empty (e.g., no follow-ups), not because writing it is annoying.
- Be honest about workarounds. If you used `.skip(!0)`, hard-coded a constant to dodge a framework limit, or marked a test `#[ignore]`, say so and add it to the Follow-ups list in `widget-roadmap.md`.

### Entry template

```markdown
## YYYY-MM-DD — <Widget or change name>

**Path:** `<source path>` (and example/doc/test paths if relevant)

**Why this, why now:** <one paragraph — what unblocks, what motivates, what consumer is asking>

**Design decisions:** <bullet list — the choices made and what was rejected and why>

**Surprises and gotchas:** <bullet list — what didn't work the first time, what RHDL or the framework did that we didn't expect, what we'd warn the next builder about>

**Validation:** <one or two sentences — which tiers of the CLAUDE.md contract are met, and any honest deviations>

**Follow-ups:** <bullet list — anything deferred; cross-link to widget-roadmap.md "Follow-ups" section if added there>
```

---

## 2026-04-29 — Tier-3 widget: SMBus / SBS host (timeout-enforced I²C wrapper) (#44)

**Path:** `crates/rhdl-fpga/src/serial_bus/smbus_host.rs`, `examples/smbus_host.rs`, `doc/smbus_host.md`, `vcd/smbus_host/`

**Why this, why now:** SMBus is electrically I²C with extra discipline rules — the most important being a 35 ms transaction-level timeout that lets a hung slave or stuck wire be detected and recovered.  Smart-battery-system (SBS) hosts (laptop / smartphone fuel-gauge stacks) require this watchdog or they wedge.  Building it as a thin shim over `I2cMaster` proves the composition story: protocol-discipline layers stack on top of physical-layer widgets without modification.

**Design decisions:**
- **Thin wrapper around `I2cMaster`** — no new bit-level FSM.  The widget owns a tick counter, an `in_flight` latch, and a `timed_out` latch; the I²C master owns the wire.  Clean separation.
- **Two const generics** — `DIV_W` (passed through to the inner I²C) and `T_W` (timeout-counter width).  At 100 MHz, 35 ms = 3.5 M cycles → `T_W = 22`.  Tests use `T_W = 16` for fast simulation.
- **Sticky `timeout` flag** — once set, stays high until the next `start`.  The host reads it with the next sample after `done`.
- **No FSM derive** — the state machinery is all in the inner `I2cMaster`.  The shim has only one boolean (`in_flight`) — promoting it to an enum + FSM derive would add ceremony without insight, exactly the negative case described in `doc/book/src/fsm/derive.md`.
- **`done` pulses on either normal completion or timeout** — gives the host a single edge to act on.  The `timeout` flag disambiguates.

**Surprises and gotchas:**
- **The inner I²C `done` and the outer `done_pulse` register are one cycle apart.**  When `q.i2c.done` fires, we clear `in_flight` and pulse `done_pulse` *next* cycle.  Tests inspect for `done` anywhere in the trace, not at a specific cycle, so this is invisible to the contract.
- **`q.tick >= t_max` is correct, `q.tick == t_max` would also be correct.**  Used `>=` so the timeout still fires if `t_max` is set very small relative to the I²C transaction length — defensive against operator error.

**Validation:** All five tiers.  Tier-1: idle no activity, normal-transaction-completes-without-timeout, timeout-fires-when-T_max-exceeded.  Tier-3 HDL snapshot length and Tier-5 VCD digest blessed.  Tier-4 RTL `iverilog` round-trip passes.

**Follow-ups:**
- **Clock-low timeout (`t_LOW:SEXT` = 25 ms)** — separate counter that ticks only while `scl_drive_low == false && sda_in == false` (slave holding SCL).  Mostly a copy of the existing tick counter with an extra gate.
- **PEC byte (CRC8 over the transaction)** — wraps the data byte through a CRC8 engine, appends the result.  `core::crc` already exists; needs polynomial parameterization.
- **SBS block-read protocol layer** — multi-byte transactions with length prefix.  Higher-level widget that strobes `start` repeatedly with auto-incremented register addresses.
- **Battery management state machine (#46)** is the natural composer.

---

## 2026-04-29 — Tier-3 widget: MIPI DBI Type B (8080 parallel) display driver (#43)

**Path:** `crates/rhdl-fpga/src/serial_bus/mipi_dbi_type_b.rs`, `examples/mipi_dbi_type_b.rs`, `doc/mipi_dbi_type_b.md`, `doc/mipi_dbi_type_b_fsm.md`, `vcd/mipi_dbi_type_b/`

**Why this, why now:** The parallel sibling of DBI Type C — same controllers (ST7735/ST7789/ILI9341/ILI9488/SSD1351/RA8875), same command sets, but byte-per-`/WR`-pulse instead of 8-SPI-clocks-per-byte.  Faster at the cost of 8 extra data pins; ships with the Type C widget so users can pick on a per-target basis.

**Design decisions:**
- **4-state FSM** — `Idle / Setup / WrLow / WrHigh`.  Setup gives data + D/C# time to settle before `/WR` falls; WrLow holds the active strobe; WrHigh enforces minimum pulse-high before the next byte may begin.  Tagged with `#[derive(Fsm, FsmWidget)]` from the start.
- **Strobe timings as `DbiBTimings<T_W>` struct** — three knobs (`t_setup`, `t_wr_pulse_low`, `t_wr_pulse_high`).  Same FPGA-cycle convention as every other timing-parameterized widget.
- **8-bit only, write-only** — covers ~95% of real-world use.  16-bit bus (`/WR` + `D[15:0]`) and the `/RD` read path deferred to v2.
- **`/RD` held high in v1** — exposed as an output so the host can wire it through; keeps the pad assignment stable when v2 ships.
- **No SPI master composed** — DBI-B is structurally different from DBI-C.  This is a fresh tiny FSM, not a shim over [`SpiMaster`].

**Surprises and gotchas:**
- **Data must be valid *before* `/WR` falls**, not coincident with it.  That's what the `Setup` state enforces — first cycle's `Idle → Setup` latches `data_reg` and `dc_n_reg`, then `t_setup` cycles pass before the strobe goes low.  Skipping `Setup` would violate setup-time on real silicon.
- **`busy` is computed combinationally** from `state != Idle`, the same trick as `MipiDbiTypeC`.  Saves a register without losing 1-cycle latency.

**Validation:** All five tiers.  Tier-1: idle releases strobes, byte completes, data appears on bus, command drives D/C# low, /WR pulse goes low then back high.  Tier-3 HDL snapshot length and Tier-5 VCD digest blessed.  Tier-4 RTL `iverilog` round-trip passes.  FSM descriptor round-trip test confirms 4 variants and `Idle` as initial.

**Follow-ups:**
- **16-bit bus mode** — change `data_reg` to `Bits<16>`, expose `d_o: Bits<16>`.  Mostly a generic-parameter change.
- **`/RD` read path** for controllers that support memory readback (parameter readout, status query).
- **Multi-byte autoincrement burst mode** — assert `/WR` once per byte while keeping `/CS` low across N bytes.  Useful for pixel-stream bursts; pairs naturally with a `fifo::synchronous` upstream.

---

## 2026-04-29 — Tier-3 widget: TI HDQ single-wire master (#45)

**Path:** `crates/rhdl-fpga/src/serial_bus/ti_hdq.rs`, `examples/ti_hdq.rs`, `doc/ti_hdq.md`, `doc/ti_hdq_fsm.md`, `vcd/ti_hdq/`

**Why this, why now:** TI's proprietary single-wire bus for the `bq2018x`/`bq3xxx` fuel-gauge ICs that ship in nearly every laptop and smartphone battery.  Structurally similar to 1-Wire (open-drain, time-encoded bits) but with different framing — every transaction starts with a *break* pulse instead of 1-Wire's reset-with-presence-detect handshake.  Built next-to-1-Wire deliberately so the differences show up cleanly side-by-side; future battery-management-system widgets (#46) will compose this.

**Design decisions:**
- **Three primitive ops** — `Break`, `WriteByte`, `ReadByte`.  The host sequences them: `Break → WriteByte (addr w/ MSB=R/W) → WriteByte (data)` for a write; `Break → WriteByte (addr) → ReadByte` for a read.  Mirrors the 1-Wire master's `Reset/WriteByte/ReadByte` shape so users moving between the two see one mental model.
- **Timings as `TiHdqTimings<T_W>` struct in *FPGA cycles*** — same convention as 1-Wire / I²C / DHT22.  Doc table covers both standard and HDQ16 (fast) modes.
- **8-state FSM** — `Idle / BreakLow / BreakRecover / WriteBitLow / WriteBitWait / ReadBitLow / ReadBitSample / Stop`.  `#[derive(Fsm, FsmWidget)]` from the start; `FSM_TRANSITIONS` const + `write_fsm_diagram_as_markdown` + rustdoc include per CLAUDE.md §12 rule 14.
- **Open-drain output pair** `(bus_oe, bus_out)` — identical contract to `OneWireMaster`.  Host wraps with `tristate::simple` at the pad.
- **Multi-byte transactions deferred to v2** — the host is responsible for sequencing `Break → addr → data`.  Rationale: the framing is trivial enough that a wrapper can sit on top of this primitive without touching the FSM.

**Surprises and gotchas:**
- **No 1-Wire-style presence latch.**  HDQ has no presence-pulse equivalent — the slave starts shifting bits immediately after the break, no separate handshake.  Removing the `presence_ok` register from the 1-Wire template made the `BreakLow → BreakRecover → Stop` path simpler than 1-Wire's `ResetLow → ResetSample → Stop`.

**Validation:** All five tiers per the contract.  Tier-1 unit tests (4): idle releases bus, Break completes, WriteByte completes, ReadByte captures expected zero pattern.  Tier-3 HDL snapshot length (14 776 chars) and Tier-5 VCD digest blessed.  Tier-4 `iverilog` round-trip passes.  FSM descriptor round-trip test confirms 8 variants and `Idle` as the initial state.

**Follow-ups:**
- **`TiHdqTransaction` wrapper widget** that takes `(addr, data, op_kind)` and handles the Break/WriteByte/[ReadByte | WriteByte] sequence on a single `start` strobe.  Drops user-facing complexity to one strobe per register access.
- **Multi-byte block-mode transactions** (HDQ supports back-to-back addr/data without re-break in fast mode) — only relevant once a real `bq` host driver lands.
- **Battery-management state machine** (#46) is the natural composer; this widget is its physical-layer dependency.

---

## 2026-04-29 — Complete the FSM-derive migration sweep across remaining serial_bus widgets

**Path:** `crates/rhdl-fpga/src/serial_bus/{half_spi_master,ws2812,dht22,lin_master,sent_rx}.rs` + matching examples + new `doc/<name>_fsm.md` files

**Why this, why now:** Closes the loop on the CLAUDE.md §12 rule 14 directive — every FSM-shaped widget in the tree now opts in.  Previous batch (PR #23) migrated `can_master`, `one_wire_master`, `i2c_master`, `ir_nec_rx`; this batch finishes with the remaining five FSM-shaped serial-bus widgets, plus an explicit "stays bare-match" decision for the three counter-driven widgets that don't have an enum-typed state register.

**Migrations:**

- ✅ **`half_spi_master`** — 4-state HalfSpiState (Idle / Write / Turnaround / Read), 7 transitions.  **Biggest or-pattern win in the sweep**: four output-mux matches (`cs_n`, `sclk`, `sdio_oe`, `busy`) had 15 redundant arms across them; collapse to 8 arms with or-patterns (`Write | Turnaround | Read => false` for `cs_n`, `Write | Read => q.phase` for `sclk`, etc.).  7 tests pass.
- ✅ **`ws2812`** — 3-state WsState (Idle / Sending / Latching), 5 transitions.  Two output matches collapse — `data_out` (`Idle | Latching => false`) and `busy` (`Sending | Latching => true`).  5 tests pass.
- ✅ **`dht22`** — 8-state Dht22State (Idle / StartLow / StartReleaseHigh / StartReleaseLow / AckLow / AckHigh / BitLow / BitHigh), 12 transitions including 4 timeout edges back to Idle.  No or-pattern wins — each state has a unique handler.  5 tests pass.
- ✅ **`lin_master`** — 10-state LinState (Idle + Break + 4 × {Send, Wait}-pairs), 10 transitions in linear progression.  No or-pattern wins.  4 tests pass.
- ✅ **`sent_rx`** — 2-state SentState (Idle / Collecting), 3 transitions.  Smallest FSM in the migration — included for completeness, the diagram is essentially "Idle ↔ Collecting".  6 tests pass.

**Explicitly NOT migrated** (correctly):
- ❌ **`spi_master`**, **`spi_slave`**, **`uart_rx`** — none of these carry an explicit enum-typed state register.  They're driven by phase counters / bit counters / shift registers.  Per the "When NOT to use the FSM macros" guidance in `doc/book/src/fsm/derive.md`, tagging counter-driven widgets as FSMs would produce useless diagrams and zero analysis value.  They stay bare-`match` widgets.
- ❌ **`uart`**, **`midi`**, **`uart_16550`** — these compose other widgets (the underlying `Uart`, `UartTx`, `UartRx` primitives) and don't have their own state enum.  The state machinery lives in the inner widgets.  `uart_16550` is a register-mapped wrapper — its kernel is a giant address decode mux, not a state walk.
- ❌ **`uart_tx`** — has 2 internal states but they're encoded as a `bool` (`sending`), not an enum.  Could be promoted to a 2-variant enum + FSM derive, but the readability win is marginal.  Tracked as a future tidy-up.

**Surprises and gotchas:**
- **The `#[doc(hidden)]` on `LinState`** — the original `LinState` was annotated `#[doc(hidden)]` to keep the public API surface minimal.  After adding `#[derive(Fsm)]` the enum's variants need to be visible enough for the diagram, but the tag is preserved (the macro is metadata, not a public-API reshape).  No conflict.
- **Per-variant labels for readability**.  Where the Rust identifier doesn't read naturally as a diagram label (e.g., `StartReleaseHigh` → `"start (release H)"`), the `#[fsm_state(label = "...")]` annotation is added.  Consistent across the sweep.

**Validation:** `cargo test --package rhdl-fpga --lib` continues to pass with the same 429+ count.  HDL emission length and VCD digests unchanged for every migrated widget — proof that adding the derives + the or-pattern collapses is byte-identical at the IR level.

**Follow-ups:**
- **Promote `uart_tx`'s 2-state `sending: bool` register to an `Fsm`-derived enum** as a small tidy-up.  Marginal readability win; not blocking.
- **Wire the RHIF extraction pass** so `FSM_TRANSITIONS` becomes derivable rather than author-curated.  Layer 2 is shipped (PR #2); the integration into the rustdoc emission pipeline is the missing piece.  Until then, the hand-rolled `FSM_TRANSITIONS` is the contract.
- **Future Tier-3+ widgets** that ship state machines should be FSM-tagged from day one — saves a re-migration round-trip.

---

## 2026-04-29 — CRITICAL: every FSM-tagged widget must emit + include its FSM diagram (CLAUDE.md §12 rule 14); migrate `i2c_master` and `ir_nec_rx`

**Path:** `CLAUDE.md` §12 (new rule 14), `crates/rhdl-fpga/src/doc.rs` (new `write_fsm_diagram_as_markdown` helper), `crates/rhdl-fpga/src/serial_bus/{can_master,one_wire_master,i2c_master,ir_nec_rx}.rs`, the four matching examples + `doc/<name>_fsm.md` files, `doc/book/src/fsm/derive.md`

**Why this, why now:** the FSM derive shipped in PR #2 is metadata-only — the *diagram* is the user-visible payoff.  Without a contractual requirement to emit and include it, widgets can carry the derive without surfacing the diagram, defeating the entire FSM track.  The new CLAUDE.md §12 rule 14 closes this: every `#[derive(FsmWidget)]` widget MUST author-curate a `FSM_TRANSITIONS` const, the example MUST call `write_fsm_diagram_as_markdown`, and the source MUST `include_str!` the resulting `doc/<name>_fsm.md` in its rustdoc.  This entry catches up the four widgets that already use the derive.

**Design decisions:**
- **Helper in `rhdl_fpga::doc`** — `write_fsm_diagram_as_markdown::<W: FsmWidget>(transitions, filename)` and `render_fsm_diagram_markdown<W>(transitions) -> String`.  Layered on top of the existing `rhdl::core::fsm::diagram::{build_fsm_diagram, render_fsm_svg}` infrastructure from PR #2.  Produces a self-contained `<p><svg>...</svg></p>` markdown fragment that drops directly into rustdoc via `include_str!`.
- **Author-curated `FSM_TRANSITIONS: &[Transition]` const** in each widget — until Layer 2's RHIF-extraction pass is wired into the rustdoc emission pipeline, the author records the transitions explicitly.  Indices match the source enum's declaration order.
- **Per-variant labels for diagram readability** — `#[fsm_state(label = "...")]` is added on every variant whose Rust identifier doesn't match the canonical spec terminology (e.g., `Sof` → `"SOF"`, `CrcDelim` → `"CRCDelim"`, `AckSlot` → `"ACK"`, `LeadingBurst` → `"lead burst"`).
- **Stub `doc/<name>_fsm.md` committed** — the source's `include_str!` requires the file to exist at build time, before the example regenerates it.
- **Or-pattern collapse where opportunity exists** — `i2c_master`'s `in_byte_phase` 7-arm match collapses to 2 arms; the AckAddr/AckData output paths share an arm.  These are textbook or-pattern wins per `kernel-language-extensions.md` §2.2 (PR #3).
- **Book chapter expansion** — `doc/book/src/fsm/derive.md` now opens with a "Why use the FSM macros at all?" section and a "When NOT to use the FSM macros" section.  The five reasons (auto-diagram, static analysis, SVA surface, LLM workflows, vocabulary consistency) and the three negative cases (not-a-state-machine, unbounded state space, non-canonical update logic) are the rationale future contributors / agents read first.

**Surprises and gotchas:**
- **`rhdl_fpga` can't depend on `rhdl_core` directly.**  Per `architecture.md` §2, widgets pull through the meta-crate.  The `Transition` and diagram types are imported as `rhdl::core::fsm::analysis::Transition` (since `rhdl::core` is the re-export of `rhdl_core`).  First batch of code that needed this path; recorded for future widget authors.
- **`include_str!` evaluates at build time, not at example-run time.**  Stubs first, regenerate later.  Same pattern as the existing `doc/<name>.md` waveform-trace files.

**Migration coverage:**
- ✅ `serial_bus::can_master` — 13-variant CanField FSM, 20 transitions including 4 self-loops, 7 tests pass.
- ✅ `serial_bus::one_wire_master` — 8-variant OneWireState, 12 transitions, 10 tests pass.
- ✅ `serial_bus::i2c_master` — 7-variant I2cState, 9 transitions, or-pattern collapse on `in_byte_phase` (7 arms → 2) and the AckAddr/AckData output arm (2 arms → 1), 5 tests pass.
- ✅ `serial_bus::ir_nec_rx` — 6-variant NecState, 10 transitions, 7 tests pass.

**Validation:** Full lib sweep passes (429+ tests).  HDL emission length and VCD digest unchanged for every migrated widget — proof that adding the derives + the or-pattern collapse is byte-identical at the IR level.

**Follow-ups:**
- **Migrate the remaining FSM-shaped Tier-3 widgets** as separate small batches: `dht22`, `half_spi_master`, `lin_master`, `sent_rx`, `spi_master`, `spi_slave`, `uart_rx`, `ws2812`.  Each is a self-contained mini-PR following the same template.  `half_spi_master` has the largest pending or-pattern win (14 collapsible arm RHSes).  `i2c_master`'s prior CHANGELOG entry explicitly noted "match with or-patterns is forbidden in `#[kernel]`" — that note is now historical.
- **Wire the RHIF extraction pass** so `FSM_TRANSITIONS` becomes derivable rather than author-curated.  Layer 2 is shipped (PR #2); the integration into the rustdoc emission pipeline is the missing piece.  Until then, the hand-rolled `FSM_TRANSITIONS` is the contract.
- **Auto-include FSM diagrams in `Descriptor::hdl_for(target).rustdoc()`** so the `#![doc = include_str!(...)]` boilerplate isn't needed in every widget source.  Touches the rustdoc machinery; orthogonal to the widget-by-widget migration.

---

## 2026-04-29 — Reorganise widget directories: `serial_bus/`, `video/`, `audio/`

**Path:** `crates/rhdl-fpga/src/{audio,serial_bus,video}/` (new), `crates/rhdl-fpga/src/core/` (slimmed), `architecture.md` (§4 update)

**Why this, why now:** `core/` had grown to ~40 widgets across heterogeneous domains.  The 24 widgets that are foundation primitives (DFFs, RAMs, counters, arithmetic, control) and the 19 widgets that drive off-chip peripherals (UART family, SPI, I²C, CAN, LIN, 1-Wire, video, audio) were uncomfortably mixed.  Splitting by *what kind of off-chip thing it talks to* makes the directory tree match how contributors think about the library.

**Design decisions:**

- **Three new top-level categories.**  `serial_bus/` (16 widgets), `video/` (3 widgets), `audio/` (1 widget — seedbed).  Per `architecture.md` §4 the threshold for a new category is "two widgets motivate it"; serial_bus and video clear that easily, and audio is added because future I²S / S/PDIF / AC'97 widgets are well-defined enough to anchor the category now.
- **`midi` lives in `serial_bus/`, not `audio/`.**  Its wire layer is essentially UART at 31250 baud — the structural shape is closer to the protocol-PHY family than to the audio family.  When MIDI grows a synth / sequencer companion, that companion goes in `audio/`.
- **`core/` keeps the foundation primitives only:** registers, RAMs, counters, control widgets (priority encoders, arbiters, debouncer, edge detector, pulse stretcher), computation (CRC, MAC, divider, popcount, leading_zeros, barrel_shifter, comparator), generic helpers (option, slice, constant, delay, one_hot), and generic output (PWM).  Anything that talks to an off-chip protocol has been moved out.
- **Cross-directory imports use `crate::core::`, not `super::`.**  For widgets in `serial_bus/` or `video/` that depend on foundation primitives, the import becomes `use crate::core::{dff, constant};`.  Sibling-only `super::` references are reserved for intra-category composition (e.g., `serial_bus::midi → serial_bus::uart::Uart`, `video::cga_rgbi → video::video_timing`).  This convention is documented in `architecture.md` §4.

**Surprises and gotchas:**

- **`git mv` preserves history cleanly when the file content barely changes.**  All 19 moves show as `R100`/`R99` renames in `git log --follow`, so blame and bisect keep working across the reorg.
- **Brace-form imports vs. path-form imports.**  Both `use rhdl_fpga::core::uart_rx::...;` and `use rhdl_fpga::{core::uart_rx, doc::write_svg_as_markdown};` appear in the example files; the sed rewrite needed both patterns.
- **The `include_str!` paths in widget rustdoc don't change.**  Each widget's source has `#![doc = include_str!("../../examples/<name>.rs")]` and `#![doc = include_str!("../../doc/<name>.md")]` — those are *two* levels up from `src/core/<name>.rs` and *also* two levels up from `src/serial_bus/<name>.rs` (depth from file to the package root is the same).  The macro paths transparently survive the move.

**Validation:**
- `cargo build --package rhdl-fpga`: clean (lib + examples + tests).
- `cargo test --package rhdl-fpga --lib`: 424 passed, 0 failed, 1 ignored — same numbers as before the reorg.  No HDL or VCD snapshot perturbed because no kernel logic changed.

**Follow-ups:**
- **Promote `tristate/` to be tagged as a co-category of `serial_bus/`** in the docs — it's the natural pairing for any open-drain protocol PHY (I²C, 1-Wire, half-SPI, CAN, LIN).  Not a structural move, just a doc cross-link.
- **Eventual `sensor/` category** if the corpus of analog-sensor protocols (DHT22, SENT, future SPI-attached IMUs / ADCs) grows beyond what fits naturally in `serial_bus/`.  For now they live in `serial_bus/` because their wire layer is the dominant concern.

---

## 2026-04-29 — Full 16550A register surface (`uart_16550`, supersedes `bus_uart`)

**Path:** `crates/rhdl-fpga/src/serial_bus/uart_16550.rs` (renamed from `bus_uart.rs`), `crates/rhdl-fpga/examples/uart_16550.rs`, `crates/rhdl-fpga/doc/uart_16550.md`, `crates/rhdl-fpga/vcd/uart_16550/`

**Why this, why now:** v1 of this widget shipped as `bus_uart` — a 2-register minimum-viable subset.  This v2 brings it up to the canonical 8-register PC16550D layout, which is what Linux `8250_core`, QEMU `hw/char/serial.c`, and every PC-derived firmware stack expects to talk to.  Software written against a real 16550A can probe-detect, read/write all eight registers in correct banks, route interrupts via IIR, drive RTS / DTR / OUT1 / OUT2, and self-test via loopback — without modification.  The rename ("bus_uart" → "uart_16550") makes the chip-family correspondence explicit so future readers don't have to guess at the layout.

**Design decisions:**

- **8-register layout exactly per the PC16550D datasheet** — RBR/THR (banked with DLL), IER (banked with DLM), IIR/FCR, LCR (with DLAB), MCR, LSR, MSR, SCR.  Bit positions match the datasheet so software is bit-compatible.
- **DLAB bank-switching implemented in the kernel** via a single decode against `(addr, q.lcr & LCR_DLAB)`.  Tested with `test_dlab_round_trip` writing distinct values to DLL (0x42) and DLM (0x13) and reading them back through the bank.
- **IIR with priority encoding** per the datasheet table (line-status > RX-data > THR-empty > modem-status > none).  `test_iir_priority_encoding` verifies the bits-1-3 encoding and the always-on `0xC0` FIFO-state field.
- **Loopback wired in the kernel** (MCR bit 4) — when set, the underlying UART's `tx` line drives its own `rx` input, and the four MCR output bits (DTR/RTS/OUT1/OUT2) drive the four MSR input bits internally.  This lets software self-test the entire data path without external wires.  Verified by `test_loopback_byte` round-tripping 0x5A through THR → loopback → RBR.
- **Modem-status delta bits** computed against a `prev_modem: dff::DFF<Bits<4>>` register.  CTS/DSR/DCD use straight delta; RI uses trailing-edge per the datasheet (DDCD-style "was set, now clear" semantics).  `test_msr_modem_inputs_visible` exercises the cts_n input pin → MSR.bit4 path.
- **Active-low modem pins at the I/O.**  Inputs `cts_n`, `dsr_n`, `ri_n`, `dcd_n` and outputs `rts_n`, `dtr_n`, `out1_n`, `out2_n` all carry `_n` in the name, follow the connector convention, and get inverted to active-high "asserted" semantics inside the kernel.
- **Break control** via LCR bit 6 — when set, the kernel forces the TX line to 0 regardless of what the underlying UART would output.  `test_break_control_drives_tx_low` verifies.

**Scope deferred to v3 (clearly documented in the rustdoc):**

- **Programmable word length / parity / stop bits** — the underlying `UartTx` and `UartRx` are hardcoded 8N1.  LCR's word-length / parity / stop fields are accepted into storage but don't yet alter the wire format.  Wiring them through requires extending the TX / RX primitives.
- **Programmable baud via DLL/DLM** — the actual divisor is fixed at construction; DLL/DLM are storage-only.  Same root cause: the underlying TX / RX take divisor as a `Constant`, not a runtime input.
- **Parity / framing / break-interrupt detection** — LSR bits 2/3/4 always read 0 because the underlying RX doesn't surface those error conditions.
- **FIFO clear on FCR write** — the underlying FIFO doesn't expose a clear input, so FCR.bit1 / .bit2 are accepted-and-ignored for now.
- **FIFO trigger levels** — FCR bits 6-7 are stored but the underlying FIFO has fixed triggering.

**Surprises and gotchas:**

- **Const-generic disambiguation in test helpers.**  A test helper `fn run_stream<const D: usize, const F: usize>(uut: &Uart16550<D, F>, ...)` compiled fine for the type parameter use, but the `where rhdl::bits::W<D>: BitWidth` bound parsed `D` as a type rather than a const.  Renamed to `DV` / `FW` to disambiguate.  The same pattern probably affects future test helpers parameterised over const-generic widgets.
- **The `include_str!` paths survived the rename.**  The widget points at `examples/uart_16550.rs` and `doc/uart_16550.md` — those got renamed at the same time, so there's no broken include after the move.

**Validation:** All 5 tiers, **12 tests pass** including 6 register-interface integration tests (DLAB round-trip, MCR drives outputs, MSR sees modem pins, loopback round-trips a byte, RX→RBR, break drives TX low) plus IIR priority encoding, no-irq idle, and the SCR scratchpad round-trip.  Tier 4 iverilog RTL clean.  Tier 5 VCD digest blessed.

**Follow-ups:**

- **Programmable baud rate via DLL/DLM.**  Requires extending `UartTx` and `UartRx` to take divisor as a runtime input rather than a `Constant<Bits<DIV_W>>`.  Probably ~80 LOC of TX/RX changes, then one line in `uart_16550` to wire `((q.dlm.raw() << 8) | q.dll.raw())` to the underlying divisor.
- **Programmable word length / parity / stop bits.**  Bigger lift — the TX shifter needs to count to a programmable bit count, the RX sampler needs the same, and parity has to be computed both directions.  Probably ~200 LOC across `UartTx` / `UartRx` plus the LCR-decode in `uart_16550`.
- **Parity / framing / break-interrupt detection.**  Falls out of programmable word length plus an explicit "rx_error: Bits<3>" output on `UartRx` covering parity/framing/break.  LSR bits 2/3/4 then carry these.
- **FIFO clear hooks.**  `SyncFIFO` needs a `clear` input.  Once that lands, FCR.bit1/.bit2 wire through trivially.
- **FIFO trigger levels.**  Less urgent — most software uses the default level.  Would require parameterising the underlying FIFO or wrapping it.
- **Optional: 16-byte FIFO depth at `FIFO_W=4`** is the canonical 16550A; we're already there with the existing `Uart::<DIV_W, 4>` instantiation.

---

## 2026-04-29 — Refactor `core::can_master` and `core::one_wire_master` to use FSM macros + or-patterns

**Path:** `crates/rhdl-fpga/src/core/can_master.rs`, `crates/rhdl-fpga/src/core/one_wire_master.rs`

**Why this, why now:** First two widget rewrites that opt into the FSM derives (PR #2) and the new top-level or-pattern syntax (PR #3).  The point of the refactor isn't behavioural — emitted Verilog is byte-identical to before — it's to validate that the new tooling holds up against real Tier-3 widgets and to demonstrate the readability win.

**Design decisions:**

- **`can_master`** — picked CanField (the 13-variant frame-walking enum) as the FSM-tagged enum, not CanState (the 2-variant Idle/Tx).  CanField is what the kernel matches on extensively; CanState is essentially a boolean.  The widget can only carry one FSM tag, so the choice is between "useful diagram + analysis on the field-walk" vs "trivial diagram on Idle/Tx".  The first wins easily.  Per-variant labels are added on the variants whose source name doesn't match the canonical CAN spec terminology — `Sof` → `"SOF"`, `CrcDelim` → `"CRCDelim"`, `AckSlot` → `"ACK"`, etc.
- **`one_wire_master`** — only one state DFF, so the choice is forced.  Per-variant labels expose the natural human-readable phase names (`"Reset (low)"`, `"Reset (sample)"`, `"Write (low)"`, `"Read (sample)"`) instead of the camel-case Rust identifiers.  This is exactly the case `#[fsm_state(label = "...")]` was designed for.
- **Or-pattern collapse — `can_master`.**  Three matches collapse:
  - `raw_bit`: 13 arms → 6 arms (4 dominant variants share one arm, 5 recessive variants share another, 4 keep their own arm because each computes a per-bit-index value).
  - `in_stuff_zone`: 9 arms (8 + wild) → 2 arms.
  - `crc_input_active`: 8 arms (7 + wild) → 2 arms.
  Net delete: ~25 lines of redundant arm boilerplate.
- **Or-pattern collapse — `one_wire_master`.**  `bus_oe` match: 4 arms (3 + wild) → 2 arms.  Smaller win in absolute terms but the kernel reads as "drive low whenever we're in any *Low state, else release," which is much closer to the actual semantics than the three-line per-arm form.
- **HDL snapshots not re-blessed.**  The `test_vlog_generation` length checks and `test_*_trace` VCD digests are unchanged — proof that the desugaring is byte-identical at the IR level.
- **Two new tests per widget** (`test_fsm_descriptor_round_trip`).  Walks the variant table emitted by `#[derive(Fsm)]` + `#[derive(FsmWidget)]` and verifies widget name, state-field name, state-var binding, variant count, per-variant labels, and initial-index — i.e., that the metadata the analysis pass and diagram renderer will read is exactly what the source enum says.

**Surprises and gotchas:**

- **Or-patterns inside the kernel feel natural** — once the syntax is allowed, the `Sof | Rtr | Ide | R0 => false` form reads better than the four separate arms ever did.  This wasn't a surprise so much as a confirmation of the original §2.2 motivation.
- **The 12-tuple ceiling for `Synchronous` derive** still bites here.  `can_master` is at 11 sub-circuit fields after the StuffState consolidation; if the FSM derive ever gains a `&'static [FsmDescriptor]` field on the widget itself (rather than the current associated-function form), the ceiling becomes load-bearing.  Tracked as a follow-up; the current associated-function design avoids the issue by not adding any DFF or sub-circuit.
- **The widget-name string the macro emits** uses the bare ident (`"CanMaster"`), not the fully-qualified path (`"rhdl_fpga::core::can_master::CanMaster"`).  Confirmed working via the round-trip test.  If two widgets ever share a name, the descriptor's widget_name field will collide; tracked as a future-iteration concern in `fsm-architecture.md` §10.

**Validation:**
- `can_master`: 7 tests pass (6 original + 1 new fsm-descriptor round-trip).  HDL emission length 26937 chars — unchanged from pre-refactor.  VCD digest unchanged.  iverilog RTL clean.
- `one_wire_master`: 10 tests pass (9 original + 1 new fsm-descriptor round-trip).  HDL emission length 16431 chars — unchanged.  VCD digest unchanged.  iverilog RTL clean.

**Follow-ups:**
- **Apply the same refactor to the rest of the FSM-shaped widget corpus.**  Top candidates: `i2c_master` (already documented as wanting or-patterns in its CHANGELOG entry), `lin_master`, `spi_master`, `spi_slave`, `sent_rx`, `ir_nec_rx`, `bus_uart`, `dht22`, `audio_pwm`, `midi`.  Each is a self-contained mini-PR.
- **Wire `cargo rhdl prove` through these widgets** once Phase 4b ships — the metadata is now in place to drive SymbiYosys against the can_master frame structure (e.g. "after a `start` strobe in `Idle`, the FSM eventually reaches `Stop`") and the one_wire_master timing invariants.
- **Auto-generated diagrams in the rustdoc.**  The diagram renderer is shipped (PR #2 Layer 3); the next step is wiring `Descriptor::fsm_diagram_svg()` into the existing rustdoc emission pipeline so a widget's docs page automatically shows its state diagram.  Tracked separately because it touches the rustdoc machinery.

---

## 2026-04-29 — FSM macro family + analysis + diagram + SVA-property surface (PR #2)

**Path:** `crates/rhdl-core/src/fsm/`, `crates/rhdl-macro-core/src/{fsm.rs,fsm_widget.rs,fsm_properties.rs}`, `crates/rhdl-macro/src/lib.rs`, `crates/rhdl/src/prelude.rs`, `crates/rhdl/tests/fsm.rs`, `doc/book/src/fsm/*.md`

**Why this, why now:** Lands the four-layer FSM design from `fsm-architecture.md` in one upstream-clean PR (intentionally skipping fork-local docs — this entry catches them up).  Strictly additive: no widget HDL snapshots perturbed, no IR layer or pass-trait family added, no kernel-as-pure-fn invariant relaxed.

**Design decisions:**
- **Metadata, not new syntax.**  `#[derive(Fsm)]` plus `#[fsm(...)]` / `#[fsm_state(...)]` helper attributes record metadata trait impls; the kernel body is unchanged.  Decision recorded in `fsm-architecture.md` §13 — keeps rust-analyzer working, keeps LLM-generated kernels portable.
- **`FsmWidget` is the second derive, not a generic.**  Tagging a widget struct with the state field + state enum produces an `FsmDescriptor`-returning helper, decoupling analysis/diagram tooling from the widget's concrete state-enum type.
- **Pure-function leaf for analysis.**  `fsm/analysis.rs` consumes a transition list + descriptor and emits diagnostics; `fsm/extraction.rs` walks RHIF and produces the transition list.  Two-stage architecture means the analysis is unit-testable without spinning up the compiler.
- **Three diagram formats from one layout pass.**  Inline SVG (rustdoc-friendly, no Graphviz dep), Graphviz `dot` (external tooling), structured JSON (LLM workflows).  Layered BFS layout from the initial variant.
- **Single `#[fsm_properties(...)]` attribute, not four.**  Composes `invariant`, `liveness`, `cover`, `assume` declarations in one place with named-call syntax.  Less surface area than four separate attribute macros while keeping the same expressive power.
- **Cargo subcommand deferred.**  The `cargo rhdl prove` driver that hands SVA off to SymbiYosys is Phase 4b — the metadata surface (this PR) ships now so any tooling can be built against it.

**What guarantee is preserved.**  Kernel-as-pure-fn (no kernel-body changes); type-safe matching (the analysis reads RHIF, doesn't transform it); the existing `Pass` trait architecture (no new passes registered into stage drivers — analysis is a leaf the user invokes explicitly).

**Surprises and gotchas:**
- **The 12-tuple ceiling for `Synchronous` derive** that bit `can_master` is now load-bearing for FSM widgets too — `FsmWidget` doesn't add fields, but a widget with an FSM is more likely to have many DFFs.  No fix this PR; tracked as a follow-up in `widget-roadmap.md`.
- **Raw-string delimiter conflict in SVG output** (`r#"...fill="#444"..."#`).  Bumped to `r##"..."##` because `"#` would otherwise close the raw-string early.  Worth a note for any future SVG-emitting code in the tree.
- **`TypedBits` discriminant decoding** (in `fsm/extraction.rs`) had to walk the bit slice manually since the public API doesn't expose the integer value directly for arbitrary kinds.  Sign-extension handled for both `Kind::Signed` and signed-discriminant `Kind::Enum`.

**Validation:** All 5 tiers, 62 tests pass — 23 unit tests in `rhdl-core::fsm::*`, 22 macro-snapshot tests in `rhdl-macro-core`, 17 end-to-end integration tests in `crates/rhdl/tests/fsm.rs`.  Existing widget HDL snapshots untouched (verified by spot-checking `core::dff`, `core::counter`, `core::pwm`).

**Follow-ups:**
- **Widget rewrites** — opt-in `#[derive(Fsm)]` and `#[derive(FsmWidget)]` on the FSM-shaped widget corpus.  First two land in `refactor/use-fsm-and-or-patterns` (`can_master`, `one_wire_master`); the rest follow as separate batches.
- **`cargo rhdl prove`** — the SymbiYosys driver subcommand that compiles the widget Verilog with SVA included, generates a `.sby` config, runs `sby`, and structures the counterexample trace.  Phase 4b in `fsm-architecture.md`.
- **In-kernel BMC** — Phase 5.  Aspirational; symbolic execution of the kernel function over `(state, input)` for K cycles via z3/boolector bindings.  6+ months of work; not committed.
- **Pattern-distribution for nested or-patterns inside state-construction** — orthogonal but the FSM analysis becomes richer once that lands (see `kernel-language-extensions.md` §2.2 follow-up).
- **Widget snapshot regression** — the FSM derives are zero-cost on existing widgets (no fields added), but if `Synchronous` derive ever changes its tuple layout, the FSM macros need to track it.
- **The 12-tuple ceiling for `Synchronous` derive** noted above — when the macro emits a real generated struct instead of a raw tuple, FSM widgets benefit too.

---

## 2026-04-29 — Top-level or-patterns in `#[kernel]` match arms (PR #3)

**Path:** `crates/rhdl-macro-core/src/kernel.rs` (`match_ex`, `pattern_has_nested_or`, `pat()`), `crates/rhdl-macro-core/src/expect/match_or_pattern.expect`, `crates/rhdl/tests/match_or.rs`, `doc/book/src/kernels/match.md`

**Why this, why now:** Lands `kernel-language-extensions.md` §2.2 — the first item from Phase 1 of the kernel-language-extensions plan.  Or-patterns are by far the highest-frequency pattern friction in FSM-style kernels (every protocol PHY has clusters of variants with the same body — see the `can_master::raw_bit` / `in_stuff_zone` / `crc_input_active` matches that this PR's companion refactor collapses).

**Design decisions:**
- **Macro-layer flat-map, not IR change.**  RHIF `Case`'s `table: Vec<(CaseArgument, Slot)>` already permits multiple entries pointing at the same Slot — the macro just emits one entry per alternative with the same target slot.  Equivalent Verilog at zero IR cost.
- **Top-level only.**  Nested or-patterns inside tuple/struct/slice patterns (`(A | B, C)`) are caught by `pattern_has_nested_or` and rejected with a specific diagnostic that points the user at the manual distribution rewrite (`(A, C) | (B, C)`).  Same restriction Spade and Bluespec ship with.
- **Existing helpers anticipated this.**  Three of the macro-layer pattern helpers (`pattern_has_bindings`, `rewrite_pattern_to_use_dont_care_for_bindings`, `add_scoped_binding`) already handled `Pat::Or` recursively from prior groundwork.  Only the dispatcher (`match_ex`) and the diagnostic in `pat()` needed updating.

**What guarantee is preserved.**  Kernel-as-pure-fn (purely a macro-layer transformation, no kernel-body semantics change); type-safe matching (Rust's own checker enforces same-bindings-same-types across alternatives before our macro sees the AST); exhaustiveness (the desugared form preserves arm coverage).

**Surprises and gotchas:**
- **`arm()` shortcut-routing for no-binding patterns.**  Patterns without bindings get routed through `rewrite_pattern_as_typed_bits`, which would silently emit invalid Rust for nested or-patterns like `(A | B, C)`.  The recursive `pattern_has_nested_or` check in `match_ex` catches this case before it reaches `arm()`.
- **The "Surprise" line in the `i2c_master` CHANGELOG entry** (saying or-patterns aren't supported) is now historical — kept as-is to record the prior state, but the surrounding context has shifted.

**Validation:** 54 macro-core tests pass (52 original + 2 new: `test_match_or_pattern` snapshot + `test_match_nested_or_pattern_rejected` negative).  5 integration tests in `crates/rhdl/tests/match_or.rs` covering enum or-patterns, three-alternative groups, and literal-value alternatives — each runs through both VM and iverilog round-trip.

**Follow-ups:**
- **IR-level multi-discriminant `CaseArgument`** — would compile each or-pattern to a single `Case` arm with `CaseArgument::Slots(Vec<Slot>)` instead of N arms with the same target.  More efficient but requires extending the RHIF spec; the macro-layer flat-map is fine for v1.
- **Nested or-patterns via pattern distribution.**  Tractable but combinatorial-explosion-prone at depth; not on the near-term roadmap.
- **Other Phase-1 pattern desugarings** from `kernel-language-extensions.md` §2.1–2.9 — `let-else`, range patterns, match guards, `@` bindings, array destructuring, `?` on Option, `for x in array`, compile-time `assert!`.  Each ships as its own PR per CLAUDE.md §11.1.

---

## 2026-04-29 — Bus-attached UART (16550A-style register interface, v1)

**Path:** `crates/rhdl-fpga/src/core/bus_uart.rs` (+ example, doc, vcd)

**Why this, why now:** Roadmap row #24 — Tier 3 protocol PHY. Wraps the shipped `core::uart` (#36) with a tiny memory-mapped register interface. This is the minimal viable subset of the Intel 16550A — enough for a soft-CPU SoC to do interrupt-driven serial I/O — without the full register-bit compatibility that Linux `8250_core` expects.

**Design decisions:**
- **Two registers, not the full 16550A.** v1 ships `DATA` (RW, 0x0) and `STATUS` (R, 0x1); reserves 0x2/0x3 for future LCR/IER. Full 16550A register bit-compatibility (DLL/DLM with DLAB bank-switch, IIR with priority-encoded interrupt sources, MSR, MCR, FCR, etc.) is at least 4–5× more code and is tracked as a v2 follow-up. The minimal layout fits in ~30 lines of C driver.
- **Wraps `core::uart` as a single sub-circuit field.** Pure-combinational kernel does address decoding, status assembly, and the read-data mux. No additional state. This is the reference example of how to compose an existing widget into a register-mapped one.
- **`tx_push = write_enable && addr == 0x0`** and **`rx_pop = read_enable && addr == 0x0`** — the inner UART's FIFO push/pop strobes are gated by the address decode. Means a write to STATUS or any unmapped address is silently ignored (which is the right semantics for a memory-mapped peripheral).
- **`Option<Bits<8>>` from `uart.rx_data` decoded via `match`** in the kernel: `Some(byte) → (byte, true), None → (0, false)`. The `rx_valid` flag goes into bit 7 of STATUS; the byte goes into the read mux. This is the canonical pattern for consuming `Option`-returning sub-circuits inside a kernel — first one in the tree to do it explicitly.
- **Single combined `irq`** (asserted while RX FIFO non-empty). The full IIR with TX-empty-vs-RX-ready-vs-line-status priority encoding is a v2 follow-up.

**Surprises and gotchas:**
- **Inner-kernel name resolution** — same `use uart_kernel as _;` pattern as `cga_rgbi` and `ntsc_composite`. The `#[kernel]` macro generates a reference to the sub-circuit's kernel function during expansion; without the import the name doesn't resolve. Adding to the §13 "common kernel-composition pattern" docs.
- **Status reads always return the current FIFO state, not a latched snapshot.** This means `read STATUS` and `read DATA` in successive cycles see consistent state, but a CPU doing a wide read or a multi-cycle bus transaction sees the FIFO as it advances. For this v1 scope it's fine; v2 with a CPU-side handshake will need wait-states or a status-latch.

**Validation:** All 5 tiers, 7 tests including: idle no-irq, STATUS reads `rx_empty=1, tx_full=0` after reset, TX wire toggles when host writes DATA, **bit-exact 0xA5 round-trip from RX wire → DATA register**. Tier 3 HDL emission length 58674 chars (substantially larger than other widgets — composing the FIFO'd UART balloons the synthesis); Tier 4 iverilog RTL clean; Tier 5 VCD digest blessed.

**Follow-ups:**
- **Full 16550A register layout** — DLL/DLM divisor-latch with DLAB bank-switch in LCR; IIR with priority-encoded interrupt sources; MSR (modem status); MCR (modem control); FCR (FIFO control / clear). Needed for Linux `8250_core` and QEMU `hw/char/serial.c` compatibility. Probably ~400 LOC of additional widget code; the natural reference is the QEMU implementation.
- **Programmable LCR** — word length (5/6/7/8), parity (none/even/odd/mark/space), stop bits (1/1.5/2). Each requires a small change to the underlying TX/RX pipelines.
- **Hardware handshake** (RTS/CTS/DTR/DSR/DCD/RI) — modem-status pads + modem-control register + status-change interrupt. Each pad is a 1-bit input/output; the bookkeeping is the work.
- **Loopback mode** (LCR bit 4) — internally connects TX → RX for self-test.
- **Break detect/generate** — host writes 0x40 to LCR to assert break; an extended low (longer than a frame) on RX is detected as a break-received status bit.
- **Status-latch / wait-state for multi-cycle bus** — current STATUS is "live"; a CPU on a slow bus or with asynchronous register access wants either a latched snapshot or a wait-state.

---

## 2026-04-29 — NTSC composite sync encoder (monochrome v1)

**Path:** `crates/rhdl-fpga/src/core/ntsc_composite.rs` (+ example, doc, vcd)

**Why this, why now:** Roadmap row #39 — Tier 3 video PHY. Composes the shipped `VideoTimingCore` into a 2-bit composite-video output that drives a standard composite monitor or capture device. Pairs with a $0.10 R-2R DAC (two FPGA pins → one video pin). Together with the CGA RGBI (#35), this gives RHDL the full "drive a VHS-era display" capability set.

**Design decisions:**
- **Monochrome only.** No color subcarrier, no colorburst, no chrominance modulation. A real NTSC color encoder needs a 3.579545 MHz colorburst phase-locked to the horizontal scan, gated into the back porch of each line, with chrominance quadrature-modulated by I/Q color-difference signals. That is at least 2× the LOC of this monochrome encoder and is tracked as a v2 follow-up.
- **2-bit output** that maps to the standard composite levels: `00` = sync tip (0 IRE), `01` = blank/black (7.5 IRE setup pedestal), `10`/`11` = picture luma. This is the minimum for valid composite output and is the cheapest DAC option (two FPGA pins + 2 resistors).
- **Simplified VSYNC** — v1 emits a single broad VSYNC pulse for the duration of `VideoTimingCore`'s vsync region, rather than the standard 9-line equalize/vsync/equalize sequence. Most "rough sync" capture equipment accepts this; broadcast-quality VSYNC is a v2 follow-up.
- **Black-pedestal gating** — `pic_sample = 00` is gated to `01` (blanking) during active. This is the right semantics: a real video signal has a 7.5 IRE setup pedestal, so "black" reads correctly through the receiver's blanking comparator. Without the gate, picture content of `00` would briefly look like a sync tip.
- **No interlace** — v1 emits a 262-line progressive frame ("240p"). NTSC is 525 lines interlaced; full 480i is a v2 follow-up that needs a field counter.

**Surprises and gotchas:** None — the widget is a tiny 4-way mux on top of `VideoTimingCore`. The `#![doc = ...]` and `use ... as _` boilerplate matched the established pattern from `cga_rgbi`. First widget in the tree where the kernel literally has zero own state.

**Validation:** All 5 tiers, 7 tests including: composite is `00` during HSYNC/VSYNC, composite is `01` during blanking (not active, not sync), composite passes `pic_sample = 11` through during active, `pic_sample = 00` is gated to `01` during active. Tier 3 HDL emission length 8090 chars; Tier 4 iverilog RTL clean; Tier 5 VCD digest blessed.

**Follow-ups:**
- **NTSC color encoder** — adds the 3.579545 MHz subcarrier generator, colorburst-gating logic (during the back porch of each line), and YIQ→QAM modulation of the chrominance. Probably ~400 LOC; the canonical reference is the Atari 2600 / Atari 8-bit "TIA" or the Sega Genesis VDP's composite output.
- **Standards-compliant VSYNC** with 6 equalizing pulses + 6 broad VSYNC pulses + 6 equalizing pulses (each at half-line frequency). Required for picky monitors and broadcast equipment.
- **480i interlace** — emits two fields per frame with a half-line offset; needs a field counter and a field-dependent VSYNC adjustment.
- **PAL variant** — 50 Hz, 625 lines, 4.43361875 MHz subcarrier, line-by-line colorburst phase alternation. Mostly the same skeleton with different timing constants; the "PAL switch" makes it more complex than NTSC.
- **Pixel-clock divider** — at the canonical 13.5 MHz pixel clock the FPGA needs either a PLL or an internal divider gating the timing-core advance.

---

## 2026-04-29 — CGA digital RGBI video (test-pattern v1)

**Path:** `crates/rhdl-fpga/src/core/cga_rgbi.rs` (+ example, doc, vcd)

**Why this, why now:** Roadmap row #35 — Tier 3 video PHY. Demonstrates that the shipped `core::video_timing::VideoTimingCore` composes cleanly into a per-format video widget. The natural next layer (framebuffer + character ROM + attribute decoder) is a separate concern.

**Design decisions:**
- **Test-pattern generator, not framebuffer.** Emits a 16-color RGBI pattern (4-pixel-wide bars cycling every 64 pixels) that exercises the full CGA palette. The framebuffer + character ROM + attribute byte decoder layer is the natural follow-up — but each is a self-contained widget that composes on top of this one (give us `pixel_x`, `pixel_y`, `active`, get back a RGBI value to gate). Keeping this widget thin makes that composition obvious.
- **Wraps `VideoTimingCore` as a single sub-circuit field.** Pure-combinational mapping from `(pixel_x, active)` to RGBI happens in the kernel; no additional state. Two-field widget total (timing core + the kernel is logic-only).
- **`cga_320x200_60hz()` constructor** with the canonical IBM CGA timings (h_total = 912, h_active_end = 640, h_sync = 668..768; v_total = 262, v_active_end = 200, v_sync = 224..230). Requires `HW >= 10` and `VW >= 9` to hold the literal values; this is enforced at instantiation by `bits()` saturation rather than the type system, but the docstring spells it out.
- **RGBI gated by `active`** so the widget's output is black during blanking — what real CGA monitors expect. (Without gating, the test pattern would also appear during the blanking interval, which is technically valid but visually wrong.)

**Surprises and gotchas:**
- **`bits<N>(value)` panics if `value >= 2^N`.** Hit it on the first run of the mini test (h_total=64 in HW=6, but Bits<6> max is 63). The error message is `assertion failed: value <= Bits::<N>::mask().raw()`, which doesn't immediately point at the bit-width-vs-value mismatch. **Lesson:** when picking const-generic widths for a wrapper widget, always allow at least one extra bit beyond the literal value. Bumped mini's HW to 7 to give headroom.
- **Power-of-2 bar width.** The test pattern divides the active scanline into 16 equal bars, but at the canonical CGA active=640 each bar would be 40 pixels — and 40 isn't a power of 2, so doing the divide cleanly inside a kernel needs either a divider widget or a small lookup. Punted by using a fixed 4-pixel-per-bar pattern that just cycles every 64 pixels (= 10 cycles across the canonical 640-pixel active region). Less visually clean but trivially synthesizable.
- **Re-importing the inner kernel function** (`use video_timing as video_timing_kernel`) was needed for the `#[kernel]` macro to find the sub-circuit's kernel during expansion. The `#[allow(unused_imports)]` is there because the actual reference is generated by the macro after type-checking. Other widgets that compose sub-kernels (e.g., MDA via `video_timing`) follow the same pattern.

**Validation:** All 5 tiers, 6 tests. Tier 2 includes `test_pattern_covers_all_16_colors` (sweep one full frame and verify every RGBI 4-bit code appears in `active` cycles), `test_blanking_zeros_rgbi` (RGBI is 0 outside `active`), and `test_hsync_and_vsync_pulse` (both sync pulses fire). Tier 3 HDL emission length 8884 chars; Tier 4 iverilog RTL clean; Tier 5 VCD digest blessed.

**Follow-ups:**
- **Framebuffer layer** — composes this widget with `core::ram` to hold the pixel data; map `(pixel_x, pixel_y)` to a RAM address, gate the read value with `active`. Both 320×200 4-color and 640×200 mono modes.
- **Character ROM + 80×25 text mode** — composes this widget with two `core::ram` instances (font ROM + text buffer); decode the IBM CGA attribute byte (foreground 4 bits + background 3 bits + blink 1 bit). The classic.
- **Composite-NTSC artifact-color path** — the famous mode-4-and-7 "16-color" output that drove the 8088 MPH demo. Adds the NTSC-encoder widget (#39) on top of this RGBI generator. Real implementation needs the colorburst alignment trick that Andrew Jenner documented.
- **Pixel-clock divider** — at the canonical 14.318 MHz pixel clock the FPGA needs either a PLL synthesizing exactly that clock or a divider that gates the timing-core advance. Currently the FPGA clock IS the pixel clock; both extensions are useful.
- **Configurable-width bar generator** — replace the fixed 4-pixel bar with `(active_width / 16)` for clean visual bars at any active width. Needs a divider; trivial once the per-resolution bar count is provided as a `Constant`.

---

## 2026-04-29 — SENT receiver (SAE J2716, framing-helper v1)

**Path:** `crates/rhdl-fpga/src/core/sent_rx.rs` (+ example, doc, vcd)

**Why this, why now:** Roadmap row #31 — Tier 3 protocol PHY. Closes out the third leg of the automotive sensor/actuator interface set (CAN + LIN + SENT). Niche compared to CAN, but increasingly common in modern OEM stacks for absolute-position, pressure, and temperature sensors (Melexis MLX90324, Allegro A1335, Infineon TLE5012B SENT mode).

**Design decisions:**
- **Framing helper, not full decoder.** v1 finds frame boundaries (sync pulses) and emits per-nibble timing measurements (`last_period`, `nibble_idx`, `nibble_strobe`); the host computes the nibble value from `(period / tick_period) - 12`. The trade-off is intentional — in-kernel division would either need a 28-deep iterative-subtract cascade or a 16-element threshold-lookup table; either is fine but adds code for the framing-helper use case where a soft-CPU running a tiny SENT decoder in firmware can do the math in microseconds. v2 follow-up tracks the in-kernel decode if the use case materializes.
- **2-state FSM (`Idle`, `Collecting`).** Each falling edge measures the period since the last falling edge and classifies it: long → sync (start frame), in-range → nibble (during Collecting), else → abandon. Counts to 8 nibbles after sync, then emits `valid` and returns to `Idle`. Compared to most other widgets the FSM is genuinely tiny because all the work is in period-classification logic.
- **`SentTimings<T_W>` struct** holds 4 thresholds (`t_nibble_min/max`, `t_sync_min/max`) — bundled into a single Constant. Brings the widget to 9 sub-circuit fields, well under the 12-tuple ceiling.
- **No CRC-4 validation** in v1. CRC nibble is captured as the 8th nibble strobe; host validates against the 6 data nibbles using the standard SAE J2716 polynomial `0x1D`. Same rationale as the in-kernel decode — easy to do in firmware, doesn't gate the framing.
- **No tick-period auto-calibration.** v1 takes pre-computed FPGA-cycle thresholds. The full SENT receiver auto-calibrates by measuring the sync pulse and back-computing `tick = sync_period / 56`. Same in-kernel-division concern; tracked as v2.
- Reset comes last (CLAUDE.md §12), forces FSM to `Idle`, clears `prev_in = true`, and clears all latched state.

**Surprises and gotchas:**
- **Off-by-one in `period` measurement.** The first run of the kernel reset `tick` to 0 on the falling-edge cycle and then started counting from 1 on the next cycle. So at the next falling edge, `q.tick` reads `period - 1`, not `period`. Fixed by `let period = q.tick + one_t;`. Caught by the `test_nibble_periods_match_input` test which checks the period for each nibble against `(12 + N) * tick_cycles` exactly. **Lesson:** edge-driven kernels with tick counters need a "is the count inclusive or exclusive of the edge cycle" convention, and it should be made explicit in a comment. Adding to the §13 troubleshooting doc.
- **`q.state` lookahead.** When checking `q.state == SentState::Collecting` inside the falling-edge block, `q.state` reflects the state *before* this cycle's edge, so the state set by the previous falling edge is what's visible. This is the right semantics — sync arms Collecting on cycle T, the next falling edge at T+k sees `q.state == Collecting`. Worth flagging because it's the kind of cross-cycle dependency that a casual reader assumes is an off-by-one.

**Validation:** All 5 tiers, 6 tests including idle (no spurious strobes), full-frame round-trip with 8 nibbles `[0..7]`, and a *bit-exact* per-nibble period match for nibbles `[0, 5, 10, 15, 3, 8, 12, 7]` — verifies the period measurement matches `(12 + N) * tick_cycles` for every nibble. Tier 3 HDL emission length 10563 chars; Tier 4 iverilog RTL clean; Tier 5 VCD digest blessed.

**Follow-ups:**
- **In-kernel nibble decode.** Add a `decoded_nibble: Bits<4>` output computed from `last_period` and `tick_period`. Either iterative-subtract over 16 cycles (multi-cycle) or 16-element threshold cascade (combinational, deeper LUT). Probably the cascade is better since it lands the value in the same cycle as `nibble_strobe`.
- **CRC-4 validation** (polynomial `0x1D`). Captures all 8 nibbles into a 32-bit shift register and validates the last nibble. Could compose with a parameterized `core::crc::CrcEngine`.
- **Auto-calibration.** Measure sync period, divide by 56 to recover `tick_period` in FPGA cycles, then use that as the basis for nibble thresholds. Closes the "host has to know the tick period in advance" gap. Same division concern as in-kernel nibble decode — a one-shot iterative-subtract is fine since it only happens once per frame.
- **Pause-pulse detection.** SENT's optional pause pulse (variable length after the CRC nibble) carries inter-frame status info; capture its length and emit a `pause_period` output.
- **Slow-channel decode.** SENT's status nibble carries per-frame slow-channel bits that, accumulated over many frames, form a longer slow-channel message. A separate `core::sent_slow_channel` widget would consume the status nibbles emitted by this widget.

---

## 2026-04-28 — NEC IR remote receiver

**Path:** `crates/rhdl-fpga/src/core/ir_nec_rx.rs` (+ example, doc, vcd)

**Why this, why now:** Roadmap row #30 (the receive half) — Tier 3 protocol PHY. The most-used consumer infrared protocol; covers the bulk of TVs, set-top boxes, fans, and simple AV remotes. Pairs with a $0.50 TSOP4838 / VS1838B 38 kHz IR receiver module (which strips the carrier so this widget sees a clean digital input). RC5 / RC6 receivers and the NEC transmitter are tracked as v2 follow-ups.

**Design decisions:**
- **NEC protocol only**, 32-bit codes (the typical address + ~address + command + ~command layout). No address/command split inside the widget — host masks `code` as needed. RC5 (Manchester, 14 bits) and RC6 (variable-length, longer leader) are different enough state-machine-wise that a separate widget per protocol beats a parameterized superset.
- **Receiver only**, no transmitter in v1. The TX side composes the existing `core::pwm` widget at 38 kHz with a small bit-pattern FSM; the bit pattern is identical to what this RX decodes, so it's a self-contained spinoff.
- **Edge-driven FSM with a per-state `tick` counter.** State transitions happen on rising/falling edges of `ir_in` (kernel keeps `prev_ir` to detect them); the duration measured between edges is compared against threshold fields in the `NecTimings` struct to classify burst length, leading-space type (data vs repeat), and bit value (0 vs 1).
- **6-state machine** (`Idle`, `LeadingBurst`, `LeadingSpace`, `DataBurst`, `DataSpace`, `FinalBurst`). Repeat-code detection lives entirely in `LeadingSpace`: a long high-period (~4.5 ms) → data frame; a short one (~2.25 ms) → `repeat_pulse` + back to `Idle`.
- **Bit-shift convention:** new bits shift into the LSB of `code_reg`. After 32 shifts, the first received bit (NEC sends MSB-first) sits at `code_reg[31]`. The host gets a code already in conventional MSB-first numeric layout.
- **Bundle-into-Constant** pattern again: 6 timing fields go into one `NecTimings` struct held in a single `Constant`. Brings the field count to 8 (well under the 12-tuple ceiling).
- Reset comes last (CLAUDE.md §12), forces FSM to `Idle`, clears `prev_ir = true`, and clears all latched state.

**Surprises and gotchas:**
- **NEC's "MSB-first wire, LSB-first sample-into-shifter" trick.** First-received bit is the MSB of the final code; my shifter pushes bits into the LSB and shifts left. After 32 shifts, the first-received bit is at MSB. This is the cleanest pattern for any MSB-first wire protocol — same idiom used by SPI, UART (LSB-first variant), and I2C — but it always feels backwards on first read. Test `test_decodes_data_frame` round-trips `0x12345678` to verify.
- **`prev_ir` initial value matters.** If it defaulted to `false`, the very first cycle would look like a falling edge and prematurely arm `LeadingBurst`. Used `dff::DFF::new(true)` to initialize idle-high. Same fix is needed for any edge-detected widget on a normally-high line.

**Validation:** All 5 tiers, 7 tests including: idle emits no pulses, full data-frame decode (round-trips `0x12345678`), repeat-code detection (no spurious data valid), short-burst rejection (frames shorter than `t_lead_burst_min` ignored). Tier 3 HDL emission length 14371 chars; Tier 4 iverilog RTL clean; Tier 5 VCD digest blessed.

**Follow-ups:**
- **NEC transmitter** (`IrNecTx`). Composes (22) `core::pwm` at 38 kHz with the same bit pattern this widget decodes, gated by an FSM that walks SOF → 32 bits → stop. ~150 LOC.
- **RC5 receiver** (Manchester-encoded, 14 bits at 1.778 ms per half-bit). Different state-machine shape — needs a Manchester-decode primitive.
- **RC6 receiver** (similar to RC5 with extensions and a longer leader). Same Manchester primitive.
- **Tolerance windows** — current widget uses bare-min thresholds (`t_lead_burst_min`, `t_data_zero_one_threshold`). Real-world remotes vary; a "min/max" pair per timing with explicit error-frame emission would be more robust. Not needed for v1 demos.
- **Address/command split helper.** Most NEC users want `(address, command)` not raw 32 bits; a small `core::ir_nec_decode` kernel that does the unpacking + the byte/inverse-byte validation would close the loop.

---

## 2026-04-28 — Dallas / Maxim 1-Wire master (single-byte v1)

**Path:** `crates/rhdl-fpga/src/core/one_wire_master.rs` (+ example, doc, vcd)

**Why this, why now:** Roadmap row #27 — Tier 3 protocol PHY. The third widget that exercises the open-drain (oe, out) tristate pattern after I2C and half-SPI; closes the "every electronics hobbyist has a DS18B20 in a drawer" use case. Designed so the same widget covers DS18B20 standard speed, the DS28E01 overdrive mode, and the DS2401 silicon-serial-number flow by varying the timings struct rather than changing the kernel.

**Design decisions:**
- **Three operations:** `Reset` (with presence-pulse latch), `WriteByte` (8 bits LSB-first), `ReadByte` (8 bits LSB-first). Each takes one `start` strobe; multi-byte transactions are sequenced by the host. Keeps the widget small; ROM-search algorithm and full DS18B20 command sequencing live above this layer.
- **Bus timings as a single `Constant<OneWireTimings<T_W>>` struct** — eight named fields (`t_rst_low`, `t_rst_sample`, `t_rst_total`, `t_w0`, `t_w1`, `t_read_low`, `t_read_sample`, `t_slot`) all in *FPGA cycles*, not microseconds. The user pre-scales. This bundling is what keeps the widget at 8 sub-circuit fields total, well under the 12-tuple `Synchronous` derive ceiling.
- **8-state FSM** (`Idle`, `ResetLow`, `ResetSample`, `WriteBitLow`, `WriteBitWait`, `ReadBitLow`, `ReadBitSample`, `Stop`). Single `tick: Bits<T_W>` counter increments by 1 each cycle; states transition when `tick` matches a timing-struct field. State-transition `tick = zero_t` resets are explicit.
- **`(bus_oe, bus_out)` open-drain pair** matching the I2C / half-SPI convention. `bus_out` is hardwired `false` because the master only ever pulls the line low; the host wraps with `tristate::simple` (or just gates an open-drain pad directly).
- **Read-bit shift register** — sampled bit captures into MSB (bit 7) of `data_reg`; right-shifted at end of each non-final slot. After 8 bits, the byte sits LSB-first at bit 0, which matches the wire convention. Same `data_reg` is used for writes, where the LSB drives the low-pulse-width selector and is right-shifted at end of each slot.
- Reset comes last (per CLAUDE.md §12), forces the FSM to `Idle` and clears `presence_ok`.

**Surprises and gotchas:**
- **`data_reg.into::<u128>()` doesn't exist** in `Bits<8>` — the conversion to u128 is via `.raw()`, not `.into()`. Used the same pattern as `spi_slave::tests::test_*` to be consistent. Worth a small docs PR to clarify the canonical Bits→primitive conversion in tests.
- **`include_str!` requires the `.md` to exist at build time**, not just at doc-generation time. Solved by writing a one-line stub `doc/one_wire_master.md` before first build, then letting the example overwrite it. This applies to every new widget; consider adding a "create the stub" line to the §9 widget-build workflow.

**Validation:** All 5 tiers, 9 tests including: idle releases bus, reset completes with presence-pulse latch, reset low pulse meets minimum duration (≥ `t_rst_low`), write byte completes, read byte captures expected value when bus held low. Tier 3 HDL emission length 16431 chars; Tier 4 iverilog RTL clean; Tier 5 VCD digest blessed.

**Follow-ups:**
- **CRC-8 polynomial `0x31` engine** — every 1-Wire device uses this CRC for ROM ID validation and EEPROM page integrity. Composes with the existing `core::crc::CrcEngine` parameterized over polynomial; would validate the CRC engine's polynomial flexibility.
- **ROM Search algorithm** — the binary-tree walk that enumerates every slave on a multi-device 1-Wire bus. Lives above this layer (uses Reset / Write / Read primitives) but worth a dedicated widget since the search-step state machine is ~100 LOC of its own.
- **Overdrive auto-switch** — DS18B20 supports both standard (~80 kbit/s) and overdrive (~640 kbit/s); the protocol to switch is "send overdrive ROM command at standard speed, then re-clock at overdrive timings." Would require swappable `Constant<OneWireTimings>` or a runtime-mux of two timing sets.
- **Parasitic-power strong pull-up** — DS18B20 in parasitic-power mode needs the master to actively drive the line *high* (not just release it) during temperature conversion, to provide power. v1 has no provision for this; would add a `strong_pullup` mode to `bus_out` and a `t_strong_pullup` timing field.
- **1-Wire slave** (`OneWireSlave`) — for FPGA emulation of a slave device. Different state machine (sample master pulse widths, respond to ROM commands).
- **DS18B20 driver layer** — composes (this widget) + (CRC-8) + (ROM search) into a "convert temperature, read scratchpad, return °C" black box. The natural demo.

---

## 2026-04-28 — CAN master (Classical CAN 2.0A, TX-only v1)

**Path:** `crates/rhdl-fpga/src/core/can_master.rs` (+ example, doc, vcd)

**Why this, why now:** Roadmap row #37 — Tier 3 protocol PHY. Wraps up the automotive-bus trifecta (LIN, MIDI, CAN) and is by far the most structurally complex of the three: a real frame producer with field-walking FSM, CRC-15 accumulator, and bit-stuffer all running off a divided-down CAN bit clock. With this, an FPGA driving a TJA1050 / MCP2551 / SN65HVD230 transceiver can transmit standard 11-bit frames onto a real CAN bus.

**Design decisions:**
- **TX-only v1.** Standard 11-bit ID, data frames, DLC 0..=8, CRC-15 polynomial `0x4599`, full bit stuffing, no ACK detection (drives the ACK slot recessive expecting some other node to dominate it). No receiver, no acceptance filter, no error counters, no bus-off, no SJA1000 register interface. Each of those is a self-contained v2 follow-up.
- **Frame-walking FSM keyed on a `CanField` enum** (`Sof / Id / Rtr / Ide / R0 / Dlc / Data / Crc / CrcDelim / AckSlot / AckDelim / Eof / Ifs`) plus a `field_bit_idx: Bits<7>` counter. The 7-bit width covers the 64-bit Data field. Field transitions handled per-variant in a giant `match` rather than computed; explicit but readable.
- **`StuffState` substruct** bundles `last_bit`, `run`, `pending` into one DFF — purely to stay under the 12-tuple ceiling that the `Synchronous` derive enforces (the natural decomposition would have been three separate DFFs, pushing the widget to 13 fields). The substruct is documented in its own `///` block with the rationale spelled out.
- **`total_data_bits = dlc * 8`** computed via a hand-rolled `match` on each DLC value rather than a runtime multiply or shift. Necessary because `as_bits::<7>()` defaults to `Bits<DIV_W>` inside a kernel (the as_bits-generic-default footgun, see DHT22 follow-up). Explicit lookup table is uglier than `dlc << 3` but actually compiles.
- **Position arithmetic in `Bits<7>`** (the source width of `field_bit_idx`) and shifting target registers (`Bits<11>`, `Bits<4>`, `Bits<64>`, `Bits<15>`) directly via the generic `Shr<Bits<M>> for Bits<N>` impl. Avoids the as_bits trap entirely. **This is now the canonical pattern for runtime bit selection inside RHDL kernels** — recorded as a footgun-avoiding idiom worth lifting into the docs.
- Reset comes last (per CLAUDE.md §12 rule), forces the FSM back to `Idle` and clears all latched state.

**Surprises and gotchas:**
- **`as_bits()` defaults to outer kernel's `DIV_W` const generic.** When you write `q.field_bit_idx.dyn_bits().resize::<11>().as_bits()` inside a kernel that's generic over `DIV_W`, the inferred width is `DIV_W`, not `11`. The compiler error is `cannot subtract Bits<DIV_W> from Bits<11>` and is genuinely confusing the first time. Workaround: either annotate the result type explicitly (`let x: Bits<11> = ...`) — but in some positions that still doesn't help — or restructure to never need width conversion at all by computing in the source width and using `Shr<Bits<M>>`. Already documented as a follow-up from DHT22; this widget reinforces that the second workaround is the more reliable one.
- **The 12-tuple ceiling for `Synchronous` derive.** The macro generates a `S` type that's a flat tuple of every field's state plus the `Q` type. With 13 sub-circuits, you get `cannot compare (..., ..., ..., ..., ..., ..., ..., ..., ..., ..., ..., ..., ..., ()) with itself` — `PartialEq` isn't derived for tuples beyond 12 elements. Fix: bundle related single-bit/few-bit DFFs into a substruct. Worth lifting the cap eventually (probably needs the macro to generate a real struct rather than a tuple).
- **CRC-15 must skip stuff bits.** The CRC accumulator updates on raw frame bits only (SOF + ID + control + DLC + data), NOT on stuffed bits. Easy to get wrong because the stuff bit *is* on the wire. The kernel gates the CRC update with the same condition as the field-advance branch (`!q.stuff.pending && crc_input_active`).

**Validation:** Tier 1 (functional behavioral checks: `test_idle_line_recessive`, `test_frame_starts_with_sof_dominant`); Tier 2 (`test_frame_completes` — drive a frame and verify the `done` pulse arrives); Tier 3 (HDL emission length 26937 chars); Tier 4 (`iverilog` RTL round-trip clean); Tier 5 (VCD digest). 6 tests, all passing. **No CRC-bitwise validation against a known-good frame yet** — that requires either porting a CAN model into the test harness or capturing a real-bus trace; recorded as a follow-up.

**Follow-ups:**
- **Bit-exact CRC validation.** Cross-check the emitted frame against a software CAN model (`canlib`, `python-can`, or hand-computed) for at least one or two test vectors with known CRCs. Until this lands, the CRC implementation is "structurally plausible" rather than "verified bit-correct."
- **CAN receiver** (`CanReceiver`). Sample the RX line, sync to SOF, decode the same field walk in reverse with bit destuffing, validate CRC, drive an ACK slot dominant.
- **29-bit extended ID** (CAN 2.0B). Adds an SRR + IDE = 1 + 18 more ID bits before the RTR — small extension to the field walk.
- **ACK slot detection.** Sample the bus during the ACK slot; if no node dominated it, raise an `ack_error` flag.
- **Programmable bit timing** (Sync / Prop / Phase1 / Phase2 segments per ISO 11898-1). Required for receive-side resync; not needed for v1 TX-only.
- **Error handling** (CRC error frame, form error, bit error, error-active / error-passive / bus-off counters).
- **SJA1000 / FlexCAN-style register interface** so a CPU can drive frames via memory-mapped registers rather than the current direct `(id, dlc, data, start)` ports.

---

## 2026-04-28 — Multi-bit handshake bridge (slow CDC)

**Path:** `crates/rhdl-fpga/src/cdc/slow_crosser.rs` (+ example, doc, vcd)

**Why this, why now:** Roadmap row #4 (Tier 0) — the only multi-bit CDC primitive currently in the tree is the Gray-coded `cross_counter` inside `AsyncFIFO`, which is specialized for monotonic counters. Anything else (config buses, status registers, command codes) had no path across clock domains. This is the textbook 4-phase req/ack handshake with single-bit synchronizers gating a stable W-domain data register sampled by R.

**Design decisions:**
- Hand-written `impl Circuit` (matching the `Sync1Bit` and `BitSyncChain` pattern) rather than a kernel-based composition. The data crossing primitive (W-domain wire sampled by R-domain register only after `req_sync_2` confirms stability) cannot be expressed with the framework's current type-system-enforced domain separation, so the widget directly implements `sim()` and `hdl()`.
- Two state machines, one per clock domain: source has `Idle / WaitForAck / WaitForAckClear`, destination has `Idle / WaitForReqClear`. Encoded as `Digital` enums (`SrcState`, `DstState`).
- `req` (W→R) and `ack` (R→W) each go through a 2-FF synchronizer chain in the destination domain. Documented; the metastability protection lives in those chains.
- Output struct carries signals from *both* domains (`data: Signal<T, R>`, `busy: Signal<bool, W>`) — verified that `#[derive(Timed, Digital)]` handles multi-domain output structs cleanly.
- `data_reg` is held stable from step 1 through step 5 of the handshake, so the destination samples it directly without per-bit synchronization. This is the standard CDC trick and saves `T::BITS` worth of flip-flops vs. naively chaining a sync per bit.

**Surprises and gotchas:**
- **vlog pretty-printer drops the trailing `;` after `wire [0:0] src_send;` specifically.** I lost ~30 minutes to this. Renaming `src_send` → `send_in` (and the corresponding wire) made the issue go away. Other identifiers using the same `wire [0:0] <name>;` form (`src_clock`, `src_reset`, `dst_clock`, `dst_reset`) printed correctly. I do not yet know whether `src_send` is somehow keyword-adjacent in the vlog grammar or whether this is a printer bug; recorded as a follow-up to investigate.
- The async testbench iverilog limitation strikes again — same `.skip(!0)` workaround as `Sync1Bit` and `BitSyncChain`. Cross-link to the existing follow-up in `widget-roadmap.md`.
- **Pattern recap for hand-written multi-domain widgets:** state struct holds *current* and *next* values for every register, plus the last-seen clock for each domain (for edge detection). The `sim()` body has three logical stages per call: pre-edge computation (when each clock is low, compute next values), reset overrides (force next values to safe defaults if reset is asserted), edge-triggered latching (copy next → current on each rising edge). Hard to get right the first time; the `Sync1Bit` source is the canonical reference.

**Validation:** Tier 2 (`test_crossings_arrive_in_order` — sample R-domain output on each negative-edge of `dst_clock` and verify the four sent values appear in order); Tier 3 (HDL length sanity check at 2522 chars); Tier 4 (`iverilog` elaboration via `.skip(!0)`); Tier 5 (VCD digest). 4 tests, all passing.

**Follow-ups:**
- **Investigate the `wire [0:0] src_send;` vlog pretty-printer issue.** Reproducible: revert the rename and the generated Verilog drops the trailing `;`. Fix is either in the vlog parser, the pretty-printer, or both. May affect other widgets that use `_send`-suffixed identifiers.
- **`r_data` ready-to-consume strobe** — current API doesn't tell the destination *when* a new value arrived (`data` always presents the latest, but a one-cycle "fresh" pulse on the R side would let consumers chain on it). Recorded for v2.
- **Throughput** — every crossing takes ~6–8 source cycles + ~6–8 destination cycles. For higher throughput, use `AsyncFIFO`. Documented in the widget rustdoc.

---

## 2026-04-28 — Multiply-accumulate (MAC) unit (unsigned)

**Path:** `crates/rhdl-fpga/src/core/mac.rs` (+ example, doc, vcd)

**Why this, why now:** Roadmap row #15 — DSP foundation primitive. Required for any FIR/IIR filter, signal-processing pipeline, or integer-arithmetic neural-net inference. Companion to the divider just shipped.

**Design decisions:**
- Single-cycle multiply-and-accumulate (no pipeline registers between multiply and add). Throughput is one MAC per cycle. Considered a 2-stage pipelined variant — rejected for v1 because the single-stage form is simpler and the wider critical path is acceptable at the small `N` typical for early DSP work. The pipelined variant becomes natural once `auto-pipelining-plan.md` lands.
- Full-precision intermediate via `DynBits::xmul` (the same primitive `dsp::lerp::fixed::lerp_unsigned` uses). Two `Bits<N>` operands give a `2N`-bit product, then `.resize::<A_W>().as_bits()` widens to the accumulator width. `A_W >= 2N` is documented; if smaller, single products overflow.
- Interface: `(a, b, enable, clear)` mirrors the CRC engine's pattern. `clear` overrides `enable`. The accumulator output is always present; consumers gate on their own message-end signal.
- **Unsigned only** for v1. Signed MAC is one of the most-requested DSP primitives and will need either `xmul` on signed `DynBits` (already available — see `lerp_signed`) or a dedicated `SignedMacUnit<N, A_W>`. Recorded as follow-up.

**Surprises and gotchas:** None — `DynBits::xmul` was a well-blazed trail thanks to `lerp`. The `dyn_bits()` → `xmul()` → `resize::<A_W>().as_bits()` idiom is worth lifting into a documented kernel pattern.

**Validation:** All five tiers, 9 tests including a 7-pair stream test against the software reference (`Σ a_i * b_i`) and a max-product test (`0xFF × 0xFF` = `0xFE01`, fits in 24-bit accumulator). `iverilog` RTL+NTL clean.

**Follow-ups:**
- **Signed MAC unit** (`SignedMacUnit<N, A_W>`). Composes `SignedBits::xmul` and uses signed accumulator addition.
- **Pipelined multi-cycle variant** for high-throughput at large `N` once `auto-pipelining-plan.md` ships.

---

## 2026-04-28 — Integer divider (unsigned, shift-subtract)

**Path:** `crates/rhdl-fpga/src/core/divider.rs` (+ example, doc, vcd)

**Why this, why now:** Roadmap row #14 — the Rust `/` and `%` operators do not synthesize in `#[kernel]`, so any project that needs runtime division (baud-rate generation, fixed-point scaling, address calculation) must instantiate this widget. First multi-cycle widget in this batch — most prior widgets (popcount, leading-zero count, barrel shifter, strict arbiter) were single-cycle combinational.

**Design decisions:**
- Multi-cycle restoring shift-subtract algorithm. Computes `N`-bit ÷ `N`-bit in `N` cycles after `start`. The classic textbook approach; minimum hardware footprint at the cost of latency.
- **No `N+1`-bit arithmetic.** The standard formulation needs an extra carry bit on the partial remainder. I avoid it by capturing the would-be carry (`rem`'s old MSB before the left shift) into a separate `rem_msb` signal, computing the comparison `(carry || new_rem) >= divisor` as `carry==1 || new_rem >= divisor`, and exploiting `N`-bit wrapping subtraction — `(2^N + new_rem_low) - divisor mod 2^N == new_rem_low - divisor` with wrap. The kernel uses only `Bits<N>` operations; there's no need to invent a wider intermediate type.
- Interface: `(dividend, divisor, start)` in, `(quotient, remainder, busy)` out. `start` is ignored while `busy`. Result held until next `start`. Considered a richer ready/valid handshake; rejected because `busy` is sufficient and keeps the example simple.
- Divide-by-zero is *not* trapped — the algorithm naturally produces `quotient = 2^N - 1`, `remainder = dividend`. Documented; callers should gate `start` on `divisor != 0` if they care.
- **Signed division deferred** — the unsigned core is the building block; signed version composes it with operand-sign-extraction and result-sign-correction. Recorded as roadmap follow-up.

**Surprises and gotchas:**
- The "carry-bit-without-N+1-bits" trick is well known to hardware designers but worth restating in the kernel comments because the algebra is non-obvious to a reader the first time. The CHANGELOG-as-narrative format is the right place to record *why* the design looks the way it does.
- This widget would benefit from `auto-pipelining-plan.md` once that lands — the per-cycle critical path is `compare → conditional-subtract → shift`, which on wide `N` will dominate clock period. Recorded as a future improvement.

**Validation:** All five tiers, 6 tests including a 56-pair grid sweep against the software reference (`u128 / u128`) and an explicit divide-by-zero test. `iverilog` RTL+NTL clean with default options.

**Follow-ups:**
- **Signed integer divider** (sign-detect, divide unsigned magnitudes, sign-correct quotient and remainder). Deferred from #14.
- **Pipelined variant** to meet higher clock frequencies once `auto-pipelining-plan.md` ships. Per-bit critical path is the limiting factor.

---

## 2026-04-28 — Barrel shifter

**Path:** `crates/rhdl-fpga/src/core/barrel_shifter.rs` (+ example, doc, vcd)

**Why this, why now:** Roadmap row #7 — variable-amount shifter / rotator. The built-in `Bits<N>::<<` and `>>` cover logical shifts but not arithmetic right shift (sign-extend) or rotates, which are the operations that actually need a named widget. Foundation for variable shifts in DSP, bit-field extraction, and the rotation half of round-robin/CRC routines.

**Design decisions:**
- Single unified kernel function with a `ShiftOp` enum that selects one of five modes (`LogicalLeft`, `LogicalRight`, `ArithmeticRight`, `RotateLeft`, `RotateRight`). Considered providing five separate kernel functions; the enum approach keeps callers from having to dispatch in their own code and makes the synthesizer free to share intermediate logic across modes.
- `amount` documented as `[0, N)`. Out-of-range amounts trip the kernel VM at simulation time and are undefined in synthesis. Did not add automatic mod-by-N — that would force a divider into the critical path; callers that need it can pre-reduce.
- ASR sign-extension implemented as `LSR | sign_extend_mask`, where the mask covers the top `amount` bits when the input MSB is 1. Considered using `SignedBits<N>` directly; rejected because the widget should work on `Bits<N>` and let the caller decide how to interpret the result.

**Surprises and gotchas:**
- **`if/else` in kernels lowers to a combinational mux — *both branches always evaluate*.** I initially wrote `if amount == 0 { data } else { (data << amount) | (data >> n_minus_amount) }` to handle the `amount == 0` rotate case, where `n_minus_amount = N` would otherwise trip the kernel VM's `shift < N` check. The unit tests (which call the kernel as a Rust function) passed, but `test_kernel_vm_and_verilog_synchronous` failed with "Shift amount 8_b4 must be less than 8". The fix: clamp the shift amount itself, not just the result. Compute `let safe_n_minus = if is_zero { bits(0) } else { n_minus_amount };` and use `safe_n_minus` everywhere a shift might otherwise be `N`. The output mux still picks the logically-correct value; the always-evaluated branch's shift now always uses a safe amount.
- **Lesson, generalized:** any time a kernel does `if guard { ... } else { expr_with_potentially_invalid_arg }`, you must ensure `expr_with_potentially_invalid_arg` is *valid for all inputs*, not just inputs that satisfy the guard. The if/else is just a mux on the result; both inputs flow through the hardware. This is an extension of the "Reset semantics belong at the end of the kernel" rule in CLAUDE.md §12.
- **Rust direct call vs kernel VM diverge on shift bounds.** Rust's `Bits<N> << k` is permissive (it wraps gracefully); the kernel VM is strict. So Tier-1 unit tests (Rust direct) can mask this class of bug — only the VM cross-validation catches it. Worth adding `test_kernel_vm_and_verilog_synchronous` to every combinational kernel that uses variable shifts.

**Validation:** All five tiers, 11 tests including a 280-input cross-validation sweep (7 data values × 8 amounts × 5 modes) through Verilog. Tier-1 unit tests cover identity at amount=0, swap-nibbles at amount=4 (rotate ROL/ROR symmetry), ROL∘ROR=identity round-trip, and exhaustive 8-bit × 8-amount sweeps for LSL and LSR against `u8::wrapping_shl/shr`.

**Follow-ups:**
- The if/else-evaluates-both-branches behavior should probably be called out explicitly in CLAUDE.md §4 ("The Subset of Rust That `#[kernel]` Accepts") so future agents don't have to rediscover it. Recorded as a documentation follow-up.

---

## 2026-04-28 — Leading-zero count

**Path:** `crates/rhdl-fpga/src/core/leading_zeros.rs` (+ example, doc, vcd)

**Why this, why now:** Roadmap row #9 — foundational primitive for fixed/floating-point normalization, dynamic-range estimation in DSP, and integer-to-float conversion. Companion to popcount (just shipped) and a thin variant of `priority_encoder_msb`.

**Design decisions:**
- Implemented inline rather than as a wrapper around `priority_encoder_msb`. Wrapping would have required a runtime `if input == 0 { N } else { N - 1 - msb }` post-step plus an `Option`-to-`Bits<W>` conversion in the kernel; doing it inline keeps the all-zeros special case cheap and the synthesized adder tree bounded.
- Pure `#[kernel]` function. Same parameterization shape as `popcount`: separate `N` (input width) and `W` (output width), user picks `W >= ceil(log2(N+1))`.

**Surprises and gotchas:** None — same loop pattern as priority encoder (MSB-first scan with `mut found` + `mut clz` accumulator). Validated exhaustively against `u8::leading_zeros()` (kernel) and `test_kernel_vm_and_verilog_synchronous` (Verilog), both 256-input sweeps.

**Validation:** All five tiers, 7 tests, Verilog cross-validation clean.

**Follow-ups:** None.

---

## 2026-04-28 — Population count (popcount)

**Path:** `crates/rhdl-fpga/src/core/popcount.rs` (+ example, doc, vcd)

**Why this, why now:** Roadmap row #8 — combinational primitive used by ECC syndrome weighting, hash-table sizing, normalization, and ML inference (binary neural net activation counts). Independent enough from other widgets to ship in any order, picked here because it's a one-screen kernel and good warmup for the longer combinational utilities (barrel shifter, leading-zero count) coming next.

**Design decisions:**
- Pure `#[kernel]` function — no struct, no Synchronous wrapper. Users that need it as a participating subcore can wrap with `Func` (see the example) or call it inline from another kernel.
- Two const generics: `N` (input width) and `W` (output width). The user picks `W >= ceil(log2(N+1))` so the maximum count is representable. Documented; not asserted (no compile-time arithmetic in stable const generics).
- Implemented as the unrolled "test-each-bit, conditional `+= 1`" loop. The synthesizer turns this into an adder tree. Rejected: pre-baked Wallace/Dadda trees — would have required hand-coded reduction tables and offered no advantage at the small input widths typical for RHDL kernels.

**Surprises and gotchas:**
- The `let one_w: Bits<W> = if bit_i != bits(0) { bits(1) } else { bits(0) };` cast pattern is the cleanest way to widen a 1-bit AND result to the accumulator width inside a kernel — `.resize::<W>()` and similar Rust-side methods are not (yet?) kernel-compatible for this kind of conditional widen-and-mux.
- Validated against `u8::count_ones()` exhaustively (256 inputs) at the kernel level *and* via `test_kernel_vm_and_verilog_synchronous` (256 inputs through Verilog).

**Validation:** All five tiers, 7 tests, Verilog cross-validation clean over the entire 8-bit input space.

**Follow-ups:** None.

---

## 2026-04-28 — Tier-3 batch 4: half-duplex SPI master + stereo PWM audio

#### Half-duplex / 3-wire SPI master — `core::half_spi_master`

Roadmap row #23.  The first widget in the tree that genuinely exercises the `tristate` design end-to-end via an `(sdio_oe, sdio_out)` pair the host wraps with `tristate::simple` at the pad.  State machine: `Idle → Write → Turnaround → Read`.  Runtime-configurable `write_bits`, `read_bits`, and `turnaround` per transaction (latched at `start`).  Mode 0 / MSB-first / 2 FPGA cycles per SPI bit, matching the existing `spi_master`.

Same widget covers both 3-wire (use `sdio_oe` to gate the pad) and 4-wire (treat `sdio_out` as MOSI, ignore `sdio_oe`, feed slave's MISO into `sdio_in`) — documented in the rustdoc.

**Surprise:** built a write-then-read round-trip Tier-2 test that drives a fake slave on `sdio_in` based on the master's exposed cycle timing.  First version had an off-by-one error: my `read_start` formula was `1 + 1 + 2*write_bits + turnaround`, but the actual Read state begins one cycle earlier — `1 + 2*write_bits + turnaround`.  The "extra +1" was me double-counting the start-cycle latency.  Caught when the rx pattern came out shifted right by one bit.  Lesson: when writing a stimulus that races the kernel's state machine, sketch out the cycle-by-cycle q.state transitions explicitly before computing offsets — don't reason from the SPI protocol's perspective.

7 tests, including three round-trips (8w/8r, 8w/8r with turnaround, 4w/4r), `iverilog` RTL+NTL clean.

#### Stereo PWM audio output — `core::audio_pwm`

Roadmap row #36 (naive PWM v1).  Two parallel `core::pwm::PwmGenerator` channels share a sample-rate divider and a per-channel sample register.  The host responds to `sample_request` pulses with the next `(left, right)` pair; the widget latches and holds them as the PWM duties for the next sample period.

**Sigma-delta noise-shaping deferred.**  Naive PWM is good for ~5–6 effective bits at moderate carrier rates (fine for hobbyist audio); CD-quality output needs a 1st/2nd-order modulator, which adds a signed-arithmetic accumulator (the `SignedBits<N>` ↔ `Bits<N>` conversions are still awkward in the kernel — see DHT22's earlier follow-up).  Recorded as a follow-up.

5 tests including a Tier-2 sample-cadence test that verifies `sample_request` pulses every `sample_period` cycles, plus a duty-latch test that observes the PWM output statistics shift from idle to (high left, low right) after the host starts feeding samples.  `iverilog` RTL+NTL clean.

---

## 2026-04-28 — Tier-3 batch 3: MIDI wire layer + Video timing core

#### MIDI interface — `core::midi`

Roadmap row #37 (wire layer v1).  Composes `core::uart` verbatim and adds a small `last_status` DFF that latches every received status byte (MSB=1).  Three outputs: the inner UART's TX/RX, plus an `is_status` flag and a held `last_status` value.  This is the substrate for downstream message-level parsing (Note On / SysEx / running-status etc.) — that FSM consumes the byte stream this widget exposes.  4 tests including a Tier-2 test that decodes a 0x90 (Note On status) byte and verifies `is_status` fires.

#### Video timing core — `core::video_timing`

Roadmap rows #32 (MDA), #33 (CGA), #34 (VGA) — *all three* covered by a single parameterized widget.  H/V counter pair plus four sync-region boundaries and two active-region ends (all runtime constants).  Reference timings for MDA, VGA 640×480, and VGA 800×600 are documented in the rustdoc table.  4 tests including an exhaustive sweep over a 10×4 mini-mode that verifies every cycle's hsync, vsync, and active outputs match the expected (x, y) → flags lookup.

The video core is the **sync-and-coordinate spine** of any video output widget.  Framebuffer, character ROM, palette LUT, and DAC drive all compose on top — those are mode-specific and deferred per-target (CGA framebuffer != VGA framebuffer != MDA framebuffer).  Shipping this one widget closes three roadmap rows because the frequently-shared part *is* the timing core.

**Surprise:** my first attempt at the struct used `#[derive(Default)]` because it has only DFF + Constant subcores.  But `Constant<T>` does not implement `Default` (it always needs a value), so `Default` doesn't derive cleanly.  Removed `Default` from the derive list; the explicit `new()` constructor stays.  Recorded as a follow-up to `core::constant`: optionally implement `Default` for `Constant<T>` when `T: Default`, which would let composing widgets keep `#[derive(Default)]` clean.

---

## 2026-04-28 — Tier-3 composition batch: full-duplex UART, LIN master

Two more Tier-3 widgets, both pure compositions of earlier work — the reusability dividend in action.

#### Full-duplex UART — `core::uart`

Roadmap row #18 closeout (the previously-deferred FIFO-buffered variant).  Pure dataflow composition: `tx_fifo` + `tx_uart` + `rx_uart` + `rx_fifo`.  Inputs: push to TX FIFO, pop from RX FIFO; the FIFOs decouple the host's clock-domain rate from the wire's baud rate.  4 tests including a Tier-2 round trip that drives an externally-encoded byte onto the RX line and verifies it shows up in the RX FIFO at the right cycle.

#### LIN bus master — `core::lin_master`

Roadmap row #28.  Single-byte v1.  Composes `core::uart_tx` for the byte-oriented sub-fields (sync, PID, data, checksum), adds a small FSM for the break field.  Computes PID parity (P0/P1) and classic checksum in the kernel.

**Surprise:** the kernel macro restricts turbofish to a small set of methods (`resize`, `xext`, `xshl`, `xshr`).  My first attempt at extracting `id_acc_8` used `q.id_reg.dyn_bits().resize::<8>().as_bits::<8>()` to widen `Bits<6>` to `Bits<8>` — `as_bits::<8>` was rejected.  Workaround: build the widened value bit-by-bit via a constant-bound loop:

```
let mut id_acc_8: Bits<8> = bits::<8>(0);
for k in 0..6 {
    let bit_k = (q.id_reg >> (k as u128)) & bits::<6>(1);
    if bit_k != bits::<6>(0) {
        id_acc_8 |= bits::<8>(1) << (k as u128);
    }
}
```

This is the third instance of "RHDL kernel doesn't accept the obvious type-cast" pattern (the others: `Bits<40> → Bits<16>` in DHT22, runtime-indexed array sizing in register file).  The pattern of "extract bits with a loop, then OR into the wider register" works around all three.  Recorded as a kernel-language-extensions follow-up — `Bits<N> → Bits<M>` with implicit zero/sign-extend is a clear ergonomic miss.

4 tests, `iverilog` clean.

---

## 2026-04-28 — Tier-3 protocol PHY batch (8 widgets)

A focused day of Tier-3 work. Lib test count: 275 → **346 passing** (0 regressions).

### Per-widget notes

#### PWM generator — `core::pwm`

Roadmap row #22.  Saw-tooth counter + comparator: `output = counter < duty`.  Period = `2^N` cycles; duty in `[0, 2^N - 1]`.  100% duty isn't representable (gate externally if needed).  10 tests including a Tier-2 test that runs each duty value through one full period and verifies the high-cycle count exactly matches the duty.

#### UART TX — `core::uart_tx`

Roadmap row #18 (TX half).  Standard 8-N-1, runtime divisor.  State machine: `Idle → Transmitting`, with a 4-bit `bit_counter` walking start (0) → data[0..=7] (1..=8) → stop (9).  The "compute current TX bit from `bit_counter`" path uses `bit_idx_safe = (bit_counter - 1) & 0b111` to mask the shift amount into `[0, 7]` so the always-evaluated mux input never trips the kernel-VM shift bound — same lesson as the barrel shifter.  12 tests including a round-trip decode that samples `tx` at the middle of each baud period and reconstructs the byte.

#### UART RX — `core::uart_rx`

Roadmap row #18 (RX half).  Mid-baud sampling for noise immunity.  Edge-detects falling start bit using a `prev_rx` register.  Shift register is 8 bits, sampled MSB-in so the LSB-first protocol naturally lands `data[0]` at the LSB after 8 samples.  6 tests including back-to-back multi-byte reception.  Documents the metastability requirement to externally `Sync1Bit` the `rx` line.

#### N-stage synchronizer chain (already shipped, not in this batch)
*(already in CHANGELOG above — this is just the batch's UART RX entry.)*

#### SPI master — `core::spi_master`

Roadmap row #19.  Mode 0 (CPOL=0, CPHA=0), MSB-first, 4-wire (`sclk`, `mosi`, `miso`, `cs_n`).  Two FPGA cycles per SPI bit.  Other modes / bit orders deferred — they're a small kernel change but the parameter explosion (`<W, CW, CPOL: bool, CPHA: bool, MSB_FIRST: bool>`) wasn't worth the v1 surface.  5 tests including a 6-pair round-trip with a simulated slave that drives MISO MSB-first.

#### SPI slave — `core::spi_slave`

Roadmap row #20.  Mirror of the master.  Samples external `sclk_in` on the FPGA clock and edge-detects (standard pattern when the SPI bus is much slower than FPGA clock).  Bidirectional: samples MOSI into `shift_rx`, drives MISO from `shift_tx` (latched at the falling edge of `cs_n_in`).  5 tests.

#### I2C master — `core::i2c_master`

Roadmap row #21 (write-only single-byte v1).  This is the first widget that exercises the `tristate` design end-to-end via `scl_drive_low` / `sda_drive_low` open-drain outputs (host wraps with `tristate::simple` at the pad).  4-phase per-bit timing (low setup, low hold, high sample, high hold) with each phase taking `divisor` FPGA cycles.  State machine: `Idle → Start → Addr → AckAddr → Data → AckData → Stop`.  5 tests.  **Surprise:** `match` with or-patterns (`A | B => ...`) is not supported in `#[kernel]`; had to expand into one arm per variant.  Recorded as a kernel-language-extensions follow-up — already on the list.

#### WS2812 / NeoPixel — `core::ws2812`

Roadmap row #26 (single-pixel v1).  Runtime-configurable timings (`t0_high`, `t1_high`, `bit_period`, `latch_period`) cover WS2812B, WS2811, SK6812 RGB by changing constants.  Sends a single 24-bit pixel per `send` strobe; multi-pixel chains are host-managed (strobe `send` per pixel in succession, then `latch` for the inter-frame idle).  5 tests including a Tier-2 test that records the data line, decodes per-bit pulse widths, and verifies the recovered pattern equals the sent pixel MSB-first.

#### DHT22 / AM2302 — `core::dht22`

Roadmap row #29.  Single-wire humidity/temperature sensor.  State machine: `Idle → StartLow → StartReleaseHigh → StartReleaseLow → AckLow → AckHigh → BitLow → BitHigh`.  The two `StartRelease*` states are split (rather than a single `StartRelease`) because the line is *still master-driven low* on the cycle the FSM exits `StartLow` — without an explicit "wait for high (line released)" step before "wait for low (sensor ACK)", the FSM races and treats its own master-low as the sensor's ACK.  Caught and fixed by the round-trip test.

**Surprise:** `Bits<40>::resize::<16>().as_bits()` does not give `Bits<16>` — the `as_bits` method's return type defaulted to the kernel's outer const generic (`CW`) instead of the requested `16`, and I couldn't find an annotation that pinned it.  Worked around by exposing the raw 40-bit `frame` in the output and letting the host mask `(frame >> 24) & 0xFFFF` for humidity etc.  Recorded as a kernel-type-inference follow-up.  5 tests.

### Cross-cutting observations from this batch

- **`match` with or-patterns is forbidden in `#[kernel]`** (CLAUDE.md §4 already lists it under "Forbidden" via the kernel-language-extensions reference).  Hit it twice — in `i2c_master` and again in some debugging.  Always expand to one arm per variant.  When refactoring shared bodies, accept the duplication or extract to a kernel function call.
- **State machines that include "wait for line release"** (DHT22, slow_crosser-style handshakes) need explicit two-step waits — first for the released-high state, then for the next driven-low state — to avoid racing against the master's own driven-low period.  Pattern: split the wait into `*_WaitHigh` and `*_WaitLow` states with a clean transition.
- **`if-else` inside the data path of a `match` arm** still lowers to a mux that always evaluates both branches.  Same gotcha as the barrel shifter: any operand that would be invalid (out-of-range shift, divide-by-zero) must be clamped at the operand level, not just guarded at the result level.

---

## 2026-04-28 — Generic memory-mapped register file

**Path:** `crates/rhdl-fpga/src/core/register_file.rs` (+ example, doc, vcd)

**Why this, why now:** Roadmap row #17. The existing `axi4lite::register::{single, bank, rom}` widgets couple register storage to the AXI4-Lite protocol. Building UART, SPI, I2C, etc. on top of those means each protocol PHY drags in an AXI dependency. This widget is the bus-agnostic register storage primitive — any bus adapter (AXI4-Lite, Wishbone, APB, custom) wraps it by translating its own `(read_addr, read_enable)` and `(write_addr, write_data, write_enable)` to the widget's flat input struct.

**Design decisions:**
- Combinational read + registered write semantics. Standard FPGA register-file model. Same-cycle read of an address being written returns the *old* value (documented).
- Outputs include `read_data` (combinational, from the read mux) AND `registers: [T; N]` (live view of every register). Adapters use the former; client logic that wants a specific register inline can pull from the latter without paying the read-mux delay.
- `read_enable` is passed through to a `read_valid` output (echoes input one cycle later — a common adapter pipelining pattern). Does not affect the data path.
- Three-parameter generic: `T` (data type), `N` (register count), `W` (address width). User picks `W >= ceil(log2(N))`. The widget does not enforce.
- Reset zeroes all registers via `T::default()`; `with_reset_values([T; N])` constructor exposes per-register reset values for use cases where defaults aren't appropriate (e.g. configured magic numbers in a status register).

**Surprises and gotchas:**
- **First implementation tripped RHDL's "Path .0.read_data is not covered" error.** I wrote `let mut read_data = T::dont_care(); for k in 0..N { if i.read_addr == bits(k) { read_data = q.regs[k]; } }` and then `o.read_data = read_data;`. Even though the assignment to `o.read_data` is unconditional, the kernel's coverage analyzer flagged it — likely because `T::dont_care()` for a generic `T` doesn't satisfy the field-coverage check. The fix turned out to be much simpler: **runtime array indexing**. `o.read_data = q.regs[i.read_addr];` lowers to an N-input mux on `read_addr` directly, no mut-local accumulator needed. RHDL handles `[T; N][Bits<W>]` cleanly per CLAUDE.md §4 ("Indexing arrays with constant or runtime indices").
- **Lesson, generalized:** for a "select one of N elements based on a runtime index", prefer direct array indexing `arr[idx]` over a `for`-loop-with-conditional-assignment. The compiler synthesizes the same hardware (an N-input mux) but the indexing form satisfies the coverage analyzer cleanly. The loop form is still correct for *local* mut accumulators (priority encoder, popcount), just not for struct fields.
- **Synchronous derive's bound on T.** Since `SynchronousIO::Kernel` propagates the kernel's `T: Default` constraint, the parent struct also needs `T: Digital + Default` in its definition, not just the constructor `impl`. Took one cycle to spot.

**Validation:** All five tiers, 10 tests including a write-then-read sequence verifying `0xA0..0xA3` land in addresses `0..3`, a concurrent-read-write-same-address test confirming old-value semantics, and `iverilog` RTL+NTL clean.

**Follow-ups:**
- **Per-register read-only flag** so adapters can refuse writes to specific addresses without external logic.
- **Optional registered read** (1-cycle latency, higher fmax) for designs where the combinational read is the critical path. Composes with the `delay::Delay<T, 1>` widget at the call site for now.

---

## 2026-04-28 — Wide-bus comparator

**Path:** `crates/rhdl-fpga/src/core/comparator.rs` (+ example, doc, vcd)

**Why this, why now:** Roadmap row #10 (Tier 1). Closing a Tier 1/2 gap left over from the second batch. The built-in `Bits<N>::==` and `<` already cover the bit-level work, but a named widget that emits all five comparison flags at once is useful as an arbiter/scheduler subblock and as a clear reference point for callers building wider or signed variants.

**Design decisions:**
- Pure `#[kernel]` function (no struct, no state). Caller wraps with `Func` if needed at the boundary.
- Returns a `Flags { eq, lt, le, gt, ge }` struct. Considered five separate function variants (`eq_kernel`, `lt_kernel`, ...) — rejected because a caller wanting more than one flag would compute `a < b` and `a == b` twice; emitting all five at once shares the underlying compare.
- Implementation: derive `eq` and `lt` from primitives, then `le = lt || eq`, `gt = !lt && !eq`, `ge = !lt`. The synthesizer should de-duplicate.
- **Unsigned only.** Signed variant (`SignedBits<N>`) deferred — needs sign-bit XOR-and-flip and is enough of a separate algorithm to warrant its own kernel.

**Surprises and gotchas:** None. Validates exhaustively against Rust's `==/<` over 256 4-bit pairs, both at the kernel level and through `test_kernel_vm_and_verilog_synchronous`.

**Validation:** All five tiers, 8 tests, Verilog cross-validation clean.

**Follow-ups:** Signed comparator variant.

---

## 2026-04-28 — PWM generator

**Path:** `crates/rhdl-fpga/src/core/pwm.rs` (+ example, doc, vcd)

**Why this, why now:** Roadmap row #22 (Tier 3). First Tier-3 protocol-ish widget: a saw-tooth counter feeding a single comparator. Useful immediately for LED dimming, motor control, and as a building block for more complex modulation schemes.

**Design decisions:**
- Single `N` const generic for both period (= `2^N` cycles) and duty width. Keeps the API minimal at the cost of forcing period and duty to scale together.
- `duty = 0` is "always low"; `duty = 2^N - 1` is the closest representable to 100% (high for `(2^N - 1) / 2^N` of cycles). Exact 100% is *not* representable — documented; gate externally if needed.
- Duty input is sampled combinationally each cycle; mid-period duty changes take effect immediately (the next comparison). Documented; for glitch-free duty changes the caller registers the duty externally.

**Surprises and gotchas:** None. The Tier-2 stream test exercises six duty values and checks the high-cycle count per period matches the duty *exactly* — a useful invariant test for any future re-implementation.

**Validation:** All five tiers, 10 tests, `iverilog` clean.

**Follow-ups:** Center-aligned PWM (triangle counter instead of saw-tooth) for motor-control applications that prefer symmetric switching.

---

## 2026-04-28 — Strict-priority arbiter

**Path:** `crates/rhdl-fpga/src/core/strict_priority_arbiter.rs` (+ example, doc, vcd)

**Why this, why now:** Roadmap row #13 — the trivial variant of the round-robin arbiter, useful as a fixed-priority interrupt controller, exception-ranking primitive, and a deliberately *unfair* baseline against which to test fair arbiters. Shipped immediately after `RoundRobinArbiter` so the two have matching I/O signatures and are drop-in swappable.

**Design decisions:**
- I/O signature (`Bits<N>` → `Option<Bits<W>>`) deliberately mirrors `RoundRobinArbiter` so the two are interchangeable.
- Implemented as an *empty-struct* Synchronous widget with no DFFs and no subcores. The kernel just calls `priority_encoder_lsb` and returns its result. This was a small experiment: I wanted to know whether an empty Synchronous struct would derive correctly. It does — `#[derive(Synchronous, SynchronousDQ)]` on a struct with zero fields produces `Q { } / D { }`, the kernel takes `_q: Q<N, W>` and returns `D::<N, W>::dont_care()`, and the framework synthesizes the expected zero-state Verilog. This is a useful pattern for any combinational widget that needs to participate as a Synchronous subcore in a higher-level composition.
- Kept rejected: making this a pure `#[kernel]` function (would have required users to add their own `Func` wrapper at every use site, breaking the swappability with `RoundRobinArbiter`).

**Surprises and gotchas:** Empty-struct Synchronous widgets work cleanly. Tier 4 (`iverilog` RTL+NTL) passes with the default test bench options — no `.skip(...)` workaround needed because there's no DFF, no non-zero reset value, and no async domain crossing.

**Validation:** All five tiers, 7 tests, `iverilog` RTL+NTL clean. Tier 2 includes a starvation test (constant `0b0101` for 16 cycles → bit 2 *never* gets a grant) that doubles as the load-bearing demonstration of why round-robin exists.

**Follow-ups:** None.

---

## 2026-04-28 — First eight widgets (`feat/widget-roadmap-batch-1`)

A single-day batch advancing through the recommended-first-eight list in `widget-roadmap.md` (rows 1–8 of "Recommended first eight"). The batch was a deliberate AI-assisted shakedown of the full CLAUDE.md contract: every widget got rustdoc with schematic + internals diagrams, runnable example, committed waveform, and Tier 1–5 tests. The lib test count grew from **149 → 224 passing** with **0 regressions**.

### Cross-cutting observations from the batch

These showed up in multiple widgets and shape what we'd do differently next time:

- **`q.<subcore>` semantics depend on whether the subcore has internal state.** For purely combinational subcores (e.g. `Constant`), `q.<field>` reflects same-cycle output; for pipelined subcores (e.g. `DFF`), `q.<field>` is one cycle behind `d.<field>`. The debouncer composition initially failed because the composer assumed `q.settle` (a `PulseStretcher` output, which sits behind a DFF) was same-cycle — see the debouncer entry below for the fix.
- **Async testbench cycle alignment is a real framework limitation.** Hand-written multi-domain widgets (`Sync1Bit`, `BitSyncChain`) cannot use the default `TestBench::rtl(...)` per-sample comparison — the codebase convention is `.skip(!0)`, which gets you elaboration coverage but not functional ground-truth. Recorded as a follow-up.
- **Verilog `initial begin` ≠ Rust `dont_care`.** When a DFF's reset value is non-zero, the Verilog `initial` block sets the reg at time 0 but the Rust simulator's initial state is `dont_care` (which prints as 0). They agree after the first clock edge. `core::crc` hits this and uses `.skip(2)`; `core::counter` doesn't because its reset value is 0. Recorded as a follow-up.
- **Const-generic loops in kernels.** `for i in 0..N` with const-bound `N` is unrolled at compile time; `i` is constant *per iteration*. `bits(i as u128)` works. `Bits<N> >> usize` does **not** compile — use `>> (i as u128)`. `bits::<N>(1) << index` (where `index: Bits<W>`) also works for shift-by-runtime.
- **For pure combinational kernels, `test_kernel_vm_and_verilog_synchronous`** is the right Tier 3+4 cover — it compiles to Verilog, runs both Rust VM and iverilog, and compares per input. Used by `core::priority_encoder` and `core::one_hot`.

### Per-widget notes

#### Edge detector — `crates/rhdl-fpga/src/core/edge_detector.rs`

**Why:** Roadmap #1 — the simplest possible RHDL kernel and the canonical reference for the AI-assisted-build workflow.

**Design:** Single `DFF<bool>` for `prev`, three combinational outputs (`rising`, `falling`, `any`) packed into an `Edges` struct. Reset zeroes `prev` and forces all three outputs low. Outputs use `dont_care()` + per-field assignment per the template.

**Surprises:** None — pattern lifted cleanly from `core::counter` and `fifo::write_logic`.

**Validation:** All five tiers pass. `iverilog` round-trip RTL+NTL clean. 9 tests.

#### Pulse stretcher — `crates/rhdl-fpga/src/core/pulse_stretcher.rs`

**Why:** Roadmap #2 — used by debouncer, watchdog, blink-on-event. Composes a counter with a held flag.

**Design:** Bit-width `N` parameterizes the counter; runtime `stretch` value supplied via a `Constant<Bits<N>>` subcore. The kernel reads `q.stretch` and re-arms the counter to `q.stretch` on every high input cycle, decrements otherwise. Output is `q.counter != 0`.

**Surprises:** First widget where I needed a runtime-configurable value inside the kernel. The pattern is to hold it in a `Constant<T>` subcore and read it as `q.<field>`. Same idiom shows up in `axi4lite::register::rom`.

**Validation:** All five tiers, 11 tests, `iverilog` round-trip clean.

#### N-stage synchronizer chain — `crates/rhdl-fpga/src/cdc/synchronizer_chain.rs`

**Why:** Roadmap #3 — generalizes the existing 2-stage `Sync1Bit` to depth `N`. Required by every CDC pattern.

**Design:** Hand-written `impl Circuit` (matching `Sync1Bit`'s style), since `#[kernel]` widgets can't currently express clock-domain-crossing primitives. State holds `[bool; N]` for next/current. HDL is generated programmatically with `quote!` repetition inside `parse_quote!{ ... }` — `#(#reg_decls)*` works for vlog token streams the same way it does for syn.

**Surprises:**
- I emitted the chain without an `initial begin` and got `iverilog` `Expected 0, got x` — non-blocking assignments on undeclared regs start as `X`. Adding `initial begin reg_i = 1'b0; end` for each stage fixed it. `Sync1Bit` doesn't have `initial`s but also uses `.skip(!0)` so it never sees the divergence.
- After fixing initial, hit a different `iverilog` mismatch (`Expected 1 got 0`) under the async testbench. Confirmed via `cross_counter` that this is a framework-level issue with per-event comparison vs `posedge`-driven Verilog updates. Followed prior-art convention of `.skip(!0)` and documented honestly.

**Validation:** Tier 1 N/A (widget is hand-written, no kernel to unit-test directly), Tier 2 (Rust glitch_check), Tier 3 (HDL snapshot for both N=2 and N=4), Tier 4 (`iverilog` elaboration via `.skip(!0)` — see follow-up), Tier 5 (VCD digest).

#### Priority encoder — `crates/rhdl-fpga/src/core/priority_encoder.rs`

**Why:** Roadmap #4. Foundation for arbiters, interrupt controllers, leading-zero count.

**Design:** Pure `#[kernel]` functions (`priority_encoder_lsb`, `priority_encoder_msb`), no struct. Constant-bounded loop, mut `idx` accumulator + mut `found` flag. Returns `Option<Bits<W>>` (per-CLAUDE.md kernels support Option natively).

**Surprises:**
- `Bits<N> >> usize` doesn't compile — only `>> u128`, `>> Bits<M>`, `>> DynBits` exist. Cast loop index: `input >> (i as u128)`.
- For `test_kernel_vm_and_verilog_synchronous` the `K` type-parameter must be the *fully concretized* function instance: `priority_encoder_lsb::<8, 3>` (not `priority_encoder_lsb`). The error message is misleading.

**Validation:** Tier 1 (10 unit tests including exhaustive 8-bit sweep against `u128::trailing_zeros`/`leading_zeros`), Tier 3+4 via `test_kernel_vm_and_verilog_synchronous` for both lsb and msb (256-input sweep), Tier 5 VCD via a `Func` wrapper.

#### One-hot ↔ binary — `crates/rhdl-fpga/src/core/one_hot.rs`

**Why:** Roadmap #5 — pair to priority encoder.

**Design:** Two `#[kernel]` functions. `binary_to_one_hot<W, N>` is a single shift `bits::<N>(1) << index`. `one_hot_to_binary<N, W>` unrolls the same loop pattern as the priority encoder but unconditionally OR-accumulates indices (so multi-hot input gives the OR of indices — documented as unspecified contract).

**Surprises:** None new. The `bits::<N>(1) << Bits<W>` shift just works thanks to the existing `Shl<Bits<M>>` impl on `Bits<N>`.

**Validation:** Tier 1 (8 tests including round-trip `one_hot_to_binary . binary_to_one_hot == id`), Tier 3+4 cross-validation against Verilog for both functions over their full input space, Tier 5 VCD.

#### Debouncer — `crates/rhdl-fpga/src/core/debouncer.rs`

**Why:** Roadmap #6 — first widget to *compose* multiple existing widgets (edge detector + pulse stretcher + DFF). The composition demo.

**Design:** Three subcores; kernel routes their inputs/outputs. The "stable" condition gates whether the input is latched into the output DFF.

**Surprises (and a real bug caught by Tier 2):**
- First draft used `let stable = !q.settle;` which let the very first transition leak through to the output. The bug: `q.settle` is the `PulseStretcher`'s output, which sits behind its internal DFF and so reflects the *previous* cycle's value. On the cycle the input transitions, `q.settle` is still false (the stretcher hasn't been armed yet), so the kernel decided the input was stable and latched the new value.
- Fix: `let stable = !q.settle && !q.edge.any;` — also gate on the edge detector's same-cycle output (`q.edge.any` is `EdgeDetector`'s combinational output, available same-cycle). The Tier 2 `test_short_glitch_rejected` test caught this immediately and is now load-bearing regression coverage. Comment in the kernel calls out *why* the `&& !q.edge.any` term is required.
- The takeaway is general: **when composing widgets, distinguish subcores whose `q.<field>` is same-cycle (combinational outputs of `Constant`, `EdgeDetector`-style logic) from those whose `q.<field>` is delayed (anything fronted by a DFF).** The kernel's mental model has to match.

**Validation:** All five tiers, 10 tests, `iverilog` RTL+NTL clean. Tier 3 uses an HDL-length proxy snapshot (5066 chars) rather than a full text snapshot — see follow-up.

#### Round-robin arbiter — `crates/rhdl-fpga/src/core/round_robin_arbiter.rs`

**Why:** Roadmap #7 — required by multi-master AXI, switch fabrics, DMA channels.

**Design:** Mask-and-rotate variant. Two-DFF state: `last_granted: Bits<W>` and `valid: bool`. The kernel walks all N positions in rotated order starting from `last_granted + 1`, picks the first set request bit. `Bits<W>` arithmetic wraps mod `2^W = N`, so the de-rotated index falls out for free *if* `N = 2^W`. That constraint is documented.

**Surprises:** None — the design works first try once you accept `N = 2^W` as a precondition. Non-power-of-2 `N` would need an explicit modulo, which is more work.

**Validation:** All five tiers, 10 tests including a fairness sweep (32 cycles, all four requesters constantly asking → grants exactly cycle in `0,1,2,3,0,1,2,3,...` order), `iverilog` clean.

#### CRC engine — `crates/rhdl-fpga/src/core/crc.rs`

**Why:** Roadmap #8 — unblocks UART, Ethernet, SPI flash. Last in the first-eight batch on purpose: the dependencies (no protocol PHYs need it yet) make it the rightmost leaf in the build order.

**Design:** Bit-serial, MSB-first shift-register CRC. Polynomial and init are runtime-configurable (each lives in a `Constant<Bits<W>>` subcore). Input struct carries `bit`, `enable`, and a `clear` strobe (which reloads init without needing a global reset).

**Reflection / xor-out are deliberately omitted** — these are message-boundary post-processing steps that vary by use site. A wrapper widget can add them when a specific protocol PHY needs them.

**Surprises:**
- `iverilog` Tier 4 hit the "non-zero DFF reset vs Verilog `initial` block" issue. The DFF resets to `0xFFFF` (CRC-16-CCITT init); Verilog's `initial begin o = 0xFFFF` runs at time 0; Rust sim's state starts as `dont_care` (prints as 0). They agree after the first clock edge. Used `.skip(2)` to bypass the pre-edge sample window. Recorded as follow-up.
- Validated against the well-known `123456789` → `0x29B1` for CRC-16-CCITT (no reflection variant), and against an in-house Rust reference for back-to-back messages via `clear`.

**Validation:** All five tiers, 9 tests, `iverilog` clean (with documented `.skip(2)`). Tier 3 uses HDL-length proxy.
