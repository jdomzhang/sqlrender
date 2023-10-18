# Turbosql

An easy sql generator.

- Schema auto-defined by your Rust `struct`s
- Basic `INSERT`/`SELECT`/`UPDATE`/`DELETE` sql statements
- Use complex SQL if that's your jam

## Usage

```rust
use turbosql::Turbosql;

#[derive(Turbosql, Default)]
struct Person {
 id: Option<u64>,
 name: Option<String>,
 age: Option<u32>,
 awesomeness: Option<f64>,
 #[turbosql(skip)]
 skipped: Option<bool>,
 deleted_at: Option<String>,
}

fn main() {
 let mut person = Person {
  id: None,
  name: Some("Bob".to_string()),
  age: Some(24),
  awesomeness: Some(0.5),
  skipped: Some(true),
  ..Person::default()
 };

 {
  let sql = person.select_sql().unwrap();
  println!("select sql: {}", sql);

  let sql = person.scoped_select_sql().unwrap();
  println!("scoped select sql: {}", sql);
 }

 {
  let sql = person.insert_sql().unwrap();
  println!("insert sql: {}", sql);
 }

 {
  person.id = Some(1);
  let sql = person.update_sql().unwrap();
  println!("update sql: {}", sql);
 }

 {
  let sql = person.delete_sql().unwrap();
  println!("delete sql: {}", sql);

  let sql = person.soft_delete_sql().unwrap();
  println!("soft delete sql: {}", sql);
 }

 {
  let str = person.table_name();
  println!("table name: {}", str);
 }
}

```
