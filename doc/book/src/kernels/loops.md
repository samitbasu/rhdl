# Loops

In general, kernel's do not loop.  Inside your kernel, you can only use a very limited form of looping.  The only supported version of looping in a kernel is a `for` loop with the following constraints:

1. The loop bounds must be simple and computable at compile time.
2. The loop index must be an integer.
3. No early exit (e.g., `break`/`continue`) are allowed.
4. The `for` loop must iterate over a half open range with both ends defined, e.g., `for ndx in 3..9`.

The best way to think of a `for` loop in a hardware design is as a shorthand for copy-pasting a given design block a (fixed and predetermined) number of times.  In order to translate the loop correctly, RHDL needs to determine exactly how many times you want the block repeated.  It then "unrolls" the loop by creating the requested number of copies of the internal design.

A good example of using a `for` loop is to count the number of bits set in a given value.  The kernel for that might look something like this:

```rust,kernel:loops
#[kernel]
pub fn kernel(a: b32) -> b9 {
    let mut count = b9(0);
    for i in 0..32 {
        if a & (1 << i) != 0 {
            count += 1;
        }
    }
    count
}
```

This simple looking kernel will generate quite a monstrous tree of adders in a long chain.  Something like this:

```badascii
 b0    +---+                  
+----->|   |    +---+         
 b1    | + +--->|   |    +---+
+----->|   |    | + +--->|   |
 b2    +---+    |   |    |   |
+-------------->|   |    | + |
 b3             +---+    |   |
+----------------------->|   |
                         +---+
            .                 
            .                 
            .                 
```

where `b0..` are the bits of `a`.  You can also make the loop depend on the generic parameter (which makes it more useful).  Suppose, for example, we want a generic ones-counter that will work for any bit vector width.

```rust,kernel:loops
#[kernel]
pub fn count_ones<const N: usize, const M: usize>(a: Bits::<N>) -> Bits::<M> 
where rhdl::bits::W<N>: BitWidth,
      rhdl::bits::W<M> : BitWidth
{
    let mut count = bits::<M>(0);
    for i in 0..N {
        if a & (1 << i) != 0 {
            count += 1;
        }
    }
    count
}

#[kernel]
pub fn kernel(a: b8) -> b4 {
    count_ones::<8,4>(a)
}
```

This version of the kernel will adapt to the `const` generic parameters `N` and `M`.  

Using a loop does not necessarily imply a long linear chain of circuitry.  The linear chain nature of the ones-counter is due to the need for the count to propagate from one iteration of the loop to another.  You can also do parallel operations using a `for` loop.  Here is an example that builds a XNOR gate.

```rust,kernel:loops
#[kernel]
pub fn xnor<const N: usize>(a: Bits::<N>, b: Bits::<N>) -> Bits::<N> 
where rhdl::bits::W<N>: BitWidth
{
    let mut ret_value = bits::<N>(0);
    for i in 0..N {
        let a_bit = a & (1 << i) != 0;
        let b_bit = b & (1 << i) != 0;
        if !(a_bit ^ b_bit) {
            ret_value |= (1 << i);
        }
    }
    ret_value
}

#[kernel]
pub fn kernel(a: b8, b: b8) -> b8 {
    xnor::<8>(a,b)
}
```

After optimization, this circuit is completely parallel.  Each bit of the output can be computed from the corresponding bit of `a` and `b`, which means that there is no long chain of logic from the inputs to the output.

You can also use `for` loops to deal with arrays.  For example, the previous kernel could also be written as follows, where `a, b` have been pre-expanded into bool arrays:

```rust,kernel:loops
#[kernel]
pub fn generic<const N: usize>(a: [bool; N], b: [bool; N]) -> [bool; N] {
    let mut ret_value = [false; N];
    for i in 0..N {
        ret_value[i] = !(a[i] ^ b[i]);
    }
    ret_value
}


#[kernel]
pub fn kernel(a: [bool; 4], b: [bool; 4]) -> [bool; 4] {
    generic::<4>(a,b)
}
```

Here the parallelism is clear.