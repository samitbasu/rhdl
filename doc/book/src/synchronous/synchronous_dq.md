# Feedback

The feedback types for `Synchronous` are completely analagous to those of `Circuit`, as was detailed [here](../circuits/circuits_dq.md).  In the foundational diagram, they appear here:

```badascii
        +------------------------------------------------------------------+        
        |  +------+ SynchronousIO::Q           SynchronousIO::D +--+       |        
  input |  +                   +-----------------------+           +       | output 
 +----->+--------------------->|input            output+-------------------+------->
        |  + +---------------->|c&r      Kernel        |           +       |        
        |  | |            +--->|q                     d+-----+     |       |        
        |  | |            |    +-----------------------+     |     |       |        
        |  | |            |                                  | <---+       |        
        |  +>|            |    +-----------------------+     |             |        
        |    |q.child_1+> +----+o        child_1      i|<----+ <+d.child_1 |        
        |    +-----------+|+-->|c&r                    |     |             |        
        |    |            |    +-----------------------+     |             |        
        |    |            |                                  |             |        
  clock |    |            |    +-----------------------+     |             |        
& reset |    |q.child_2+> +----+o        child_2      i|<----+ <+d.child_2 |        
 +------+----+---------------->|c&r                    |                   |        
  (c&r) |                      +-----------------------+                   |        
        +------------------------------------------------------------------+        
```

The `SynchronousDQ` trait is simple:

```rust
{{#rustdoc_include ../code/src/synchronous.rs:synchronous-dq}}
```

and unlike `CircuitDQ`, the associated types are only required to be `Digital`, not `Timed`.  

Like the case of `Circuit`, the `D` and `Q` have implicit constraints in that they must basically take on predetermined forms to work with RHDL.  For a `Synchronous` circuit `X`:

```rust
{{#rustdoc_include ../code/src/synchronous.rs:x-sync-def}}
```

In this case, the type of `D` must be equivalent to:

```rust
{{#rustdoc_include ../code/src/synchronous.rs:xd-def}}
```

and similarly, the type of `Q` must be equivalent to

```rust
{{#rustdoc_include ../code/src/synchronous.rs:xq-def}}
```

There is a macro that automatically derives these exact type definitions, and you can simply add it to the list for `X`:

```rust
{{#rustdoc_include ../code/src/synchronous.rs:x-sync-derive-def}}
```

If using `#[derive]` macros to create new items gives you the heebie-jeebies, then feel free to write the definitions yourself.  If it's any consolation, `Rust Analyzer` seems to be able to understand the derived structs just fine.

```admonish note
The `SynchronousDQ` derive applied to a struct named `Name` will create a pair of structs named `NameD` and `NameQ` (and more generally, for a struct of name `Name`, a pair of structs named `NameD` and `NameQ`) and give them the definitions described above (with the appropriate generics as needed).  You can force `RHDL` to omit the `Name` prefix by using the attribute `#[rhdl(no_dq_prefix)]` on the struct, but this _will_ cause name collisions if you have multiple circuits in the same module.
```
