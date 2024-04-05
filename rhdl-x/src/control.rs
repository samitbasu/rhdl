// Some ideas around using a control signal trait
// Parallel to Digital, but without Default

// Could make Digital: Control + Default....

// Another option...
// Make Control : Digital
// And add some additional methods to indicate the control signals
// Doesn't address the problem of defaults...
// But that _could_ be done with an analysis pass.
// It's kind of the worst of both worlds, since it means
// Control signals can be manipulated like Digital signals.

/*

There are some additional problems.  For example, in Rust, we can
combine Control and Digital signals into a single Tuple, like this:

let c: Control;
let d: Digital;

let m = (c, d);

This would not impl Digital, nor would it impl Control.  Type checking
would now require handling _mixed_ types.  Ugh.




*/

use rhdl_core::{Kind, Notable, TypedBits};

pub struct ClockId(usize);

pub struct ResetId(usize);

pub enum ControlKind {
    Clock(ClockId),
    Reset(ResetId),
    Tuple(Tuple),
    Struct(Struct),
    Empty,
}

pub struct Tuple {
    pub elements: Vec<ControlKind>,
}

pub struct Field {
    pub name: String,
    pub kind: ControlKind,
}
pub struct Struct {
    pub name: String,
    pub fields: Vec<Field>,
}

pub trait Control: Copy + PartialEq + Sized + Clone + 'static + Notable {
    fn static_kind() -> Kind;
    fn bits() -> usize {
        Self::static_kind().bits()
    }
    fn kind(&self) -> Kind {
        Self::static_kind()
    }
    fn bin(self) -> Vec<bool>;
    fn typed_bits(self) -> TypedBits {
        TypedBits {
            bits: self.bin(),
            kind: self.kind(),
        }
    }
    fn binary_string(self) -> String {
        self.bin()
            .iter()
            .rev()
            .map(|b| if *b { '1' } else { '0' })
            .collect()
    }
}

struct Foo {
    a: u8,
    b: u8,
}

/* This fails
fn test_partial() -> Foo {
    let mut z: Foo;

    z.a = 3;
    z.b = 4;
    z
}
*/
