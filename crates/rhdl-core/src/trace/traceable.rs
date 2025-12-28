use crate::{BitX, Digital, Kind, TraceBit, TypedBits};

/// This trait captures the [Digital] interface that is dyn-compatible.
/// It is used internally by the tracing system to handle traced values
pub trait Traceable {
    /// Get the number of bits for the stored value
    fn bits(&self) -> usize;
    /// Get the number of trace bits for the stored value
    fn trace_bits(&self) -> usize;
    /// Get the [Kind] of the stored value
    fn kind(&self) -> Kind;
    /// Get the [TraceType] of the stored value
    fn trace_type(&self) -> rhdl_trace_type::TraceType {
        self.kind().into()
    }
    /// Get the binary representation of the stored value as a vector of [BitX]
    fn bin(&self) -> Box<[BitX]>;
    /// Get the binary representation of the stored value as a vector of [TraceBit]
    fn trace(&self) -> Box<[TraceBit]>;
    /// Get the [TypedBits] representation of the stored value
    fn typed_bits(&self) -> TypedBits;
}

impl<T: Digital> Traceable for T {
    fn bits(&self) -> usize {
        T::BITS
    }
    fn trace_bits(&self) -> usize {
        T::BITS
    }
    fn kind(&self) -> Kind {
        T::static_kind()
    }
    fn bin(&self) -> Box<[BitX]> {
        Digital::bin(*self)
    }
    fn trace(&self) -> Box<[TraceBit]> {
        Digital::trace(*self)
    }
    fn typed_bits(&self) -> TypedBits {
        Digital::typed_bits(*self)
    }
}
