use smallvec::SmallVec;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BitVector {
    value: SmallVec<[u32; 2]>,
    len: usize,
}

impl BitVector {
    fn reserve(&mut self, len: usize) {
        while self.value.len() <= (len / 32) {
            self.value.push(0);
        }
    }
    fn set(&mut self, ndx: usize) {
        let digit = ndx / 32;
        let bit = ndx % 32;
        self.value[digit] |= 1 << bit;
    }
    pub fn extend(&mut self, other: &Self) {
        self.reserve(self.len + other.len);
        for (ndx, bit) in other.bits().enumerate() {
            if bit {
                self.set(self.len + ndx);
            }
        }
        self.len += other.len;
    }
    pub fn push(&mut self, x: bool) {
        self.reserve(self.len + 1);
        if x {
            self.set(self.len);
        }
        self.len += 1;
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn get(&self, ndx: usize) -> bool {
        if (ndx + 1) > self.len {
            panic!("Out of bounds access");
        }
        let digit = ndx / 32;
        let bit = ndx % 32;
        self.value[digit] & (1 << bit) != 0
    }
    pub fn bits(&self) -> impl Iterator<Item = bool> + '_ {
        (0..self.len).map(move |ndx| self.get(ndx))
    }
}

// Test BitVector against the standard Rust Vec<bool> implementation
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_basic_bitvector() {
        // Choose a random number of bits between 0 and 100
        for _ in 0..1_000_000 {
            let n = rand::random::<usize>() % 100;
            let mut bv = BitVector::default();
            let mut v = Vec::new();
            for _ in 0..n {
                let x = rand::random::<bool>();
                bv.push(x);
                v.push(x);
            }
            assert_eq!(bv.len(), v.len());
            for i in 0..n {
                assert_eq!(bv.get(i), v[i]);
            }
        }
    }

    #[test]
    fn test_speed_bitvector() {
        let tic = std::time::Instant::now();
        let rand_bits = (0..200).map(|_| rand::random::<bool>()).collect::<Vec<_>>();
        let mut total_bits = 0;
        let mut data = Vec::new();
        for _ in 0..1_000_000 {
            let n = rand::random::<usize>() % 100;
            let mut bv = BitVector::default();
            rand_bits.iter().take(n).for_each(|x| bv.push(*x));
            total_bits += bv.len();
            data.push(bv);
        }
        let toc = std::time::Instant::now();
        eprintln!("BitVector: {:?}", toc - tic);
        eprintln!("Items: {}", data.len());
    }

    #[test]
    fn test_speed_vec_bool() {
        let tic = std::time::Instant::now();
        let rand_bits = (0..200).map(|_| rand::random::<bool>()).collect::<Vec<_>>();
        let mut total_bits = 0;
        let mut data = Vec::new();
        for _ in 0..1_000_000 {
            let n = rand::random::<usize>() % 100;
            let mut v = Vec::new();
            rand_bits.iter().take(n).for_each(|x| v.push(*x));
            total_bits += v.len();
            data.push(v);
        }
        let toc = std::time::Instant::now();
        eprintln!("Vec<bool>: {:?}", toc - tic);
        eprintln!("Items: {}", data.len());
    }

    #[test]
    fn test_speed_smallvec_bool() {
        let tic = std::time::Instant::now();
        let rand_bits = (0..200).map(|_| rand::random::<bool>()).collect::<Vec<_>>();
        let mut total_bits = 0;
        let mut data = Vec::new();
        for _ in 0..1_000_000 {
            let n = rand::random::<usize>() % 100;
            let mut v = SmallVec::<[bool; 8]>::new();
            rand_bits.iter().take(n).for_each(|x| v.push(*x));
            total_bits += v.len();
            data.push(v);
        }
        let toc = std::time::Instant::now();
        eprintln!("SmallVec<bool>: {:?}", toc - tic);
        eprintln!("Items: {}", data.len());
    }
}
