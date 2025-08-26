// Given a Kind, attempt to translate it into a RHDL Trace Type
// This is the "Default" implementation, which can be overridden
// by a particular DIGITAL type as needed.
use crate::rhdl_core::types::kind;
use rhdl_trace_type as rtt;

impl From<crate::rhdl_core::Color> for rtt::Color {
    fn from(color: crate::rhdl_core::Color) -> Self {
        match color {
            crate::rhdl_core::Color::Red => rtt::Color::Red,
            crate::rhdl_core::Color::Orange => rtt::Color::Orange,
            crate::rhdl_core::Color::Yellow => rtt::Color::Yellow,
            crate::rhdl_core::Color::Green => rtt::Color::Green,
            crate::rhdl_core::Color::Blue => rtt::Color::Blue,
            crate::rhdl_core::Color::Indigo => rtt::Color::Indigo,
            crate::rhdl_core::Color::Violet => rtt::Color::Violet,
        }
    }
}

impl From<kind::DiscriminantAlignment> for rtt::DiscriminantAlignment {
    fn from(da: kind::DiscriminantAlignment) -> Self {
        match da {
            kind::DiscriminantAlignment::Msb => rtt::DiscriminantAlignment::Msb,
            kind::DiscriminantAlignment::Lsb => rtt::DiscriminantAlignment::Lsb,
        }
    }
}

impl From<kind::DiscriminantType> for rtt::DiscriminantType {
    fn from(dt: kind::DiscriminantType) -> Self {
        match dt {
            kind::DiscriminantType::Unsigned => rtt::DiscriminantType::Unsigned,
            kind::DiscriminantType::Signed => rtt::DiscriminantType::Signed,
        }
    }
}

impl From<kind::DiscriminantLayout> for rtt::DiscriminantLayout {
    fn from(dl: kind::DiscriminantLayout) -> Self {
        rtt::DiscriminantLayout {
            width: dl.width,
            alignment: dl.alignment.into(),
            ty: dl.ty.into(),
        }
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use crate::rhdl_core::Kind;
    use rhdl_trace_type as rtt;
    use rhdl_trace_type::TraceType;
    pub(crate) fn kind_to_trace(kind: &Kind) -> TraceType {
        match kind {
            Kind::Array(array) => kind_array_to_trace(array),
            Kind::Tuple(tuple) => kind_tuple_to_trace(tuple),
            Kind::Struct(strukt) => kind_struct_to_trace(strukt),
            Kind::Enum(enumerate) => kind_enum_to_trace(enumerate),
            Kind::Bits(len) => TraceType::Bits(*len),
            Kind::Signed(len) => TraceType::Signed(*len),
            Kind::Signal(ky, color) => {
                TraceType::Signal(Box::new(kind_to_trace(ky)), (*color).into())
            }
            Kind::Empty => TraceType::Empty,
        }
    }

    // An array must consist of physical elements.  So the base element
    // must be translated into a TraceType.
    fn kind_array_to_trace(array: &kind::Array) -> TraceType {
        let base = kind_to_trace(&array.base);
        TraceType::Array(rtt::Array {
            base: Box::new(base),
            size: array.size,
        })
    }

    // A tuple may contain a mix of physical and non-physical elements.
    // The non-physical elements need to be dropped from the representation.
    fn kind_tuple_to_trace(tuple: &kind::Tuple) -> TraceType {
        let elements = tuple.elements.iter().map(kind_to_trace).collect();
        TraceType::Tuple(rtt::Tuple { elements })
    }

    fn kind_struct_to_trace(strukt: &kind::Struct) -> TraceType {
        let fields = strukt
            .fields
            .iter()
            .map(|field| rtt::Field {
                name: (*field.name).clone(),
                ty: kind_to_trace(&field.kind),
            })
            .collect();
        TraceType::Struct(rtt::Struct {
            name: (*strukt.name).clone(),
            fields,
        })
    }

    fn kind_enum_to_trace(enumerate: &kind::Enum) -> TraceType {
        let variants = enumerate
            .variants
            .iter()
            .map(|variant| rtt::Variant {
                name: (*variant.name).clone(),
                ty: kind_to_trace(&variant.kind),
                discriminant: variant.discriminant,
            })
            .collect();
        TraceType::Enum(rtt::Enum {
            name: (*enumerate.name).clone(),
            variants,
            discriminant_layout: enumerate.discriminant_layout.into(),
        })
    }
}
