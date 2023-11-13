There are a bunch of problems...
- RA is big - about 300KLOC
- RA includes a bunch of performance oriented features I don't need or want (salsa among them)
- RA does not do stuff I need (like monomorphism)
- rustc is even worse... about 2.1MLOC
- Everything about rustc is considered unstable, not fit for human consumption.

Taking a step back.  

The problem is one of name resolution, imports and generics.
The question is if this is something that can be handled by the rust compiler itself.

One idea is to have the kernel function belong to a type.  This would mean it could be
covered by a trait.  And as a result, the rust compiler would allow me to do more at proc-macro time.

We already have one trait for `digital`, which defines anything that can be laid out.

- This is the "data" trait.

Now we need a trait for stuff that computes.  Let's call it the `compute` trait. 

How does that help?  We could use the parent struct/type as a container/namespace of some
kind.  There is already a `Synchronous` trait.  If we require HDL generation elements
to use it, we can handle generics via `rustc`.  It would go something like this:

```rustc
pub trait Synchronous: Sized {
    type Input: Copy + Loggable + PartialEq;
    type Output: Copy + Loggable + Default;
    type State: Copy + Default + Loggable;
    // User provided
    fn compute(
        &self,
        logger: impl Logger,
        inputs: Self::Input,
        state: Self::State,
    ) -> (Self::Output, Self::State);
}
```

Then our compute function is connected to a data structure.  Something like:

```rustc
struct FIFO<D: Digital, const N: usize> {
    // internal details
}

impl<D: Digital, const N: usize> Synchronous for FIFO<D, N> {
    type Input = ..

    #[kernel]
    fn compute(&self, logger: impl Logger, inputs: Self::Input) {}
}
```

This way, the #[kernel] invokation can probably be moved to the struct
itself as a Derive.  Something like:

```rustc

#[derive(Synchronous)]
struct FIFO<D: Digital, const N: usize> {
   //internal details
}

// Somehow, we provide the compute function...

```

Maybe need to revisit how salsa does this.
