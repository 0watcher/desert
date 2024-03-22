use std::{collections::HashMap, marker::PhantomData, ops::Add, path::Path};

use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};

pub enum SelectOpt<'a> {
    All,
    Distinct,
    Where(&'a str),
    OrderBy(bool),
}

impl<'a> Add<SelectOpt<'a>> for SelectOpt<'a> {
    type Output = OptVec<SelectOpt<'a>>;

    fn add(self, rhs: Self) -> Self::Output {
        OptVec(vec![self, rhs])
    }
}

impl<'a> Add<OptVec<SelectOpt<'a>>> for SelectOpt<'a> {
    type Output = OptVec<SelectOpt<'a>>;

    fn add(self, rhs: OptVec<SelectOpt<'a>>) -> Self::Output {
        let mut v = vec![self];
        v.extend(rhs.0);
        OptVec(v)
    }
}

struct OptVec<T>(pub Vec<T>);

impl<'a> Add<SelectOpt<'a>> for OptVec<SelectOpt<'a>> {
    type Output = OptVec<SelectOpt<'a>>;

    fn add(mut self, rhs: SelectOpt) -> Self::Output {
        self.0.push(rhs);
        self
    }
}

impl<'a> Add<OptVec<SelectOpt<'a>>> for OptVec<SelectOpt<'a>> {
    type Output = OptVec<SelectOpt<'a>>;

    fn add(mut self, rhs: OptVec<SelectOpt<'a>>) -> Self::Output {
        self.0.extend(rhs.0);
        self
    }
}

pub enum UpdateOpt {
    Where(String),
}

pub struct Db {
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

    pub(crate) fn get(&mut self) -> &Connection {
        &self.conn
    }
}

pub struct Table<'a, T> {
    pub(crate) db: &'a Db,
    table_name: &'a str,
    _marker: PhantomData<T>,
}

impl<'a, T> Table<'a, T> {
    pub fn new(table_name: &str) {}
    pub fn set_db(db: &Db) {}

    pub fn select<S>(&mut self, what: S, options: Vec<SelectOpt>) -> Result<T, rusqlite::Error> {
        let conn = self.db.get();
        conn.prepare("SELECT");
    }

    pub fn insert_one(&mut self, object: &T) -> Result<(), rusqlite::Error> {}

    pub fn insert_many(&mut self, objects: &[T]) -> Result<(), rusqlite::Error> {}

    pub fn update_one(&mut self, object: &mut T) -> Result<(), rusqlite::Error> {}

    pub fn update_many(&mut self, objects: &[T]) -> Result<(), rusqlite::Error> {}

    pub fn delete_one(&mut self, object: T) -> Result<(), rusqlite::Error> {}

    pub fn delete_many(&mut self, objects: &[T]) -> Result<(), rusqlite::Error> {}

    pub fn count_rows(&mut self) -> Result<u32, rusqlite::Error> {}
}

fn tokenize_json<'a, T: Serialize>(json_text: T) -> HashMap<&'a str, &'a str> {
}
