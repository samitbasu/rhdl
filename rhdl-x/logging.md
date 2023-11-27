We want logging to be something super simple and trivial to use.   The equivalent of the `log!` macro used in 
standard rust.  However, logging does not use formatting or textual descriptions of the payloads (no formatting).
To avoid conflicts with the `log` macro (which may be used for regular ole logging), we use the synonym `note!`.

A `note!` must have a level, that can be used for filtering later on.  These are 
synonymous with the usual log levels: trace, debug, info, warn and error.  A `note!` can also take a target
string literal (if none is provided, the argument is stringified as the string literal).  Finally, the `note!` 
macro will take the value to be noted.  Which must `impl Digital`.  We do not make `note!` a function, since
we want to concatenate the source file and module information into the string used in the note index.


So let's start with the macro.  Borrowed heavily from the Rust log macro

```rust

macro_rules! note {
    (target: $target:literal, $lvl:expr, $value:expr) => ({
        let lvl = $lvl;
        $crate::note(lvl, concat!($target, module_path!(), file!()), &$value)
    })
    // note!(level, value) --> note!(target: module_path)
    ($lvl: expr, $value: expr) => ($crate::note!(target: module_path!(), $lvl, $value))
}

```

Let's start with that...  A few wrinkles, but the following works:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NoteLevel {
    Error = 0,
    Warn,
    Info,
    Debug,
    Trace,
}

#[macro_export]
macro_rules! note {
    (target: $target:literal, $lvl:expr, $value:expr) => {{
        let lvl = $lvl;
        $crate::note(
            lvl,
            concat!(module_path!(), "::", file!(), "::", line!(), "::", column!(), "::", $target),
            &$value,
        )
    }};
    // note!(level, value) --> note!(target: module_path)
    ($lvl: expr, $value: expr) => {{
        let lvl = $lvl;
        $crate::note(
            lvl,
            concat!(module_path!(), "::", file!(), "::", line!(), "::", column!(), "::", stringify!($value)),
            &$value,
        )
    }};
}

