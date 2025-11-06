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
pub trait SynchronousDQ: 'static {
    type D: Digital;
    type Q: Digital;
}
```

and unlike `CircuitDQ`, the associated types are only required to be `Digital`, not `Timed`.  

Like the case of `Circuit`, the `D` and `Q` have implicit constraints in that they must basically take on predetermined forms to work with RHDL.  For a `Synchronous` circuit `X`:

```rust
#[derive(Synchronous)]
pub struct X {
    child_1: A,
    child_2: B,
    child_3: C
}
```

In this case, the type of `D` must be equivalent to:

```rust
#[derive(Digital, Clone, Copy, PartialEq)]
pub struct D {
    child_1: <A as SynchronousIO>::I,
    child_2: <B as SynchronousIO>::I,
    child_3: <C as SynchronousIO>::I,
}
```

and similarly, the type of `Q` must be equivalent to

```rust
#[derive(Digital, Clone, Copy, PartialEq)]
pub struct Q {
    child_1: <A as SynchronousIO>::O,
    child_2: <B as SynchronousIO>::O,
    child_3: <C as SynchronousIO>::O,
}
```

There is a macro that automatically derives these exact type definitions, and you can simply add it to the list for `X`:

```rust
//                       👇 new!
#[derive(Synchronous, SynchronousDQ)]
pub struct X {
    child_1: A,
    child_2: B,
    child_3: C
}
```

If using `#[derive]` macros to create new items gives you the heebie-jeebies, then feel free to write the definitions yourself.  If it's any consolation, `Rust Analyzer` seems to be able to understand the derived structs just fine.
