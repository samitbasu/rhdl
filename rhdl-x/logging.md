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

And then the `note` function could simply do:


