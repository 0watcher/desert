use rusqlite::types::{ToSql, Value as SqliteValue};
use serde::Serialize;
use serde_json::Value;
use std::ops::{Add, Deref};

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

impl<T> Deref for OptVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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

struct QueryBuilder;

pub(crate) fn make_select_query(
    table_name: &str,
    column_names: &[&str],
    options: OptVec<Sql>,
) -> String {
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
            _ => panic!("This enum variant is not valid for select"),
        }
    });

    query
}

pub(crate) fn serialize_fields<T: Serialize + Default>() -> Vec<(String, &'static str)> {
    let json = serde_json::to_string(&T::default()).unwrap();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    parsed
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| {
            let field_type = match v {
                Value::Number(n) if n.is_i64() => "INTEGER",
                Value::Number(_) => "REAL",
                Value::String(_) => "TEXT",
                Value::Bool(_) => "BOOLEAN",
                Value::Array(_) => "BLOB",
                Value::Object(_) => "TEXT",
                _ => "UNKNOWN",
            };
            (k.to_owned(), field_type)
        })
        .collect::<Vec<(String, &'static str)>>()
}

// can be const
pub(crate) fn make_create_query<T: Serialize + Default>(table_name: &str) -> String {
    let fields = serialize_fields::<T>();

    let mut query = format!("CREATE TABLE IF NOT EXISTS {} (", table_name);

    for (name, field_type) in fields {
        query.push_str(&format!("{} {}, ", name, field_type));
    }

    // Remove the trailing comma and space
    query.pop();
    query.pop();

    query.push_str(")");

    query
}

pub(crate) fn make_insert_query<T: Serialize + Default>(table_name: &str) -> String {
    todo!()
}
pub(crate) fn make_update_query<T: Serialize + Default>(table_name: &str) -> String {
    todo!()
}
