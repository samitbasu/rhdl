pub mod bits;
pub mod core;
//pub mod synchronous;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod test_utils;

/* #[cfg(test)]
mod test_upstream;

#[cfg(test)]
mod test_downstream;
 */
pub use crate::bits::Bits;
pub use crate::bits::SignedBits;
pub use crate::core::Digital;
pub use crate::core::Kind;
pub use rhdl_macro::kernel;
pub use rhdl_macro::Digital;
