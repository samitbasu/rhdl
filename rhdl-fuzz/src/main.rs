use rand::{thread_rng, Rng};

#[derive(Debug)]
enum Kind {
    Bits(usize),
    Struct(StructDef),
    Enum(EnumDef),
    Tuple(TupleDef),
    Empty,
}

#[derive(Debug)]
struct TupleDef {
    fields: Vec<Kind>,
}

#[derive(Debug)]
struct StructDef {
    fields: Vec<FieldDef>,
}

#[derive(Debug)]
struct FieldDef {
    name: String,
    kind: Kind,
}

#[derive(Debug)]
struct EnumDef {
    variants: Vec<VariantDef>,
    discriminant_width: usize,
}
#[derive(Debug)]
struct VariantDef {
    name: String,
    kind: Kind,
}

fn random_bits() -> Kind {
    let num_bits = thread_rng().gen_range(1..=64);
    Kind::Bits(num_bits as usize)
}

fn random_tuple() -> Kind {
    let num_fields = thread_rng().gen_range(1..=8);
    let mut fields = Vec::with_capacity(num_fields);
    for _ in 0..num_fields {
        fields.push(random_kind());
    }
    Kind::Tuple(TupleDef { fields })
}

fn random_struct() -> Kind {
    let num_fields = thread_rng().gen_range(1..=8);
    let mut fields = Vec::with_capacity(num_fields);
    for n in 0..num_fields {
        fields.push(FieldDef {
            name: format!("field_{}", n),
            kind: random_kind(),
        });
    }
    Kind::Struct(StructDef { fields })
}

fn random_enum() -> Kind {
    let num_variants = thread_rng().gen_range(1..=8);
    let mut variants = Vec::with_capacity(num_variants);
    for n in 0..num_variants {
        variants.push(VariantDef {
            name: format!("variant_{}", n),
            kind: random_kind_or_empty(),
        });
    }
    Kind::Enum(EnumDef {
        variants,
        discriminant_width: 8,
    })
}

fn random_kind_or_empty() -> Kind {
    if thread_rng().gen_bool(0.5) {
        random_kind()
    } else {
        Kind::Empty
    }
}

fn random_kind() -> Kind {
    match thread_rng().gen_range(0..=3) {
        0 => random_bits(),
        1 => random_tuple(),
        2 => random_struct(),
        3 => random_enum(),
        _ => unreachable!(),
    }
}
fn main() {
    println!("{:?}", random_kind());
}
