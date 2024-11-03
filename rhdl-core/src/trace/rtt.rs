use rhdl_trace_type::TraceType;

use crate::Kind;

use crate::types::kind;
use rhdl_trace_type as rtt;

// Given a Kind, attempt to translate it into a RHDL Trace Type
// This is the "Default" implementation, which can be overridden
// by a particular DIGITAL type as needed.
pub(crate) fn kind_to_trace(kind: &Kind) -> TraceType {
    match kind {
        Kind::Array(array) => kind_array_to_trace(array),
        Kind::Tuple(tuple) => kind_tuple_to_trace(tuple),
        Kind::Struct(strukt) => kind_struct_to_trace(strukt),
        Kind::Enum(enumerate) => kind_enum_to_trace(enumerate),
        Kind::Bits(len) => Some(TraceType::Bits(*len)),
        Kind::Signed(len) => Some(TraceType::Signed(*len)),
        Kind::Signal(ky, _) => kind_to_trace(ky),
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
        .map(|(name, kind)| (name.clone(), kind_to_trace(kind)))
        .collect();
    TraceType::Struct(rtt::Struct {
        name: strukt.name.clone(),
        fields,
    })
}
