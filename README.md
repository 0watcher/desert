# 🏜️ desert
 Simple serde wrapper for rusqlite, that prevent you among others from harcoding queries. It isn't complete because I'am not handling all possible query cases available in SQL.

# Cargo Add
```cargo add desert``` 

# Example
```rust
#[derive(Serialize)]
struct Person {
    name: String,
    email: String,
    favourite_animal: String
}

let db = Db::mem();
let tb = Table::new(&db, "persons");

tb.insert_one(Person{
    name: "somename",
    email: "somemail@mail.com",
    favourite_animal: "dog"
});

let result = tb.select(["name"], SQL::Distinct + SQL::Where("favourite_animal = 'dog'"));
```