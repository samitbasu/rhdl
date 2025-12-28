Some ideas:

1.  Currently, there is an imperfect association between time stamps, the trace data, and the container.  
2.  This is because the trace database is either storing data continuously or not.
3.  And the containers only hold timestamps, not actual data.

Instead,

What about:

```rust
let page = tracedb.allocate(time); // Allocate a new page of trace data at the current time
let output = self.uut.sim(page, inputs, state);
```


Could let `TimedSample` hold the trace page...

```rust
pub struct TimedSample<T: Digital> {
    time: u64,
    page: Option<TracePage>,
    data: T,
}
```

This way, you can opt-in to tracing at the input iterator side....

We don't actually need to change behavior of the trace database then.  

Traces still go to the tracedb.  But the tracedb is gated by the page....

Keep thinking.


What if we pass the TracePage into the `sim` function as above.  Then the `trace` call can
access the `TracePage`.  The trace now becomes simple a list of `(hash, value, kind)` dumps.  
We could just bitpack the resulting value into a 3-value logic.  I think that already happens...  No, the current design stores the valies in a `Vec<T>`.  This is not directly possible.

However, I like the idea that now that tracing is opt-in, we can spend the cost to "serialize" the data to the page since the end user has already stated that they want
the data preserved.  

So suppose a page is just something like:

```rust
pub struct TracePage {
    data: Vec<TraceBit>, // The recorded data.  Each value is called with value.trace() when traced.
    meta: Vec<Record>, // Meta data about the trace page
    path: Vec<&'static str>, // Current path active on the page
}

pub struct Record {
    start: usize, // Start index of the tracebits in the page
    len: usize, // End index of the tracebits in the page
    tkind: Intern<TimeSeriesDetails>, // Pointer to path, key, and trace type
}
```

Now the `sim` function can take a `Option<mut &TracePage>` reference to the struct.  And pass that into each of the children.  But how do we deal with this in the `kernel` function?  

1.  Either we special case the TracePage so that every kernel contains as the first argument something like:

```rust
#[kernel]
fn kernel(trace: Option<mut &TracePage>, args...)
```

And then drop the `trace` argument when synthesizing.  This has the advantage of requiring no magic, and ultimately being compatible with threading.  Since kernels form a dag, this should be OK.  However, it means that when kernel `ka` calls kernel `kb`, then we have some additional work to do.  It also allows us some control over tracing internally, since we can substitute `None` for the argument to suppress tracing in sub-functions.


2. The second case is to use a magic handle, like an int, and then `#[derive(Digital)]` on it with a ZST so that it is automatically dropped.  This requires being able to use the handle to get back to the trace page.  And that means the use of either a global static or thread local storage.  A global static is bad for many reasons.  

What I need is something like a "scoped static".

Ok, so the `Option<mut &TracePage>` so far seems like the best option.  It allows for future multi-threading, and has very low cost in terms of passing the TracePage handle around to the different parts of the circuit.  We can then update the `TimedSample` implementation to hold a boxed tracepage:

```rust
struct TimedSample<T> {
    time: u64,
    value: T,
    trace: Option<Box<TracePage>>,
}
```


