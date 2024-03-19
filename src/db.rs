use std::path::Path;

use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};

struct Db {
    conn: Connection,
}

impl Db {
    pub fn open(&mut self, path: &mut Path) -> Self {
        Db {
            conn: self::Connection::open(path).unwrap(),
        }
    }

    pub fn mem(&mut self) -> Self {
        Db {
            conn: self::Connection::open_in_memory().unwrap(),
        }
    }

    pub fn select_all<T: Serialize>() {}

    pub fn select<T: Serialize>() {}

    pub fn insert_one<'a, T: Deserialize<'a>>() {}

    pub fn insert_many<'a, T: Deserialize<'a>>() {}

    pub fn update_one<'a, T: Deserialize<'a>>() {}

    pub fn update_many<'a, T: Deserialize<'a>>() {}

    pub fn delete_one<'a, T: Deserialize<'a>>() {}

    pub fn delete_many<'a, T: Deserialize<'a>>() {}

    pub fn count_rows<T: Serialize>() {}
}
