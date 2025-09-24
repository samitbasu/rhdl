# Chapter 2


```rhdl-silent
# Remove the foo directory to get a clean slate
rm -rf foo
```

Let's start by creating a new Rust project.  We want a binary, and we need the `rhdl` dependency.

```rhdl-shell
cargo new --bin foo
cd foo && cargo add --path ~samitbasu/Devel/rhdl rhdl
```

```rust
use rhdl::prelude::*;
```

Here we will create a directory.

```rhdl-shell
ls -la --color=always
```
