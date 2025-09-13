// Based on https://matklad.github.io/2020/03/22/fast-simple-rust-interner.html
// Sort of a :facepalm: for not thinking of this myself.

use std::{collections::HashMap, hash::Hash};

#[derive(Default, Debug, PartialEq, Eq, Hash)]
pub struct InternKey<T> {
    index: u32,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Clone for InternKey<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for InternKey<T> {}

pub struct Intern<T: Hash + Eq + Clone> {
    map: HashMap<T, InternKey<T>>,
    vec: Vec<T>,
}

impl<T: Hash + Eq + Clone> Default for Intern<T> {
    fn default() -> Self {
        Intern {
            map: HashMap::new(),
            vec: Vec::new(),
        }
    }
}

impl<T: Hash + Eq + Clone> Intern<T> {
    pub fn alloc(&mut self, value: T) -> InternKey<T> {
        if let Some(key) = self.map.get(&value) {
            return *key;
        }
        let key = InternKey {
            index: self.vec.len() as u32,
            _marker: std::marker::PhantomData,
        };
        self.vec.push(value.clone());
        self.map.insert(value, key);
        key
    }
}

impl<T: Hash + Eq + Clone> std::ops::Index<InternKey<T>> for Intern<T> {
    type Output = T;
    fn index(&self, key: InternKey<T>) -> &T {
        &self.vec[key.index as usize]
    }
}
