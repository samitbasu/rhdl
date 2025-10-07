## Setting Up

```shell,rhdl-silent
# Remove the xor directory to get a clean slate
rm -rf xor
```

Let's start by creating a new Rust project.  We want a library, since the `xor` gate is something that will go into other designs.  If we were making a final top level design, a binary might be more appropriate.  We need the `rhdl` dependency.  `RHDL` uses `miette` to provide error reporting, so we will add that as well.

```shell,rhdl
cargo new --lib xor
cd xor
cargo add --path ~samitbasu/Devel/rhdl/crates/rhdl rhdl
cargo add --dev miette
```

In the `src` directory, we will start by importing the prelude for `RHDL`, which brings a lot of useful signals into scope. 

```rust,write:xor/src/lib.rs
use rhdl::prelude::*;
```

So far, so good.

```shell,rhdl:xor
cargo check -q
```