fn note(lvl: NoteLevel, target: &'static str, value: &impl Digital) {
    eprintln!("{:?}: {} = {:?}", lvl, target, value.typed_bits());
}
```

Additional concern.  `note` is a function with side effects.  What happens when it is dropped from the
HDL version?  Is there a potential for weirdness here?  It would be better if the `$value:expr` argument 
was something that had no side effects.

I am concerned that `typed_bits` will suffer from terrible performance.  So we 
need somthing slightly better.  Ideally something that generates a Note.

```rust
trait Digital {
    fn as_note(&self) -> Note
}
```

Where a `Note` is a record that describes the object.  We do not want to store the
metadata associated with the `Note` in every record.  But the simplest solution to
this is to make it a dyn trait (object trait), so that the metadata is stored in a
virtual table, and handled by Rust.

A note stack could look something like this:

```rust
struct NoteStack {
    kind: Kind,
    notes: Vec<StampedNote>,
}
```

Where

```rust 
struct StampedNote {
    node: Note,
    timestamp: u64,
}
```

Better might be to have the Digital object serialize itself to a stream.  For example,

```rust
trait Digital {
    fn note(&self, W: impl NoteWriter)
}
```

Then `NoteSerializer` could have the following methods:

```rust
trait NoteWriter {
    fn write_bool(&mut self, val: bool);
    fn write_bits(&mut self, bits: u8, val: u128);
    fn write_signed(&mut self, bits: u8, val: i128);
    fn write_string(&mut self, val: &'static str);
}
```

This would be sufficient for any `Digital` object to serialize itself to the `NoteWriter` stream.
The resulting stream could contain an arbitrary representation.  The `Digital` object would not care
how the data is represented. 

For the data structure to be seekable, we need to be able to find the offset of any given field in
the data structure without necessarily reading the entire data structure from scratch.  For now,
let's assume that means we have a cursor interface to a Note.  A Cursor interface would have something
like:

```rust
trait NoteCursor {
    fn pos(&self) -> usize;
    fn advance_bool(&mut self);
    fn advance_bits(&mut self, bits: u8);
    fn advance_signed(&mut self, bits: u8);
    fn advance_string(&mut self);
}
```

An alternate option would be to have the NoteDB itself store the unpacked representation.  This is 
somewhat closer to what I had before with the Logging design.  In this design, we would simply add
additional keys for each of the subelements of the data structure.  So something like:

```rust
trait Digital {
    fn note(&self, key: &'static str, w: impl NoteWriter);
}
```

We then have basic impls for the core types:

```rust
impl Digital for bool {
    fn note(&self, key: &'static str, w: impl NoteWriter) {
        w.write_bool(key, *self);
    }
}
```

And similarly
```rust
impl<const N: usize> Digital for Bits<N> {
    fn note(&self, key: &'static str, w: impl NoteWriter) {
        w.write_bits(key, N, *self.0);
    }
}
```

Then, when we have a struct, we can augment the keys using the `concat` macro.  So, for example:

```rust
struct Foo {
    a: b4,
    b: b23,
}


impl Digital for Foo {
    fn note(&self, key: &'static str, w: impl NoteWriter) {
        a.note(concat!(key, ".", stringify!(a)), &mut w);
        b.note(concat!(key, ".", stringify!(b)), &mut w);
    }
}
```

This design means the individual structs no longer "exist" in the log stream.  They are just time series
with name value pairs.  Which is ideal for both delta detection (change logging), as well as post processing.

From a data storage efficiency perspective, the NoteWriter can even optimize over the size of the object being
stored.  If it matters.  Since that is an optimization detail, for now, we can assume a super simple set of options.

```rust

enum NoteRecord {
    Bool(bool),
    Bits(u128, u8),
    Signed(i128, u8),
    String(&'static str),
}

struct TimedNoteRecord {
    record: NoteRecord,
    time: u64,
}

struct DeltaVec<T> {
    vec: Vec<T>,
}

impl<T: PartialEq + Copy> DeltaVec<T> {
    pub fn push(&mut self, val: T) {
        if let Some(prev) = self.last() {
            if prev != val {
                self.vec.push(val)
            }
        } else {
            self.vec.push(val)
        }
    }
}

struct BaseNoteWriter {
    db: HashMap<&'static str, Vec<TimedNoteRecord>>,
    time: u64,
}

impl NoteWriter for BasicNoteWriter {
    fn write_bool(&mut self, key: &'static str, value: bool) {
        self.db.entry(key).or_default().push(Timed::new(value, self.time));
    }
    fn write_bits(&mut self, key: &'static str, value: ...)...
}
```

I think this will work.  It is simple, transparent, configurable.  The magic of a hierarchical
call setup is lost, but I think it's worth it.  

Additional difficulties arise.  You cannot do this:

```rust
fn foo(key: &'static str) {
    let k = concat!(key, ".");
}

```

Because `concat!` only works with string literals.  Makes sense, since we want this resolved
at compile time, but the compiler does not know contents of `key`.  

So we need another strategy.  One option is to think of the key not as a static string, but as
a tuple of static strings.  Something like:

```rust
fn foo(key: &'static str) {
    let k = (key, ".");
}
```

We can make the `foo` function generic over a trait that is implemented only for static strings
and tuples of them.  


Solving multiple instances can also be done by including string literals in the structs, and making
them ignored by the Digital trait.  Something like this:

```rust
struct MyState {
    target: &'static str,
}
```

Then when we call `note` from within the update function associated with MyState, we can include the
target in the state in the key.

```rust

fn update() {
    note!(target = q.target, ...)
}
```

This approach would at least allow for the identification of different calls to `update` in the VCD.

I could also use the `backtrace` crate to capture a backtrace at the point of the `note` call.  This
seems pretty expensive tho.

Another option would be to use some macro magic to insert caller information into the process.  Like
the span based tracers do.  That would mean something like this.

```rust
#[kernel]
fn foo<Generics>(args) -> ret {}
```

Expands to

```rust
fn foo<Generics>(args) -> ret {
    note_set_context("foo");
    // original foo() function
    fn inner<Generics>(args) -> ret {
        // original body goes here
    }
    let ret = inner::<Generics>(args);
    note_clear_context();
    return ret;
}
```

In the NoteDB, the Key could be built from this context as needed.  The injection 
