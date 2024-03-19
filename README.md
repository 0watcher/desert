# desert
 Simple serde wrapper for rusqlite, that prevent you among others from harcoding queries. It isn't complete because I'am not handling all possible query cases available in SQL.

# Cargo Add
```cargo add desert``` 

# Example
```rust
#[derive(Serialize)]
struct Person {
    id: u32,
    name: String,
    email: String,
}

Db::mem().insert_one(Person);
```