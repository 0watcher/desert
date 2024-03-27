# 🏜️ desert
A simple wrapper for rusqlite that gets rid off hardcoding queries and adds support for asynchronus operations. It's not complete because doesn't support all possible features available in sqlite.

## Installation
```
cargo add desert && cargo add serde_derive && cargo add tokio
``` 

## Example
```rust
use desert::{Sql, async::Db};
use serde_derive::{Serialize, Deserialize};
use tokio::*;

#[derive(Serialize, Deserialize, Debug)]
struct Person {
    id: u32,
    name: String,
    email: String,
    favourite_animal: String
}

#[tokio::main]
async fn main() -> Result<()> {
    let tb = Db::mem().table::<Person>("persons");

    tb.insert_one(Person{
        id: 1,
        name: "somename",
        email: "somemail@mail.com",
        favourite_animal: "dog"
    });

    let result = tb.select(SQL::Distinct + SQL::Where("favourite_animal = dog")).await?;

    println!("{:?}", result.unwrap()[0]);
}
```
