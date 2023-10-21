use std::fmt::Display;

pub fn splice<T: Display>(elems: &[T], sep: &str) -> String {
    elems
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(sep)
}
