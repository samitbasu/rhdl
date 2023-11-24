use const_fnv1a_hash::fnv1a_hash_str_32;
use fnv::FnvBuildHasher;
use std::hash::Hash;
pub trait NoteKey: Clone + Copy + Hash {
    fn as_string(&self) -> String;
}

impl NoteKey for &'static str {
    fn as_string(&self) -> String {
        self.to_string()
    }
}

impl<T: NoteKey> NoteKey for (T, &'static str) {
    fn as_string(&self) -> String {
        format!("{}{}", self.0.as_string(), self.1)
    }
}

#[derive(Default)]
struct NoteDB {
    //    db: HashMap<u64, Vec<i32>>,
    db: fnv::FnvHashMap<u32, Vec<i32>>,
    keys: HashMap<String, u32>,
}

impl NoteDB {
    fn log(&mut self, key: impl NoteKey, value: i32) {
        let mut hasher = fnv::FnvHasher::default();
        key.hash(&mut hasher);
        let key_hash = hasher.finish() as u32;
        if let Some(values) = self.db.get_mut(&key_hash) {
            values.push(value);
        } else {
            self.keys.insert(key.as_string().to_string(), key_hash);
            self.db.insert(key_hash, vec![value]);
        }
    }
}

lazy_static::lazy_static! {
    static ref DB: Mutex<NoteDB> = Mutex::new(NoteDB::default());
}

fn note(key: impl NoteKey, value: i32) {
    DB.lock().log(key, value);
}

pub fn dump() {
    let db = DB.lock();
    for (key, key_hash) in &db.keys {
        if let Some(values) = db.db.get(key_hash) {
            println!("{}: {}", key, values.len());
        }
    }
}

use anyhow::Result;
use anyhow::{anyhow, bail};
use parking_lot::Mutex;
use std::hash::Hasher;
use std::time::Instant;
use std::{collections::HashMap, fmt::Display};

pub fn func_1(key: impl NoteKey) {
    note(key, 42);
}

pub fn func_2(key: impl NoteKey, value: bool) {
    note(((key, "."), "bar"), 65);
}

pub fn func_3(key: impl NoteKey, value: i32) {
    func_2(((key, "."), "baz"), value == 0);
    note((key, "bar"), 98);
}

lazy_static::lazy_static! {
    static ref GDB: Mutex<fnv::FnvHashMap<String, Vec<i32>>> = Mutex::new(fnv::FnvHashMap::default());
}

pub fn gote(key: &str, value: i32) {
    let mut gdb = GDB.lock();
    if let Some(values) = gdb.get_mut(key) {
        values.push(value);
    } else {
        gdb.insert(key.to_string(), vec![value]);
    }
}

pub fn gump() {
    let gdb = GDB.lock();
    for (key, values) in &*gdb {
        println!("{}: {}", key, values.len());
    }
}

pub fn gunk_1(key: &str) {
    gote(key, 42);
}

pub fn gunk_2(key: &str, value: bool) {
    gote(&format!("{}{}", key, "."), 65);
}

pub fn gunk_3(key: &str, value: i32) {
    gunk_2(&format!("{}{}", key, "."), value == 0);
    gote(&format!("{}{}", key, "bar"), 98);
}
