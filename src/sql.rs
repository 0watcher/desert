use serde::Serialize;
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

pub(crate) fn serialize_fields<V: Serialize + Default>() -> Vec<String> {
    let json = serde_json::to_string(&V::default()).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    parsed
        .clone()
        .as_object()
        .unwrap()
        .keys()
        .map(|k| k.to_owned())
        .collect::<Vec<String>>()
}
