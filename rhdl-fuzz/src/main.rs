enum Kind {
    Bits(usize),
    Struct(StructDef),
    Enum(EnumDef),
    Tuple(TupleDef),
}

struct TupleDef {
    fields: Vec<Kind>,
}

struct StructDef {
    fields: Vec<FieldDef>,
}

struct FieldDef {
    name: String,
    kind: Kind,
}

struct EnumDef {
    variants: Vec<VariantDef>,
    discriminant_width: usize,
}
struct VariantDef {
    name: String,
    kind: Kind,
}

fn random_kind() -> Kind {
    // Return a randomly generated Kind each time it is called
}
fn main() {
    println!("Hello, world!");
}
