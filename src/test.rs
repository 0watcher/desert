#[cfg(test)]
mod test {
    use crate::db::sync::*;
    use crate::Sql;
    use serde_derive::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, Default)]
    struct Person {
        id: u32,
        name: String,
        email: String,
        favourite_animal: String,
    }

    #[test]
    fn db_functionality() {
        let mut tb = Db::mem().table::<Person>("persons");

        tb.insert_one(&Person {
            id: 1,
            name: "somename".to_string(),
            email: "somemail@mail.com".to_string(),
            favourite_animal: "dog".to_string(),
        });

        let result = tb.select(Sql::Distinct + Sql::Where("favourite_animal = dog"));
        println!("{:?}", result.unwrap()[0]);
    }
}
