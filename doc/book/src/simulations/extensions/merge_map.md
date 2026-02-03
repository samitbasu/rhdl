# Merge Map

The `MergeMapExt` provides the `.merge_map` method for iterators that allows you to merge two different timed input streams into a single timed input stream.  The use case is best illustrated with an example.  Suppose we have one stream that defines a sample at time `0, 5, 10`:

```rust
{{#rustdoc_include ../../code/src/simulations.rs:stream1}}
```

and a second strem that defines samples at times `1, 3, 6, 10`:

```rust
{{#rustdoc_include ../../code/src/simulations.rs:stream2}}
```

We can define a new stream that will take the union of the two sets of timestamps, and provide the latest value from each stream to a closure you provide. 

```rust
{{#rustdoc_include ../../code/src/simulations.rs:merge-stream}}
```

The resulting stream will have samples at times `0, 1, 3, 5, 6, 10`, and the value at each time will be a tuple containing the latest value from each stream.  If one of the streams has not yet produced a value, then the default value for that type is used (for example, `b8(0)` for type `b8`).

The following table shows the resulting merged stream:

{{#include ../../code/merge_map.txt}}

One particularly handy use of `.merge_map` is to merge two different clocks into a single stream of values that includes both clocks.  This can be tricky to do by hand, but is straightforward with `.merge_map`.  Here is an example of two clocks of different frequencies being merged:

```rust
{{#rustdoc_include ../../code/src/simulations.rs:red-blue-merge}}
```

The resulting merged clock stream looks like this:

{{#include ../../code/red_blue_clocks.txt}}

