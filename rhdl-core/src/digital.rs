use rhdl_bits::{Bits, SignedBits};

use crate::{logger::LoggerImpl, Kind, LogBuilder, TagID};

/// This is the core trait for all of `RHDL` data elements.  If you
/// want to use a data type in the hardware part of the design,
/// it must implement this trait.  
pub trait Digital: Copy + PartialEq + Sized + Clone {
    fn static_kind() -> Kind;
    fn kind(self) -> Kind {
        Self::static_kind()
    }
    fn bin(self) -> Vec<bool>;
    fn allocate<T: Digital>(tag: TagID<T>, builder: impl LogBuilder);
    fn record<T: Digital>(&self, tag: TagID<T>, logger: impl LoggerImpl);
}

impl Digital for bool {
    fn static_kind() -> Kind {
        Kind::make_bits(1)
    }
    fn bin(self) -> Vec<bool> {
        vec![self]
    }
    fn allocate<T: Digital>(tag: TagID<T>, builder: impl LogBuilder) {
        builder.allocate(tag, 1);
    }
    fn record<T: Digital>(&self, tag: TagID<T>, mut logger: impl LoggerImpl) {
        logger.write_bool(tag, *self);
    }
}

impl<const N: usize> Digital for Bits<N> {
    fn static_kind() -> Kind {
        Kind::make_bits(N)
    }
    fn bin(self) -> Vec<bool> {
        self.to_bools()
    }
    fn allocate<T: Digital>(tag: TagID<T>, builder: impl LogBuilder) {
        builder.allocate(tag, N);
    }
    fn record<T: Digital>(&self, tag: TagID<T>, mut logger: impl LoggerImpl) {
        logger.write_bits(tag, self.raw());
    }
}

impl<const N: usize> Digital for SignedBits<N> {
    fn static_kind() -> Kind {
        Kind::make_bits(N)
    }
    fn bin(self) -> Vec<bool> {
        self.as_unsigned().to_bools()
    }
    fn allocate<T: Digital>(tag: TagID<T>, builder: impl LogBuilder) {
        builder.allocate(tag, N);
    }
    fn record<T: Digital>(&self, tag: TagID<T>, mut logger: impl LoggerImpl) {
        logger.write_bits(tag, self.as_unsigned().raw());
    }
}

// Add blanket implementation for tuples up to size 4.
impl<T0: Digital, T1: Digital> Digital for (T0, T1) {
    fn static_kind() -> Kind {
        Kind::make_tuple(vec![T0::static_kind(), T1::static_kind()])
    }
    fn bin(self) -> Vec<bool> {
        let mut v = self.0.bin();
        v.extend(self.1.bin());
        v
    }
    fn allocate<T: Digital>(tag: TagID<T>, builder: impl LogBuilder) {
        T0::allocate(tag, builder.namespace("0"));
        T1::allocate(tag, builder.namespace("1"));
    }
    fn record<T: Digital>(&self, tag: TagID<T>, mut logger: impl LoggerImpl) {
        self.0.record(tag, &mut logger);
        self.1.record(tag, &mut logger);
    }
}

impl<T0: Digital, T1: Digital, T2: Digital> Digital for (T0, T1, T2) {
    fn static_kind() -> Kind {
        Kind::make_tuple(vec![
            T0::static_kind(),
            T1::static_kind(),
            T2::static_kind(),
        ])
    }
    fn bin(self) -> Vec<bool> {
        let mut v = self.0.bin();
        v.extend(self.1.bin());
        v.extend(self.2.bin());
        v
    }
    fn allocate<T: Digital>(tag: TagID<T>, builder: impl LogBuilder) {
        T0::allocate(tag, builder.namespace("0"));
        T1::allocate(tag, builder.namespace("1"));
        T2::allocate(tag, builder.namespace("2"));
    }
    fn record<T: Digital>(&self, tag: TagID<T>, mut logger: impl LoggerImpl) {
        self.0.record(tag, &mut logger);
        self.1.record(tag, &mut logger);
        self.2.record(tag, &mut logger);
    }
}

