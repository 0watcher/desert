# 🏜️ desert

![Version](https://img.shields.io/badge/version-0.0.1-blue.svg)
![Status](https://img.shields.io/badge/build-passing-green.svg)
![Status](https://img.shields.io/badge/tests-failing-red.svg)

A simple wrapper for rusqlite with mongodb-like API. It's not complete because doesn't support all possible features available in sqlite.
I made this before i realized that Diesel crate exists. It's not finished and no longer maintained.

## Installation
```
cargo add desert serde_derive
``` 

## Example
```rust
use desert::{Sql, Db};
use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct Person {
    id: u32,
    name: String,
    email: String,
    favourite_animal: String
}

fn main() {
    let tb = Db::mem().table::<Person>("persons");

    tb.insert_one(Person{
        id: 1,
        name: "somename",
        email: "somemail@mail.com",
        favourite_animal: "dog"
    });

    let result = tb.select(SQL::Distinct + SQL::Where("favourite_animal = dog"));

    println!("{:?}", result.unwrap()[0]);
}
```
