use core::panic;
use std::{fmt::format, marker::PhantomData, ops::Add, path::Path};

use rusqlite::{params, Connection, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_rusqlite::*;

pub enum Sql<'a> {
    AutoIncrement,
    Distinct,
    Where(&'a str),
    OrderBy(bool),
}

impl<'a> Add<Sql<'a>> for Sql<'a> {
    type Output = OptVec<Sql<'a>>;

    fn add(self, rhs: Self) -> Self::Output {
        OptVec(vec![self, rhs])
    }
}

impl<'a> Add<OptVec<Sql<'a>>> for Sql<'a> {
    type Output = OptVec<Sql<'a>>;

    fn add(self, rhs: OptVec<Sql<'a>>) -> Self::Output {
        let mut v = vec![self];
        v.extend(rhs.0);
        OptVec(v)
    }
}

pub struct OptVec<T>(pub Vec<T>);

impl<'a> Add<Sql<'a>> for OptVec<Sql<'a>> {
    type Output = OptVec<Sql<'a>>;

    fn add(mut self, rhs: Sql<'a>) -> Self::Output {
        self.0.push(rhs);
        self
    }
}

impl<'a> Add<OptVec<Sql<'a>>> for OptVec<Sql<'a>> {
    type Output = OptVec<Sql<'a>>;

    fn add(mut self, rhs: OptVec<Sql<'a>>) -> Self::Output {
        self.0.extend(rhs.0);
        self
    }
}

pub struct Db {
    pub(crate) conn: Connection,
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

    pub fn table<'a, V: Serialize + DeserializeOwned + Default>(&'a mut self, table_name: &'a str) {
        Table::<V>::new(table_name, self);
    }
}

pub struct Table<'a, T: Serialize + Deserialize<'a> + Default> {
    pub(crate) conn: &'a Connection,
    table_name: &'a str,
    _marker: PhantomData<T>,
}

impl<'a, T: Serialize + DeserializeOwned + Default> Table<'a, T> {
    pub fn new(table_name: &'a str, db: &'a Db) -> Self {
        let mut tb = Table {
            conn: &db.conn,
            table_name: table_name,
            _marker: PhantomData::<T>,
        };

        let query = format!("CREATE TABLE IF NOT EXIST {} (?2)", table_name);
        db.conn
            .execute(&query, to_params(tb.serialize_fields()).unwrap());

        tb
    }

    pub fn set_db(&mut self, db: &'a Db) {
        self.conn = &db.conn;
    }

    pub fn select(&mut self, options: Vec<Sql>) -> Result<Vec<T>, rusqlite::Error> {
        let mut stmt = self
            .conn
            .prepare(&make_select_query(&self.table_name, &[], options))?;

        let mut result = from_rows::<T>(stmt.query([]).unwrap());

        let mut rows = Vec::new();
        while let el = result.next() {
            rows.push(el.unwrap().unwrap());
        }

        Ok(rows)
    }

    pub fn partial_select(
        &mut self,
        column_names: &[&str],
        options: Vec<Sql>,
    ) -> Result<Vec<T>, rusqlite::Error> {
        let mut stmt =
            self.conn
                .prepare(&make_select_query(&self.table_name, column_names, options))?;

        let mut result = from_rows::<T>(stmt.query([]).unwrap());

        let mut rows = Vec::new();
        while let el = result.next() {
            rows.push(el.unwrap().unwrap());
        }

        Ok(rows)
    }

    pub fn insert_one(&mut self, object: &T) -> Result<(), rusqlite::Error> {
        let query = format!("INSERT INTO {} VALUES (?2)", self.table_name);
        self.conn.execute(&query, to_params(&object).unwrap())?;
        Ok(())
    }

    pub fn insert_many(&mut self, objects: &[T]) -> Result<(), rusqlite::Error> {
        let query = format!("INSERT INTO {} VALUES (?2)", self.table_name);
        objects.iter().try_for_each(|object| {
            self.conn
                .execute(&query, to_params(&object).unwrap())
                .map(|_| ())
        })?;

        Ok(())
    }

    pub fn update_one(&mut self, object: &mut T, option: Sql) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "UPDATE ?1 SET ?2 WHERE ?3",
            (
                params![self.table_name],
                to_params(&object).unwrap(),
                if let Sql::Where(condition) = option {
                    params![&condition]
                } else {
                    panic!("Only 'Where' is allowed for update!");
                },
            ),
        )?;

        Ok(())
    }

    pub fn update_many(&mut self, objects: &[T], option: Sql) -> Result<(), rusqlite::Error> {
        objects.iter().try_for_each(|object| {
            self.conn
                .execute(
                    "UPDATE ?1 SET ?2 WHERE ?3",
                    (
                        self.table_name,
                        serde_json::to_string(&object).unwrap(),
                        if let Sql::Where(condition) = option {
                            condition
                        } else {
                            panic!("Only 'Where' is allowed for update");
                        },
                    ),
                )
                .map(|_| ())
        })?;

        Ok(())
    }

    pub fn delete_one(&mut self, object: T, option: Sql) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "DELETE FROM ?1 WHERE ?2",
            (
                self.table_name,
                serde_json::to_string(&object).unwrap(),
                if let Sql::Where(condition) = option {
                    condition
                } else {
                    panic!("Only 'Where' is allowed for delete!");
                },
            ),
        )?;

        Ok(())
    }

    pub fn delete_many(&mut self, objects: &[T], option: Sql) -> Result<(), rusqlite::Error> {
        objects.iter().try_for_each(|object| {
            self.conn
                .execute(
                    "DELETE FROM ?1 WHERE ?2",
                    (
                        self.table_name,
                        serde_json::to_string(&object).unwrap(),
                        if let Sql::Where(condition) = option {
                            condition
                        } else {
                            panic!("Only 'Where' is allowed for delete!");
                        },
                    ),
                )
                .map(|_| ())
        })?;

        Ok(())
    }

    pub fn count_rows(&mut self) -> Result<u32, rusqlite::Error> {
        let count: u32 = self
            .conn
            .query_row("SELECT COUNT(*) FROM table_name", [], |row| row.get(0))?;

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

fn make_select_query(table_name: &str, column_names: &[&str], options: Vec<Sql>) -> String {
    let mut query = String::new();
    if column_names.is_empty() {
        query = format!("SELECT * FROM {} ", table_name);
    } else {
        query = format!("SELECT () FROM {}", table_name);

        let mut names = String::new();
        for name in column_names.iter() {
            names.push_str(name);
            names.push_str(",");
        }

        query.insert_str(7, &names);
    }

    let mut is_distinct = false;
    let mut is_ordered = false;
    let mut is_whereable = false;

    options.iter().for_each(|option| {
        match option {
            Sql::Distinct => {
                if !is_distinct {
                    is_distinct = true;
                }
                panic!("Distinct can be used only once at current query");
            }
            Sql::Where(condition) => {
                if !is_whereable {
                    is_whereable = true;
                } else {
                    // Handle multiple where conditions here if needed
                }
            }
            Sql::OrderBy(ord_type) => {
                if !is_ordered {
                    is_ordered = true;
                }
                panic!("OrderBy can be used only once at current query");
            }
        }
    });

    query
}
