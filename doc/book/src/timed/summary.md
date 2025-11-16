# Timed Types and Clock Domains

A key characteristic of RHDL is that it codifies the notion of `clock domains` into the core of the type system.  In case yoiu are unfamiliar with the term, a `clock domain` refers (in our definition) to a sub-section of your circuit and I/O that operate synchronously to some time constraint.  In nearly all designs, this is probably a periodic clock with some frequency.  But it does not have to be.  Asynchronous signals and circuits also operate with some notion of signals changing in some defined way.  But starting with clocked signals is probably simplest.  So imagine we have a design that has 3 clock domains:

- Asynchronous I/O as might arrive from a button press or from a UART, and drive LEDs
- Core logic that is running at some nominal clock frequency of 100 MHz
- A high speed memory interface that is running at 800 MHz.  

They might be laid out in a diagram something like this:

```badascii
              +-------++----------++-------------+             
              |       ||          ||             |             
To/From +---->| Async ||Core Logic|| DRAM Driver +----> To/From
Reality <-----+       || 100 MHz  ||   800MHz    |<---+  DRAM  
              |       ||          ||             |             
              +-------++----------++-------------+             
```

In RHDL, there is a trait `Timed`, which indicates that a time varying quantity has been assigned to some timing domain, denoted by colors.  The actual color used is irrelevant (unless you ascribe meaning to it for your own purposes), so we could define these three domains as `Red, Green, Blue`, for example:

```badascii
              +-------++----------++-------------+             
              |       ||          ||             |             
To/From +---->| Async ||Core Logic|| DRAM Driver +----> To/From
Reality <-----+       || 100 MHz  ||   800MHz    |<---+  DRAM  
              |       ||          ||             |             
              +-------++----------++-------------+             
                  ^          ^           ^                     
                  +          +           +                     
                 Red       Green       Blue                    
```

The actual color used is not important, but the fact that these are three different colors _is_ significant.  In particular:

- RHDL assumes that signals from different clock domains cannot be mixed
- RHDL assumes that every signal belongs to exactly one clock domain

Thus, the clock domains form a partition of your design.  And they are isolated from one another via the type system.  For example, in reference to our design above, we may have three signals at different locations in our design:

- `Signal<bool, Red>`, which indicates the value of a button press, and comes from Reality (Asynchronous to any clock).
- `Signal<b8, Green>`, which counts the number of button presses, and changes only on clock edges in the `Green` domain.
- `Signal<b16, Blue>`, which is the address bus to the DRAM, and changes only on the 800 MHz clock that defines the `Blue` domain.

Each of these will `impl Timed`, but you cannot move values from one clock domain to another without using special circuitry called a `clock domain crosser` or `synchronizer`.

```admonish note
Clock domain crossers are sort of like `unsafe` blocks in normal Rust.  They are designed to transfer the signals correctly and safely from one clock domain to another.  Internally, they mix signals from different clock domains, but do so in a way that is provably correct.  The type signature of the clock domain crossers allows you to use them in circuits with multiple clock domains as long as you wire them correctly.
```

With the clock domain crossers, the circuit will look like this:

```badascii
              +-------+-+----------+-+-------------+             
              |       | |          | |             |             
To/From +---->| Async |C|Core Logic|C| DRAM Driver +----> To/From
Reality <-----+       |D| 100 MHz  |D|   800MHz    |<---+  DRAM  
              |       |C|          |C|             |             
              +-------+-+----------+-+-------------+             
                  ^           ^            ^                     
                  +           +            +                     
                 Red        Green        Blue                    
```

```admonish warning
Moving signals from one clock domain to another with going through a clock domain crosser is almost guaranteed to result in data corruption.  RHDL will try very hard to keep you from doing this.  You can force signals to cross without a domain crosser, but all correctness guarantees become your responsibility.  Ideally, doing so would require `unsafe`, but I didn't want to introduce the `unsafe` keyword into RHDL and confuse you, since there is no `unsafe` code in RHDL.
```

