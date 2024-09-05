use std::hash::{Hash, Hasher};

pub fn splice<T: std::fmt::Debug>(elems: &[T], sep: &str) -> String {
    elems
        .iter()
        .map(|x| format!("{x:?}"))
        .collect::<Vec<_>>()
        .join(sep)
}

#[derive(Default, Debug)]
pub struct IndentingFormatter {
    buffer: String,
    indent: i32,
}

impl IndentingFormatter {
    pub fn buffer(self) -> String {
        self.buffer
    }
    pub fn location(&self) -> usize {
        self.buffer.len()
    }
    pub fn write(&mut self, s: &str) {
        // Write s to the internal buffer.
        // If s contains a left brace, then increase the indent
        // if s contains a right brace, then decrease the indent
        // if s contains a newline, then add the indent
        // if s contains a semicolon, then add a newline
        // otherwise, just write the string
        for c in s.chars() {
            match c {
                '{' => {
                    self.buffer.push(c);
                    self.indent += 1;
                }
                '}' => {
                    let backup = self
                        .buffer
                        .chars()
                        .rev()
                        .take_while(|x| *x == ' ')
                        .take(3)
                        .count();
                    self.buffer.truncate(self.buffer.len() - backup);
                    self.indent -= 1;
                    self.buffer.push(c);
                }
                '\n' => {
                    self.buffer.push(c);
                    for _ in 0..self.indent {
                        self.buffer.push_str("   ");
                    }
                }
                _ => {
                    self.buffer.push(c);
                }
            }
        }
    }
}

#[test]
fn test_indenting_formatter() {
    let mut f = IndentingFormatter::default();
    f.write("hello {\n");
    f.write("let a = 2;\n");
    f.write("let b = 3;\n");
    f.write("}\n");
    println!("{}", f.buffer());
}

pub fn binary_string(x: &[bool]) -> String {
    x.iter().rev().map(|b| if *b { '1' } else { '0' }).collect()
}

// Put an underscore every 4 bits, with the first underscore at the 4th bit
// from the right (lsb).
pub fn binary_string_nibbles(x: &[bool]) -> String {
    let mut s = String::new();
    for (i, b) in x.iter().rev().enumerate() {
        if i % 4 == 0 && i != 0 {
            s.push('_');
        }
        s.push(if *b { '1' } else { '0' });
    }
    s
}

pub fn hash_id(fn_id: std::any::TypeId) -> u64 {
    // Hash the typeID into a 64 bit unsigned int
    let mut hasher = fnv::FnvHasher::default();
    fn_id.hash(&mut hasher);
    hasher.finish()
}

pub fn id<T: 'static>() -> u64 {
    hash_id(std::any::TypeId::of::<T>())
}

// This should return the number of bits
// required to hold a value of x.
pub fn clog2(x: usize) -> usize {
    if x == 0 {
        return 0;
    }
    let mut y = x;
    let mut n = 0;
    while y > 0 {
        y >>= 1;
        n += 1;
    }
    n
}

#[test]
fn test_clog2() {
    assert_eq!(clog2(0), 0);
    assert_eq!(clog2(1), 1);
    assert_eq!(clog2(2), 2);
    assert_eq!(clog2(3), 2);
    assert_eq!(clog2(255), 8);
}
