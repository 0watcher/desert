use core::panic;
use std::{marker::PhantomData, ops::Add, path::Path};

use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

pub enum Sql<'a> {
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

    pub fn table<V: Serialize>(&mut self, table_name: &str) {
        Table {
            db: &self,
            table_name: table_name,
            _marker: PhantomData::<V>,
        };
    }
}

pub struct Table<'a, T: Serialize> {
    pub(crate) db: &'a Db,
    table_name: &'a str,
    _marker: PhantomData<T>,
}

impl<'a, T: Serialize + Deserialize<'a>> Table<'a, T> {
    pub fn new<V: Serialize>(table_name: &str, db: &Db) {
        Table {
            db: db,
            table_name: table_name,
            _marker: PhantomData::<V>,
        };
    }

    pub fn set_db(&mut self, db: &'a Db) {
        self.db = &db;
    }

    // pub fn select<V: Deserialize<'a>>(
    //     &mut self,
    //     options: Vec<Sql>,
    // ) -> Result<Vec<T>, rusqlite::Error> {
    //     let conn = &self.db.conn;
    //
    //     let mut stmt = conn.prepare(&make_select_query(&self.table_name, options))?;
    //
    //     let rows = stmt.query_map([], |row| {
    //         let json_string: String = row.get(0)?;
    //         serde_json::from_str(&json_string).unwrap()
    //     })?;
    //
    //     let mut results = Vec::new();
    //     for row in rows {
    //         results.push(row?);
    //     }
    //
    //     Ok(results)
    // }
    //
    // pub fn partial_select(&mut self, column_names: &[&str], options: Vec<Sql>) {
    //     let conn = &self.db.conn;
    // }

    pub fn insert_one(&mut self, object: &T) -> Result<(), rusqlite::Error> {
        let conn = &self.db.conn;

        conn.execute(
            "INSERT INTO ?1 VALUES (?2)",
            (self.table_name, serde_json::to_string(&object).unwrap()),
        )?;

        Ok(())
    }

    pub fn insert_many(&mut self, objects: &[T]) -> Result<(), rusqlite::Error> {
        let conn = &self.db.conn;

        objects.iter().try_for_each(|object| {
            conn.execute(
                "INSERT INTO ?1 VALUES ?2",
                (self.table_name, serde_json::to_string(&object).unwrap()),
            )
            .map(|_| ())
        })?;

        Ok(())
    }

    pub fn update_one(&mut self, object: &mut T, option: Sql) -> Result<(), rusqlite::Error> {
        let conn = &self.db.conn;

        conn.execute(
            "UPDATE ?1 SET ?2 WHERE ?3",
            (
                self.table_name,
                serde_json::to_string(&object).unwrap(),
                if let Sql::Where(condition) = option {
                    condition
                } else {
                    panic!("Only 'Where' is allowed for update!");
                },
            ),
        )?;

        Ok(())
    }

    pub fn update_many(&mut self, objects: &[T], option: Sql) -> Result<(), rusqlite::Error> {
        let conn = &self.db.conn;

        objects.iter().try_for_each(|object| {
            conn.execute(
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
        let conn = &self.db.conn;

        conn.execute(
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
        let conn = &self.db.conn;

        objects.iter().try_for_each(|object| {
            conn.execute(
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
        let conn = &self.db.conn;

        let count: u32 = conn.query_row("SELECT COUNT(*) FROM table_name", [], |row| row.get(0))?;

        Ok(count)
    }
}

fn make_select_query(table_name: &str, options: Vec<Sql>) -> String {
    let mut is_distinct = false;
    let mut is_ordered = false;
    let mut is_whereable = false;

    let query = format!("SELECT * FROM {} ", table_name);

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
