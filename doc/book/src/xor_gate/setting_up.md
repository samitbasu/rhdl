## Setting Up

Let's start by creating a new Rust project.  We want a library, since the `xor` gate is something that will go into other designs.  If we were making a final top level design, a binary might be more appropriate.  We need the `rhdl` dependency.  `RHDL` uses `miette` to provide error reporting, so we will add that as well.

First we create a new project:

<!-- cmdrun to-html "cd /tmp && rm -rf xor && cargo new --lib xor" -->

Then we add the dependencies.  RHDL is available as a single crate via `crates.io`.

<!-- cmdrun to-html "cd /tmp/xor && cargo add --path ~samitbasu/Devel/rhdl/crates/rhdl" -->

We also add the `miette` crate as a development dependency, with the `fancy` feature enabled to get the best error messages.

<!-- cmdrun to-html "cd /tmp/xor && cargo add --dev miette --features fancy" -->


In the `src` directory, we will start by importing the prelude for `RHDL`, which brings a lot of useful signals into scope. 

```rust
use rhdl::prelude::*;
```

So far, so good.
