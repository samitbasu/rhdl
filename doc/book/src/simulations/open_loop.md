# Open vs. Closed Loop

When thinking about testing a circuit, one of the first decisions you need to make is on the nature of the test.  In my mind, tests can be either "Open Loop" or "Closed Loop".  An Open Loop test is simply a stream of pre-determined input values that are fed to the circuit, and a list of known output values or properties that are checked as they come out of the circuit.  Something like this:

```badascii
+--------+    +-------+    +---------+
| Input  |    |  UUT  |    | Output  |
| Values +--->|I     O+--->| Checker |
|        |    |       |    |         |
+--------+    +-------+    +---------+
```

In this case, the data "flows" in one direction only.  The object generating the input values does not really need to know or care about the unit under test (UUT).  It simply presents a sequence of predetermined input values that are fed into the UUT.  The output of the UUT is then checked by the output checker, which only receives data from the UUT.  This type of test is simple, and extremely well suited to iterator patterns.  If you think about an iterator expression like:

```rust
(0..).take(100).map(|x| x*x).collect::<Vec<i32>>()
```

The pattern is similar.  We start with a source iterator that generates a sequence of `i32` values, map it through some function (our UUT), and then collect the output.  In a similar way, an open loop test is well described by an iterator that produces a sequence of input values.  These are then transformed by the UUI into a sequence of output values.  The checker then drives the whole thing to completion, and measures some critical properties of the output along the way.  It is all linear and very clean.

Unfortunately, it's also hopelessly underpowered.  Because the series generated cannot react to the output of the UUT, you must pre-compute input sequences that test all behaviors you can reach from the initial state.  You cannot, for example, create a test that adjusts it's input based on the output of the UUT.  That is basically the defining characteristic of closed loop tests.  These would look more like this:

```badascii
+--------+    +-------+    +---------+
| Input  |    |  UUT  |    | Output  |
| Values +--->|I     O+-+->| Checker |
|        |    |       | |  |         |
+--------+    +-------+ |  +---------+
    ^                   |             
    +-------------------+             
```

Note that in this case, the output of the UUT is fed back into the input generator.  We now have the ability to compute/adapt the input values we generate to the output of the UUT.  This enables a much more powerful kind of test - one that updates it's stimulus based on the output of the UUT.  These do not fit with iterator patterns.  Iterators in Rust are good models for data flowing in a single direction.  There is no way to really have an iterator "respond" to downstream events or data.  

So for closed loop tests, you need to get further into the details of the simulation loop and write more complicated test code.  Because the logic to compute the next input value is _inside_ the simulation loop, you will generally end up writing your own simulation loop and embedding your test code inside it.  But this is quite simple to do in RHDL.  So fear not.

```admonish note
One technique that can be really nice is to build your test as a RHDL circuit!  This means you can just connect it to the UUT in a simple top level circuit, and then simulate the whole thing.  When I do this, I usually include a single output line that indicates when the output checks have failed.  You can then run an _open loop_ test of the whole thing and just check that the fail signal never goes high.  This approach has the added benefit that you can synthesize the test and the UUT onto hardware and let it do the hard work of simulating the circuit.  It's a great way to get high confidence on designs that need to be bullet proof.  An FPGA running at 100MHz can easily put 10^9 test cases through a design in a matter of seconds.  
```

Becore we go into this in more detail, and illustrate some open and closed loop testing, we will divert momentarily to talk about exhaustive testing.