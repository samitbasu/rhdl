use crate::{
    Kind, TypedBits,
    types::path::{Path, PathElement},
};

// Construct the leaf paths of the current object.  This version is a customized
// copy of [leaf_paths], which is meant to make the enumerated paths easier to
// understand for readability.
fn pretty_leaf_paths_inner(kind: Kind, base: Path) -> Vec<Path> {
    // Special case base is a payload, and kind is a single-element tuple.  This happens
    // with enums, where the payload is a single-element tuple.  For readability, we
    // project through the payload to the tuple.
    if matches!(base.iter().last(), Some(PathElement::EnumPayload(_))) {
        match kind {
            Kind::Tuple(tuple) if tuple.elements.len() == 1 => {
                return pretty_leaf_paths_inner(tuple.elements[0], base.clone().tuple_index(0));
            }
            _ => {}
        }
    }
    let mut root = vec![base.clone()];
    let branch = match kind {
        Kind::Array(array) => (0..array.size)
            .flat_map(|i| pretty_leaf_paths_inner(*array.base, base.clone().index(i)))
            .collect(),
        Kind::Tuple(tuple) => tuple
            .elements
            .iter()
            .enumerate()
            .flat_map(|(i, k)| pretty_leaf_paths_inner(*k, base.clone().tuple_index(i)))
            .collect(),
        Kind::Struct(structure) => structure
            .fields
            .iter()
            .flat_map(|field| pretty_leaf_paths_inner(field.kind, base.clone().field(&field.name)))
            .collect(),
        Kind::Signal(root, _) => pretty_leaf_paths_inner(*root, base.clone().signal_value()),
        Kind::Enum(enumeration) => enumeration
            .variants
            .iter()
            .flat_map(|variant| {
                pretty_leaf_paths_inner(variant.kind, base.clone().payload(&variant.name))
            })
            .collect(),
        Kind::Bits(_) | Kind::Signed(_) | Kind::Empty | Kind::Clock | Kind::Reset => vec![],
    };
    root.extend(branch);
    root
}

pub fn pretty_leaf_paths(kind: Kind, base: Path) -> Vec<Path> {
    // Remove all instances of #variant followed by #variant.0 - the
    // first does not add any value when pretty printing
    pretty_leaf_paths_inner(kind, base)
}

// Apply a path sequence to a TypedBits object, but use None instead of blindly
// assuming the path is valid.  There is only one case in which the path may yield
// a None, and that is when the path requests the EnumPayload but the payload does
// not match the discriminant of the enum.  All other cases, can be forwarded to
// the regular path method.
pub(crate) fn try_path(t: &TypedBits, path: &Path) -> Option<TypedBits> {
    let mut t = t.clone();
    for element in path.iter() {
        match element {
            PathElement::EnumPayload(tag) => {
                let discriminant = t.discriminant().ok()?.as_i64().ok()?;
                let tag_discriminant = t
                    .kind()
                    .get_discriminant_for_variant_by_name(tag)
                    .ok()?
                    .as_i64()
                    .ok()?;
                if discriminant == tag_discriminant {
                    t = t.path(&Path::default().payload(tag)).ok()?;
                } else {
                    return None;
                }
            }
            PathElement::EnumPayloadByValue(tag_discriminant) => {
                let tag_discriminant = *tag_discriminant;
                let discriminant = t.discriminant().ok()?.as_i64().ok()?;
                if discriminant == tag_discriminant {
                    t = t
                        .path(&Path::default().payload_by_value(tag_discriminant))
                        .ok()?;
                } else {
                    return None;
                }
            }
            x => {
                t = t.path(&Path::with_element(*x)).ok()?;
            }
        }
    }
    Some(t)
}
