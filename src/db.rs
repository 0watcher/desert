pub mod r#async {
    use core::panic;
    use std::{fmt::format, marker::PhantomData, path::Path};

    use rusqlite::{Connection, Result};
    use serde::{de::DeserializeOwned, Deserialize, Serialize};
    use serde_rusqlite::*;

    use crate::*;

    pub struct Db {
        pub(crate) conn: Connection,
    }

    impl Db {
        pub fn open(path: &mut Path) -> Self {
            let conn = Connection::open(path).unwrap();
            Db { conn }
        }

        pub fn mem() -> Self {
            let conn = Connection::open_in_memory().unwrap();
            Db { conn }
        }

        pub fn table<'a, V: Serialize + DeserializeOwned + Default>(
            &'a mut self,
            table_name: &'a str,
        ) -> Table<'a, V> {
            Table::<V>::new(table_name, self)
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

        pub async fn select(
            &mut self,
            options: OptVec<Sql<'a>>,
        ) -> Result<Vec<T>, rusqlite::Error> {
            let mut stmt = self
                .conn
                .prepare(&make_select_query(&self.table_name, &[], options))?;

            let mut result = from_rows::<T>(stmt.query([]).unwrap());

            let mut rows = Vec::new();
            while let Some(el) = result.next() {
                rows.push(el.unwrap());
            }

            Ok(rows)
        }

        pub async fn partial_select(
            &mut self,
            column_names: &[&str],
            options: OptVec<Sql<'a>>,
        ) -> Result<Vec<T>, rusqlite::Error> {
            let mut stmt =
                self.conn
                    .prepare(&make_select_query(&self.table_name, column_names, options))?;

            let mut result = from_rows::<T>(stmt.query([]).unwrap());

            let mut rows = Vec::new();
            while let Some(el) = result.next() {
                rows.push(el.unwrap());
            }

            Ok(rows)
        }

        pub async fn insert_one(&mut self, object: &T) -> Result<(), rusqlite::Error> {
            let query = format!("INSERT INTO {} VALUES (?2)", self.table_name);
            self.conn.execute(&query, to_params(&object).unwrap())?;
            Ok(())
        }

        pub async fn insert_many(&mut self, objects: &[T]) -> Result<(), rusqlite::Error> {
            let query = format!("INSERT INTO {} VALUES (?2)", self.table_name);
            objects.iter().try_for_each(|object| {
                self.conn
                    .execute(&query, to_params(&object).unwrap())
                    .map(|_| ())
            })?;

            Ok(())
        }

        pub async fn update_one(
            &mut self,
            object: &mut T,
            option: Sql<'a>,
        ) -> Result<(), rusqlite::Error> {
            let mut query = format!("UPDATE {} SET ?1", self.table_name);
            if let Sql::Where(condition) = option {
                query.push_str(" WHERE ");
                query.push_str(condition);
            } else {
                panic!("Only 'Where' is allowed for update!");
            }

            self.conn.execute(&query, to_params(&object).unwrap())?;

            Ok(())
        }

        pub async fn update_many(
            &mut self,
            objects: &[T],
            option: Sql<'a>,
        ) -> Result<(), rusqlite::Error> {
            let mut query = format!("UPDATE {} SET ?1", self.table_name);
            if let Sql::Where(condition) = option {
                query.push_str(" WHERE ");
                query.push_str(condition);
            } else {
                panic!("Only 'Where' is allowed for update!");
            }

            objects.iter().try_for_each(|object| {
                self.conn
                    .execute(&query, to_params(&object).unwrap())
                    .map(|_| {})
            })?;

            Ok(())
        }

        pub async fn delete(&mut self, option: Sql<'a>) -> Result<(), rusqlite::Error> {
            let mut query = format!("DELETE FROM {} WHERE ", self.table_name);
            if let Sql::Where(condition) = option {
                query.push_str(condition);
            } else {
                panic!("Only 'Where' is allowed for delete!");
            }

            self.conn.execute(&query, ())?;

            Ok(())
        }

        pub async fn count_rows(&mut self) -> Result<u32, rusqlite::Error> {
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
}

pub mod sync {
    use core::panic;
    use std::rc::Rc;
    use std::sync::Arc;
    use std::{fmt::format, marker::PhantomData, ops::Add, path::Path};

    use rusqlite::{Connection, Result};
    use serde::{de::DeserializeOwned, Deserialize, Serialize};
    use serde_rusqlite::*;

    use crate::*;

    #[derive(Clone)]
    pub struct Db {
        pub(crate) conn: Rc<Connection>,
    }

    impl Db {
        pub fn open(path: &mut Path) -> Self {
            let conn = Connection::open(path).unwrap();
            Db {
                conn: Rc::new(conn),
            }
        }

        pub fn mem() -> Self {
            let conn = Connection::open_in_memory().unwrap();
            Db {
                conn: Rc::new(conn),
            }
        }

        pub fn table<'a, V: Serialize + DeserializeOwned + Default>(
            &'a self,
            table_name: &'a str,
        ) -> Table<'a, V> {
            Table::<V>::new(table_name, self.clone())
        }
    }

    pub struct Table<'a, T: Serialize + Deserialize<'a> + Default> {
        pub(crate) conn: Rc<Connection>,
        table_name: &'a str,
        _marker: PhantomData<T>,
    }

    impl<'a, T: Serialize + DeserializeOwned + Default> Table<'a, T> {
        pub fn new(table_name: &'a str, db: Db) -> Self {
            let mut tb = Table {
                conn: db.conn,
                table_name: table_name,
                _marker: PhantomData::<T>,
            };

            let query = format!("CREATE TABLE IF NOT EXIST {} (?2)", table_name);
            db.conn
                .execute(&query, to_params(tb.serialize_fields()).unwrap());

            tb
        }

        pub fn set_db(&mut self, db: &'a Db) {
            self.conn = db.conn.clone();
        }

        pub fn select(&mut self, options: OptVec<Sql>) -> Result<Vec<T>, rusqlite::Error> {
            let mut stmt = self
                .conn
                .prepare(&make_select_query(&self.table_name, &[], options))?;

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
            let mut stmt =
                self.conn
                    .prepare(&make_select_query(&self.table_name, column_names, options))?;

            let mut result = from_rows::<T>(stmt.query([]).unwrap());

            let mut rows = Vec::new();
            while let Some(el) = result.next() {
                rows.push(el.unwrap());
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
            let mut query = format!("UPDATE {} SET ?1", self.table_name);
            if let Sql::Where(condition) = option {
                query.push_str(" WHERE ");
                query.push_str(condition);
            } else {
                panic!("Only 'Where' is allowed for update!");
            }

            self.conn.execute(&query, to_params(&object).unwrap())?;

            Ok(())
        }

        pub fn update_many(&mut self, objects: &[T], option: Sql) -> Result<(), rusqlite::Error> {
            let mut query = format!("UPDATE {} SET ?1", self.table_name);
            if let Sql::Where(condition) = option {
                query.push_str(" WHERE ");
                query.push_str(condition);
            } else {
                panic!("Only 'Where' is allowed for update!");
            }

            objects.iter().try_for_each(|object| {
                self.conn
                    .execute(&query, to_params(&object).unwrap())
                    .map(|_| {})
            })?;

            Ok(())
        }

        pub fn delete(&mut self, option: Sql) -> Result<(), rusqlite::Error> {
            let mut query = format!("DELETE FROM {} WHERE ", self.table_name);
            if let Sql::Where(condition) = option {
                query.push_str(condition);
            } else {
                panic!("Only 'Where' is allowed for delete!");
            }

            self.conn.execute(&query, ())?;

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
}
