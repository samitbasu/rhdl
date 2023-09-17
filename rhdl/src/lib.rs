pub mod basic_logger;
pub mod bits;
pub mod core;

pub use crate::bits::Bits;
pub use crate::core::Digital;
pub use crate::core::Kind;
pub use crate::core::LogBuilder;
pub use crate::core::LoggerImpl;
pub use crate::core::TagID;

#[cfg(test)]
mod tests {

    use rhdl_core::Logger;

    use super::*;

    #[test]
    fn test_vcd_basic() {
        #[derive(Clone, Copy, PartialEq)]
        pub struct Simple {
            a: bool,
            b: Bits<8>,
        }

        impl Digital for Simple {
            fn static_kind() -> Kind {
                Kind::make_struct(vec![
                    Kind::make_field("a", Kind::make_bits(1)),
                    Kind::make_field("b", Kind::make_bits(8)),
                ])
            }
            fn bin(self) -> Vec<bool> {
                let mut result = vec![self.a];
                result.extend(self.b.bin());
                result
            }
            fn allocate<T: Digital>(tag: TagID<T>, builder: impl LogBuilder) {
                <bool as Digital>::allocate(tag, builder.namespace("a"));
                <Bits<8> as Digital>::allocate(tag, builder.namespace("b"));
            }
            fn record<T: Digital>(&self, tag: TagID<T>, mut logger: impl LoggerImpl) {
                self.a.record(tag, &mut logger);
                self.b.record(tag, &mut logger);
            }
            fn skip<T: Digital>(tag: TagID<T>, mut logger: impl LoggerImpl) {
                <bool as Digital>::skip(tag, &mut logger);
                <Bits<8> as Digital>::skip(tag, &mut logger);
            }
        }

        let mut builder = basic_logger::Builder::default();
        let tag = builder.tag::<Simple>("simple");
        let simple = Simple {
            a: true,
            b: Bits::from(0b10101010),
        };
        let mut logger = builder.build();
        logger.set_time_in_fs(0);
        logger.log(tag, simple);
        logger.set_time_in_fs(1_000_000);
        let simple = Simple {
            a: false,
            b: Bits::from(0b01010101),
        };
        logger.log(tag, simple);
        let mut vcd_file = std::fs::File::create("test.vcd").unwrap();
        logger.vcd(&mut vcd_file).unwrap();
    }
}