impl<T0: Digital, T1: Digital, T2: Digital, T3: Digital> Digital for (T0, T1, T2, T3) {
    fn static_kind() -> Kind {
        Kind::make_tuple(vec![
            T0::static_kind(),
            T1::static_kind(),
            T2::static_kind(),
            T3::static_kind(),
        ])
    }
    fn bin(self) -> Vec<bool> {
        let mut v = self.0.bin();
        v.extend(self.1.bin());
        v.extend(self.2.bin());
        v.extend(self.3.bin());
        v
    }
    fn allocate<T: Digital>(tag: TagID<T>, builder: impl LogBuilder) {
        T0::allocate(tag, builder.namespace("0"));
        T1::allocate(tag, builder.namespace("1"));
        T2::allocate(tag, builder.namespace("2"));
        T3::allocate(tag, builder.namespace("3"));
    }
    fn record<T: Digital>(&self, tag: TagID<T>, mut logger: impl LoggerImpl) {
        self.0.record(tag, &mut logger);
        self.1.record(tag, &mut logger);
        self.2.record(tag, &mut logger);
        self.3.record(tag, &mut logger);
    }
}

impl<T: Digital, const N: usize> Digital for [T; N] {
    fn static_kind() -> Kind {
        Kind::make_array(T::static_kind(), N)
    }
    fn bin(self) -> Vec<bool> {
        let mut v = Vec::new();
        for x in self.iter() {
            v.extend(x.bin());
        }
        v
    }
    fn allocate<U: Digital>(tag: TagID<U>, builder: impl LogBuilder) {
        for i in 0..N {
            T::allocate(tag, builder.namespace(&format!("{}", i)));
        }
    }
    fn record<U: Digital>(&self, tag: TagID<U>, mut logger: impl LoggerImpl) {
        for x in self.iter() {
            x.record(tag, &mut logger);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::kind::{DiscriminantAlignment, Variant};

    #[test]
    #[allow(dead_code)]
    fn test_digital_enum() {
        #[derive(Copy, Clone, PartialEq)]
        enum State {
            Init,
            Boot,
            Running,
            Stop,
            Boom,
        }
        impl Digital for State {
            fn static_kind() -> Kind {
                Kind::make_enum(
                    vec![
                        Variant {
                            name: "Init".to_string(),
                            discriminant: 0,
                            kind: Kind::Empty,
                        },
                        Variant {
                            name: "Boot".to_string(),
                            discriminant: 1,
                            kind: Kind::Empty,
                        },
                        Variant {
                            name: "Running".to_string(),
                            discriminant: 2,
                            kind: Kind::Empty,
                        },
                        Variant {
                            name: "Stop".to_string(),
                            discriminant: 3,
                            kind: Kind::Empty,
                        },
                        Variant {
                            name: "Boom".to_string(),
                            discriminant: 4,
                            kind: Kind::Empty,
                        },
                    ],
                    Some(3),
                    DiscriminantAlignment::LSB,
                )
            }
            fn bin(self) -> Vec<bool> {
                match self {
                    Self::Init => rhdl_bits::bits::<3>(0).to_bools(),
                    Self::Boot => rhdl_bits::bits::<3>(1).to_bools(),
                    Self::Running => rhdl_bits::bits::<3>(2).to_bools(),
                    Self::Stop => rhdl_bits::bits::<3>(3).to_bools(),
                    Self::Boom => rhdl_bits::bits::<3>(4).to_bools(),
                }
            }
            fn allocate<L: Digital>(tag: TagID<L>, builder: impl LogBuilder) {
                builder.allocate(tag, 0);
            }
            fn record<L: Digital>(&self, tag: TagID<L>, mut logger: impl LoggerImpl) {
                match self {
                    Self::Init => logger.write_string(tag, stringify!(Init)),
                    Self::Boot => logger.write_string(tag, stringify!(Boot)),
                    Self::Running => logger.write_string(tag, stringify!(Running)),
                    Self::Stop => logger.write_string(tag, stringify!(Stop)),
                    Self::Boom => logger.write_string(tag, stringify!(Boom)),
                }
            }
        }
        let val = State::Boom;
        assert_eq!(val.bin(), rhdl_bits::bits::<3>(4).to_bools());
        assert_eq!(
            val.kind(),
            Kind::make_enum(
                vec![
                    Variant {
                        name: "Init".to_string(),
                        discriminant: 0,
                        kind: Kind::Empty,
                    },
                    Variant {
                        name: "Boot".to_string(),
                        discriminant: 1,
                        kind: Kind::Empty,
                    },
                    Variant {
                        name: "Running".to_string(),
                        discriminant: 2,
                        kind: Kind::Empty,
                    },
                    Variant {
                        name: "Stop".to_string(),
                        discriminant: 3,
                        kind: Kind::Empty,
                    },
                    Variant {
                        name: "Boom".to_string(),
                        discriminant: 4,
                        kind: Kind::Empty,
                    },
                ],
                Some(3),
                DiscriminantAlignment::LSB,
            )
        );
    }
}
