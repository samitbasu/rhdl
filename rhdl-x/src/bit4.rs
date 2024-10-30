#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TraceBit {
    Zero,
    One,
    X,
    Z,
}

#[cfg(test)]
mod tests {
    use smallvec::SmallVec;

    use super::*;
    #[test]
    fn test_performance_vec_tracebit() {
        let tic = std::time::Instant::now();
        let mut count = 0;
        let mut db = vec![];
        for i in 0..1000000 {
            let mut v = Vec::new();
            let n = rand::random::<usize>() % 100;
            for ndx in 0..n {
                match ndx % 4 {
                    0 => v.push(TraceBit::Zero),
                    1 => v.push(TraceBit::One),
                    2 => v.push(TraceBit::X),
                    3 => v.push(TraceBit::Z),
                    _ => unreachable!(),
                }
            }
            v.push(TraceBit::Zero);
            v.push(TraceBit::One);
            v.push(TraceBit::X);
            v.push(TraceBit::Z);
            count += v.iter().filter(|x| **x == TraceBit::X).count();
            if count % 2 == 1 {
                db.push(v);
            }
        }
        let toc = std::time::Instant::now();
        eprintln!("Vec<TraceBit>: {:?}", toc - tic);
        eprintln!("Count: {}", count);
        eprintln!("DB Size: {}", db.len());
    }

    #[test]
    fn test_performance_smallvec_tracebit() {
        let tic = std::time::Instant::now();
        let mut count = 0;
        let mut db = vec![];
        for i in 0..1000000 {
            let mut v = SmallVec::<[TraceBit; 8]>::new();
            let n = rand::random::<usize>() % 100;
            for ndx in 0..n {
                match ndx % 4 {
                    0 => v.push(TraceBit::Zero),
                    1 => v.push(TraceBit::One),
                    2 => v.push(TraceBit::X),
                    3 => v.push(TraceBit::Z),
                    _ => unreachable!(),
                }
            }
            v.push(TraceBit::Zero);
            v.push(TraceBit::One);
            v.push(TraceBit::X);
            v.push(TraceBit::Z);
            count += v.iter().filter(|x| **x == TraceBit::X).count();
            if count % 2 == 1 {
                db.push(v);
            }
        }
        let toc = std::time::Instant::now();
        eprintln!("SmallVec<TraceBit>: {:?}", toc - tic);
        eprintln!("Count: {}", count);
        eprintln!("DB Size: {}", db.len());
    }
}
