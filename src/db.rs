use std::path::Path;

use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};

pub enum SelectOpt {
    All,
    Distinct,
    Where(String),
    OrderBy(bool),
}

pub enum UpdateOpt {
    Where(String),
}

struct Db {
    conn: Connection,
}

impl Db {
    pub fn open(path: &mut Path) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        Ok(Db { conn })
    }

    pub fn mem() -> Result<Self, rusqlite::Error> {
        let conn = Connection::open_in_memory()?;
        Ok(Db { conn })
    }

    pub fn create(&mut self) {}

    pub fn select<T: Serialize>(&mut self) {}

    pub fn insert_one<'a, T: Deserialize<'a>>(&mut self) {}

    pub fn insert_many<'a, T: Deserialize<'a>>(&mut self) {}

    pub fn update_one<'a, T: Deserialize<'a>>(&mut self) {}

    pub fn update_many<'a, T: Deserialize<'a>>(&mut self) {}

    pub fn delete_one<'a, T: Deserialize<'a>>(&mut self) {}

    pub fn delete_many<'a, T: Deserialize<'a>>(&mut self) {}

    pub fn count_rows<T: Serialize>(&mut self) {}
}
