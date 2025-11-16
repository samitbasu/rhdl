//! Utilities to parse rustc style literals

/// Parse a u128 from a string literal
///
/// Note that:
///
/// - If the string starts with 0b or 0B, it is parsed as a binary literal
/// - If the string starts with 0o or 0O, it is parsed as an octal literal
/// - If the string starts with 0x or 0X, it is parsed as a hexadecimal literal
/// - Otherwise, it is parsed as a decimal literal
///
/// - Also, any "_" characters are ignored in the string
use std::num::ParseIntError;
pub fn parse_u128(x: &str) -> Result<u128, ParseIntError> {
    let x = x.replace("_", "");
    if let Some(rest) = x.strip_prefix("0b") {
        u128::from_str_radix(rest, 2)
    } else if let Some(rest) = x.strip_prefix("0B") {
        u128::from_str_radix(rest, 2)
    } else if let Some(rest) = x.strip_prefix("0o") {
        u128::from_str_radix(rest, 8)
    } else if let Some(rest) = x.strip_prefix("0O") {
        u128::from_str_radix(rest, 8)
    } else if let Some(rest) = x.strip_prefix("0x") {
        u128::from_str_radix(rest, 16)
    } else if let Some(rest) = x.strip_prefix("0X") {
        u128::from_str_radix(rest, 16)
    } else {
        x.parse::<u128>()
    }
}

/// Parse an i128 from a string literal
///
/// Note that:
///
/// - If the string starts with 0b or 0B, it is parsed as a binary literal
/// - If the string starts with 0o or 0O, it is parsed as an octal literal
/// - If the string starts with 0x or 0X, it is parsed as a hexadecimal literal
/// - Otherwise, it is parsed as a decimal literal
/// - Also, any "_" characters are ignored in the string
///
pub fn parse_i128(x: &str) -> Result<i128, ParseIntError> {
    let x = x.replace("_", "");
    if let Some(rest) = x.strip_prefix("0b") {
        i128::from_str_radix(rest, 2)
    } else if let Some(rest) = x.strip_prefix("0B") {
        i128::from_str_radix(rest, 2)
    } else if let Some(rest) = x.strip_prefix("0o") {
        i128::from_str_radix(rest, 8)
    } else if let Some(rest) = x.strip_prefix("0O") {
        i128::from_str_radix(rest, 8)
    } else if let Some(rest) = x.strip_prefix("0x") {
        i128::from_str_radix(rest, 16)
    } else if let Some(rest) = x.strip_prefix("0X") {
        i128::from_str_radix(rest, 16)
    } else {
        x.parse::<i128>()
    }
}
