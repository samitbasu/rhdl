#[derive(Copy, Clone, Default)]
struct SignedBits<const N: usize>(i128);

impl<const N: usize> SignedBits<N> {
    fn discriminant(self) -> Self {
        self
    }

    fn as_unsigned(self) -> Bits<N> {
        Bits(self.0 as u128)
    }
}

impl<const N: usize> std::ops::Add for SignedBits<N> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        SignedBits(self.0 + rhs.0)
    }
}

impl<const N: usize> std::ops::BitAnd for SignedBits<N> {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        SignedBits(self.0 & rhs.0)
    }
}

impl<const N: usize> PartialEq<SignedBits<N>> for SignedBits<N> {
    fn eq(&self, other: &SignedBits<N>) -> bool {
        self.0.eq(&other.0)
    }
}

impl<const N: usize> PartialOrd<SignedBits<N>> for SignedBits<N> {
    fn partial_cmp(&self, other: &SignedBits<N>) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

#[derive(Copy, Clone, Default)]
struct Bits<const N: usize>(u128);

impl<const N: usize> Bits<N> {
    fn discriminant(self) -> Self {
        self
    }

    fn as_signed(self) -> SignedBits<N> {
        SignedBits(self.0 as i128)
    }
}

impl From<bool> for Bits<1> {
    fn from(x: bool) -> Self {
        Bits(x as u128)
    }
}

impl<const N: usize> std::ops::BitAnd for Bits<N> {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        Bits(self.0 & rhs.0)
    }
}

impl<const N: usize> std::ops::Add for Bits<N> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Bits(self.0 + rhs.0)
    }
}

impl<const N: usize> std::ops::Add<u128> for Bits<N> {
    type Output = Self;

    fn add(self, rhs: u128) -> Self {
        Bits(self.0 + rhs)
    }
}

impl<const N: usize> std::ops::Add<Bits<N>> for u128 {
    type Output = Bits<N>;

    fn add(self, rhs: Bits<N>) -> Bits<N> {
        Bits(self + rhs.0)
    }
}

impl<const N: usize> PartialEq<Bits<N>> for Bits<N> {
    fn eq(&self, other: &Bits<N>) -> bool {
        self.0.eq(&other.0)
    }
}

impl<const N: usize> PartialEq<u128> for Bits<N> {
    fn eq(&self, other: &u128) -> bool {
        self.0.eq(other)
    }
}

impl<const N: usize> PartialEq<Bits<N>> for u128 {
    fn eq(&self, other: &Bits<N>) -> bool {
        self.eq(&other.0)
    }
}

impl<const N: usize> PartialOrd<Bits<N>> for Bits<N> {
    fn partial_cmp(&self, other: &Bits<N>) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<const N: usize> PartialOrd<u128> for Bits<N> {
    fn partial_cmp(&self, other: &u128) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl<const N: usize> PartialOrd<Bits<N>> for u128 {
    fn partial_cmp(&self, other: &Bits<N>) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&other.0)
    }
}

impl<T, const M: usize, const N: usize> std::ops::Index<Bits<N>> for [T; M] {
    type Output = T;
    fn index(&self, index: Bits<N>) -> &Self::Output {
        &self[index.0 as usize]
    }
}

impl<T, const M: usize, const N: usize> std::ops::IndexMut<Bits<N>> for [T; M] {
    fn index_mut(&mut self, index: Bits<N>) -> &mut Self::Output {
        &mut self[index.0 as usize]
    }
}

fn signed<const N: usize>(x: i128) -> SignedBits<N> {
    SignedBits(x)
}

fn bits<const N: usize>(x: u128) -> Bits<N> {
    Bits(x)
}

trait Bitify<T: Sized> {
    fn bitify(self) -> T;
}

impl<const N: usize> Bitify<SignedBits<N>> for i128 {
    fn bitify(self) -> SignedBits<N> {
        SignedBits(self)
    }
}

impl<const N: usize> Bitify<Bits<N>> for u128 {
    fn bitify(self) -> Bits<N> {
        Bits(self)
    }
}

impl Bitify<i128> for i32 {
    fn bitify(self) -> i128 {
        self as i128
    }
}

impl Bitify<u128> for i32 {
    fn bitify(self) -> u128 {
        self as u128
    }
}

impl Bitify<Bits<1>> for bool {
    fn bitify(self) -> Bits<1> {
        Bits(self as u128)
    }
}

trait Any {
    fn any(self) -> Bits<1>;
}

impl<const N: usize> Any for Bits<N> {
    fn any(self) -> Bits<1> {
        (self.0 != 0).into()
    }
}

trait All {
    fn all(self) -> Bits<1>;
}

impl<const N: usize> All for Bits<N> {
    fn all(self) -> Bits<1> {
        (self.0 == u128::MAX >> (128 - N)).into()
    }
}

trait Xor {
    fn xor(self) -> Bits<1>;
}

impl<const N: usize> Xor for Bits<N> {
    fn xor(self) -> Bits<1> {
        (self.0.count_ones() % 2 == 1).into()
    }
}

fn select<T>(cond: Bits<1>, a: T, b: T) -> T {
    if cond.0 != 0 {
        a
    } else {
        b
    }
}
