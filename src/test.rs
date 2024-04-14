#[cfg(test)]
mod test {
    use crate::db::*;
    use crate::sql::*;
    use serde_derive::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Default, PartialEq, Debug)]
    struct Person {
        id: u32,
        name: String,
        email: String,
        favourite_animal: String,
    }

    #[test]
    fn create_table_test() {
        let query = 
        "CREATE TABLE IF NOT EXISTS persons (email TEXT, favourite_animal TEXT, id INTEGER, name TEXT)";
        assert_eq!(make_create_query::<Person>("persons"), query);
    }

    #[test]
    fn db_functionality() {
        let db = Db::mem();
        let mut tb = db.table::<Person>("persons");

        let person = Person {
            id: 1,
            name: "somename".to_string(),
            email: "somemail@mail.com".to_string(),
            favourite_animal: "dog".to_string(),
        };

        assert!(tb.insert_one(&person).is_ok());

        let result = tb.select(Sql::Distinct + Sql::Where("favourite_animal = dog"));
        assert_eq!(result.unwrap()[0], person)
    }
}
