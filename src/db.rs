use core::{ops::Deref, panic};
use std::rc::{Rc, Weak};
use std::{marker::PhantomData, path::Path};

use rusqlite::{Connection, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_rusqlite::*;

use crate::sql::*;

pub struct DbInner {
    pub(crate) conn: Connection,
}

impl DbInner {
    pub fn open(path: &mut Path) -> Self {
        Self {
            conn: Connection::open(path).unwrap(),
        }
    }

    pub fn mem() -> Self {
        DbInner {
            conn: Connection::open_in_memory().unwrap(),
        }
    }
}

impl Deref for DbInner {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}

#[derive(Clone)]
pub struct Db {
    pub(crate) inner: Rc<DbInner>,
}

impl Db {
    pub fn open(path: &mut Path) -> Self {
        Self {
            inner: Rc::new(DbInner::open(path)),
        }
    }

    pub fn mem() -> Self {
        Self {
            inner: Rc::new(DbInner::mem()),
        }
    }

    pub fn table<'a, V: Serialize + DeserializeOwned + Default>(
        &self,
        table_name: &'a str,
    ) -> Table<V> {
        Table::<V>::new(Rc::downgrade(&self.inner), table_name)
    }
}

pub struct Table<T> {
    pub(crate) db: Weak<DbInner>,
    table_name: String,
    _marker: PhantomData<T>,
}

impl<T> Table<T>
where
    T: Serialize + Default,
{
    pub fn new(db: Weak<DbInner>, table_name: &str) -> Self {
        let db_ = db.upgrade().unwrap();

        db_.execute(&make_create_query::<T>(&table_name), ())
            .unwrap();

        Table {
            db: db,
            table_name: table_name.to_owned(),
            _marker: PhantomData::default(),
        }
    }

    pub fn insert_one(&mut self, object: &T) -> Result<(), rusqlite::Error> {
        let db_ = self.db.upgrade().unwrap();
        let query = format!("INSERT INTO {} VALUES (?2)", self.table_name);
        db_.execute(&query, to_params(&object).unwrap())?;
        Ok(())
    }

    pub fn insert_many(&mut self, objects: &[T]) -> Result<(), rusqlite::Error> {
        let db_ = self.db.upgrade().unwrap();
        let query = format!("INSERT INTO {} VALUES (?2)", self.table_name);
        objects
            .iter()
            .try_for_each(|object| db_.execute(&query, to_params(&object).unwrap()).map(|_| ()))?;

        Ok(())
    }

    pub fn update_one(&mut self, object: &mut T, option: Sql) -> Result<(), rusqlite::Error> {
        let db_ = self.db.upgrade().unwrap();
        let mut query = format!("UPDATE {} SET ?1", self.table_name);
        if let Sql::Where(condition) = option {
            query.push_str(" WHERE ");
            query.push_str(condition);
        } else {
            panic!("Only 'Where' is allowed for update!");
        }

        db_.execute(&query, to_params(&object).unwrap())?;

        Ok(())
    }

    pub fn update_many(&mut self, objects: &[T], option: Sql) -> Result<(), rusqlite::Error> {
        let db_ = self.db.upgrade().unwrap();
        let mut query = format!("UPDATE {} SET ?1", self.table_name);
        if let Sql::Where(condition) = option {
            query.push_str(" WHERE ");
            query.push_str(condition);
        } else {
            panic!("Only 'Where' is allowed for update!");
        }

        objects
            .iter()
            .try_for_each(|object| db_.execute(&query, to_params(&object).unwrap()).map(|_| {}))?;

        Ok(())
    }

    pub fn delete(&mut self, option: Sql) -> Result<(), rusqlite::Error> {
        let db_ = self.db.upgrade().unwrap();
        let mut query = format!("DELETE FROM {} WHERE ", self.table_name);
        if let Sql::Where(condition) = option {
            query.push_str(condition);
        } else {
            panic!("Only 'Where' is allowed for delete!");
        }

        db_.execute(&query, ())?;

        Ok(())
    }

    pub fn count_rows(&mut self) -> Result<u32, rusqlite::Error> {
        let db_ = self.db.upgrade().unwrap();
        let count: u32 = db_.query_row("SELECT COUNT(*) FROM table_name", [], |row| row.get(0))?;

        Ok(count)
    }

    pub(crate) fn serialize_fields(&mut self) -> Vec<String> {
        let json = serde_json::to_string(&T::default()).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        parsed
            .clone()
            .as_object()
            .unwrap()
            .keys()
            .map(|k| k.to_owned())
            .collect::<Vec<String>>()
    }
}

impl<T> Table<T>
where
    T: DeserializeOwned,
{
    pub fn select(&mut self, options: OptVec<Sql>) -> Result<Vec<T>, rusqlite::Error> {
        let db_ = self.db.upgrade().unwrap();
        let mut stmt = db_.prepare(&make_select_query(&self.table_name, &[], options))?;

        let mut result = from_rows::<T>(stmt.query([]).unwrap());

        let mut rows = Vec::new();
        while let Some(el) = result.next() {
            rows.push(el.unwrap());
        }

        Ok(rows)
    }

    pub fn partial_select(
        &mut self,
        column_names: &[&str],
        options: OptVec<Sql>,
    ) -> Result<Vec<T>, rusqlite::Error> {
        let db_ = self.db.upgrade().unwrap();
        let mut stmt = db_.prepare(&make_select_query(&self.table_name, column_names, options))?;

        let mut result = from_rows::<T>(stmt.query([]).unwrap());

        let mut rows = Vec::new();
        while let Some(el) = result.next() {
            rows.push(el.unwrap());
        }

        Ok(rows)
    }
}
