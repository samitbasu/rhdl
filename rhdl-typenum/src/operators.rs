use std::ops::{Add, Sub};

use crate::{digits::D1, traits::Len};

pub type Add1<A> = <A as Add<D1>>::Output;
pub type Sub1<A> = <A as Sub<D1>>::Output;
pub type Sum<A, B> = <A as Add<B>>::Output;
pub type Length<T> = <T as Len>::Output;
pub type Diff<A, B> = <A as Sub<B>>::Output;
