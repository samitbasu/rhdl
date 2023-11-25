use parking_lot::Mutex;
use rhdl_bits::alias::*;
use rhdl_core::note::{NoteKey, NoteWriter};
use rhdl_macro::Digital;
use std::hash::{Hash, Hasher};
#[derive(Default)]
struct NoteDB {
    db_bool: fnv::FnvHashMap<u32, Vec<bool>>,
    db_bits: fnv::FnvHashMap<u32, Vec<u128>>,
    db_signed: fnv::FnvHashMap<u32, Vec<i128>>,
    db_string: fnv::FnvHashMap<u32, Vec<&'static str>>,
    keys: fnv::FnvHashMap<String, u32>,
}

impl NoteWriter for NoteDB {
    fn write_bool(&mut self, key: impl rhdl_core::note::NoteKey, value: bool) {
        self.log_bool(key, value);
    }

    fn write_bits(&mut self, key: impl rhdl_core::note::NoteKey, value: u128, len: u8) {
        self.log_u128(key, value);
    }

    fn write_signed(&mut self, key: impl rhdl_core::note::NoteKey, value: i128, len: u8) {
        self.log_i128(key, value);
    }

    fn write_string(&mut self, key: impl rhdl_core::note::NoteKey, value: &'static str) {
        self.log_string(key, value);
    }
}

impl NoteDB {
    fn log_bool(&mut self, key: impl rhdl_core::note::NoteKey, value: bool) {
        let mut hasher = fnv::FnvHasher::default();
        key.hash(&mut hasher);
        let key_hash = hasher.finish() as u32;
        if let Some(values) = self.db_bool.get_mut(&key_hash) {
            values.push(value);
        } else {
            self.keys.insert(key.as_string().to_string(), key_hash);
            self.db_bool.insert(key_hash, vec![value]);
        }
    }
    fn log_u128(&mut self, key: impl rhdl_core::note::NoteKey, value: u128) {
        let mut hasher = fnv::FnvHasher::default();
        key.hash(&mut hasher);
        let key_hash = hasher.finish() as u32;
        if let Some(values) = self.db_bits.get_mut(&key_hash) {
            values.push(value);
        } else {
            self.keys.insert(key.as_string().to_string(), key_hash);
            self.db_bits.insert(key_hash, vec![value]);
        }
    }
    fn log_i128(&mut self, key: impl rhdl_core::note::NoteKey, value: i128) {
        let mut hasher = fnv::FnvHasher::default();
        key.hash(&mut hasher);
        let key_hash = hasher.finish() as u32;
        if let Some(values) = self.db_signed.get_mut(&key_hash) {
            values.push(value);
        } else {
            self.keys.insert(key.as_string().to_string(), key_hash);
            self.db_signed.insert(key_hash, vec![value]);
        }
    }
    fn log_string(&mut self, key: impl rhdl_core::note::NoteKey, value: &'static str) {
        let mut hasher = fnv::FnvHasher::default();
        key.hash(&mut hasher);
        let key_hash = hasher.finish() as u32;
        if let Some(values) = self.db_string.get_mut(&key_hash) {
            values.push(value);
        } else {
            self.keys.insert(key.as_string().to_string(), key_hash);
            self.db_string.insert(key_hash, vec![value]);
        }
    }
}

lazy_static::lazy_static! {
    static ref DB: Mutex<NoteDB> = Mutex::new(NoteDB::default());
}

pub fn note(key: impl NoteKey, value: impl rhdl_core::Digital) {
    let db = &mut *DB.lock();
    value.note(key, db);
}

pub fn dump() {
    let db = DB.lock();
    for (key, key_hash) in &db.keys {
        if let Some(values) = db.db_bool.get(key_hash) {
            println!("{}: {}", key, values.len());
        }
        if let Some(values) = db.db_bits.get(key_hash) {
            println!("{}: {}", key, values.len());
        }
        if let Some(values) = db.db_signed.get(key_hash) {
            println!("{}: {}", key, values.len());
        }
        if let Some(values) = db.db_string.get(key_hash) {
            println!("{}: {}", key, values.len());
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Digital, Default)]
pub struct Foo {
    pub field1: b4,
    pub field2: b2,
    pub field3: (b4, b6),
}
