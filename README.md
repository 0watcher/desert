# 🏜️ desert
 Simple async schemaless wrapper for rusqlite, that prevent you among others from hardcoding queries. It isn't complete because I'am not handling all possible features available in SQLite.

## Cargo Add
```
cargo add desert && cargo add serde_derive && cargo add tokio
``` 

## Example
```rust
#[derive(Serialize, Deserialize, Debug)]
struct Person {
    name: String,
    email: String,
    favourite_animal: String
}

let tb = Db::mem().table<Person>("persons");

tb.insert_one(Person{
    name: "somename",
    email: "somemail@mail.com",
    favourite_animal: "dog"
});

let result = tb.select(SQL::Distinct + SQL::Where("favourite_animal = dog")).await?;

println!("{:?}", result.unwrap()[0]);
```
