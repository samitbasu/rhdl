use rhdl::prelude::*;

#[kernel]
pub fn unpack<T: Digital + Default>(opt: Option<T>) -> (bool, T) {
    match opt {
        None => (false, T::default()),
        Some(t) => (true, t),
    }
}

#[kernel]
pub fn pack<T: Digital>(valid: bool, data: T) -> Option<T> {
    if valid {
        Some(data)
    } else {
        None
    }
}

#[kernel]
pub fn is_some<T: Digital>(x: Option<T>) -> bool {
    match x {
        Some(_) => true,
        None => false,
    }
}
