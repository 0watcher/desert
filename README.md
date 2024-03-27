# 🏜️ desert
A simple async wrapper for rusqlite that gets rid of, among other things, hardcoding queries. It is not complete because I do not support all possible features available in SQLite.

## Installation
```
cargo add desert && cargo add serde_derive && cargo add tokio
``` 

## Example
```rust
#[derive(Serialize, Deserialize, Debug)]
struct Person {
    id: u32,
    name: String,
    email: String,
    favourite_animal: String
}

let tb = Db::mem().table<Person>("persons");

tb.insert_one(Person{
    id: 1,
    name: "somename",
    email: "somemail@mail.com",
    favourite_animal: "dog"
});

let result = tb.select(SQL::Distinct + SQL::Where("favourite_animal = dog")).await?;

println!("{:?}", result.unwrap()[0]);
```
