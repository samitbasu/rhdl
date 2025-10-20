# Unsupported Rust Features

It seems easiest to start with a list of what is _not_ supported when writing kernels.  With the unsupported list in place, we can then focus on the details of what _is_ supported.  It is possible this list will change over time, but here is where it stands as of now (this list is not exhaustive, but covers the main areas)

- References, pointers, and lifetimes.  Everything in a `kernel` function is passed by value.  The idea of "borrowing" data doesn't really translate to hardware.
- Unions.  Use `enum`, which is supported.
- Floating point values.  Maybe in the future.  But you can definitely build your own floating point library in RHDL.
- Closures.  Maybe in the future if they are closures that do not capture from their environment.
- `unsafe`.  It may make sense to use `unsafe` to indicate that certain operations require analysis beyond what RHDL can prove.  For example, clock domain crossers "prove" their correctness using techniques not available to the compiler.  It might make sense to use `unsafe` for these purposes in the future.
- `while`, `while let`, `loop` and `break/continue`.  There is limited support for the `for` loop when it can be unrolled at compile time.
- Item definitions within functions.  Modules are fully supported, if you need detail hiding and encapsulation.
- Macro invocations.
- Anything referencing `self`.  Just use the function call syntax.
- Some pattern types.  Basic pattern matching will work as it can be translated into a ROM or select.  More complicated pattern matching is really inferring more hardware than you probably want.
- Anything `async`/`await`.  Hardware design isn't quite there yet.  It is interesting though, to think about writing state machine patterns this way.
- `const` blocks.  Currently unsupported, but probably achievable.
- slices.  These are pointers in disguise.  But arrays of known size (like `[T; N]`) are supported.

This may seem like a long list, but what remains is still very powerful, and more than adequate for the task of building complex hardware designs.  If it is any consolation, many/all of these features are not available in "traditional" HDLs either.  They are generally software programming paradigms or techniques that have limited or no applicability to hardware design.  Some of them, like `const` blocks and macro invocation support can be added if needed in the future.

```admonish note
You can use any/all of these features in any Rust code that does not need to be synthesizable.  I make heavy use of closures, iterators, random number generators, etc. when building test cases and testing the kernels.  The limits only apply to the functions that need to be synthesizable.   
```