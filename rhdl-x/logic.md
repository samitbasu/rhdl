What if I revisit the idea of the Logic trait, but with the new tools from RHDL in hand?

Let's put the current FSM stuff aside.  

Here is what we used to have: 

```rust
#[derive(LogicBlock)]
pub struct PulseWidthModulator<const N: usize> {
    pub enable: Signal<In, Bit>,
    pub threshold: Signal<In, Bits<N>>,
    pub clock: Signal<In, Clock>,
    pub active: Signal<Out, Bit>,
    counter: DFF<Bits<N>>,
}
```

This provides a structural representation of both inputs, outputs, clock, and
internal structure.  It's messy.  The update function then handled everything:

```rust
impl<const N: usize> Logic for PulseWidthModulator<N> {
    #[hdl_gen]
    fn update(&mut self) {
        clock!(self, clock, counter);
        self.counter.d.next = self.counter.q.val() + 1;
        self.active.next = self.enable.val() & (self.counter.q.val() < self.threshold.val());
    }
}
```

Suppose we make a few changes.

1. Lift the input type to be part of the contract of a LogicBlock.
2. Lift the output type to be part of the contract of a LogicBlock.
3. Same with the clock signal (somehow)
4. Remove inputs and outputs as structural elements of the circuit
5. Change the update function to instead provide the connectivity.

So for now, let's ignore the parameter N, and look again.  First, we
define some input and output types to make it cleaner.

```rust
pub struct PWMInput {
    enable: bool,
    threshold: b8,
    clock: bool,
}

pub struct PWMOutput {
    active: bool,
}

pub struct PWM {
    counter: DFF<b8>,
}

trait LogicBlock {
    type Input : Digital;
    type Output : Digital;

    // Implementations of these are TBD
    fn q(&self) -> Output;
    fn d(&mut self, i: Input);
    fn next();
}

impl LogicBlock for PWM {
    type Input = PWMInput;
    type Output = PWMOutput;
}
```

Next, we need to take the update function and rewrite it as such:

```rust
fn update(&mut self, i: PWMInput) -> PWMOutput {
    self.counter.


    clock!(self, clock, counter);
    self.counter.d.next = self.counter.q.val() + 1;
    self.active.next = self.enable.val() & (self.counter.q.val() < self.threshold.val());
}

```
