#[cfg(all(not(feature = "test"), any(test, doctest)))]
compile_error!("turbosql must be tested with '--features test'");

#[cfg(all(feature = "test", doctest))]
doc_comment::doctest!("../../README.md");

use itertools::{
 EitherOrBoth::{Both, Left, Right},
 Itertools,
};
use rusqlite::{Connection, OpenFlags};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

// these re-exports are used in macro expansions

#[doc(hidden)]
pub use once_cell::sync::Lazy;
#[doc(hidden)]
pub use rusqlite::{
 params, types::FromSql, types::FromSqlResult, types::ToSql, types::ToSqlOutput, types::Value,
 types::ValueRef, Error, OptionalExtension, Result,
};
#[doc(hidden)]
pub use serde::Serialize;
pub use turbosql_impl::{execute, select, Turbosql};

/// Wrapper for `Vec<u8>` that may one day impl `Read`, `Write` and `Seek` traits.
pub type Blob = Vec<u8>;

#[derive(Clone, Debug, Deserialize, Default)]
struct MigrationsToml {
 migrations_append_only: Option<Vec<String>>,
 output_generated_schema_for_your_information_do_not_edit: Option<String>,
}

struct DbPath {
 path: PathBuf,
 opened: bool,
}

static __DB_PATH: Lazy<Mutex<DbPath>> = Lazy::new(|| {
 #[cfg(not(feature = "test"))]
 let path = {
  let exe_stem = std::env::current_exe().unwrap().file_stem().unwrap().to_owned();
  let exe_stem_lossy = exe_stem.to_string_lossy();

  let path = directories_next::ProjectDirs::from("org", &exe_stem_lossy, &exe_stem_lossy)
   .unwrap()
   .data_dir()
   .to_owned();

  std::fs::create_dir_all(&path).unwrap();

  path.join(exe_stem).with_extension("sqlite")
 };

 #[cfg(feature = "test")]
 let path = Path::new(":memory:").to_owned();

 Mutex::new(DbPath { path, opened: false })
});

#[doc(hidden)]
pub static __TURBOSQL_DB: Lazy<Mutex<Connection>> = Lazy::new(|| {
 cfg_if::cfg_if! {
  if #[cfg(doc)] {
   let toml_decoded: MigrationsToml = MigrationsToml::default();
  } else if #[cfg(feature = "test")] {
   let toml_decoded: MigrationsToml = toml::from_str(include_str!("../../test.migrations.toml")).unwrap();
  } else {
   let toml_decoded: MigrationsToml = toml::from_str(include_str!(concat!(env!("OUT_DIR"), "/../../../../../migrations.toml"))).expect("Unable to decode embedded migrations.toml");
  }
 };

 let target_migrations = toml_decoded.migrations_append_only.unwrap_or_else(Vec::new);

 // filter out comments
 let target_migrations: Vec<_> =
  target_migrations.into_iter().filter(|m| !m.starts_with("--")).collect();

 let mut db_path = __DB_PATH.lock().unwrap();

 db_path.opened = true;

 // We are handling the mutex, so SQLite can be opened in no-mutex mode; see:
 // http://sqlite.1065341.n5.nabble.com/SQLITE-OPEN-FULLMUTEX-vs-SQLITE-OPEN-NOMUTEX-td104785.html

 let conn = Connection::open_with_flags(
  &db_path.path,
  OpenFlags::SQLITE_OPEN_READ_WRITE
   | OpenFlags::SQLITE_OPEN_CREATE
   | OpenFlags::SQLITE_OPEN_NO_MUTEX,
 )
 .expect("rusqlite::Connection::open_with_flags");

 conn
  .execute_batch(
   r#"
    PRAGMA auto_vacuum=INCREMENTAL;
    PRAGMA journal_mode=WAL;
    PRAGMA wal_autocheckpoint=8000;
    PRAGMA synchronous=NORMAL;
   "#,
  )
  .expect("Execute PRAGMAs");

 conn.execute("BEGIN EXCLUSIVE TRANSACTION", params![]).unwrap();

 let _ = conn.execute("ALTER TABLE turbosql_migrations RENAME TO _turbosql_migrations", params![]);

 let result = conn.query_row(
  "SELECT sql FROM sqlite_master WHERE name = ?",
  params!["_turbosql_migrations"],
  |row| {
   let sql: String = row.get(0).unwrap();
   Ok(sql)
  },
 );

 match result {
  Err(rusqlite::Error::QueryReturnedNoRows) => {
   // no migrations table exists yet, create
   conn
    .execute_batch(
     r#"CREATE TABLE _turbosql_migrations (rowid INTEGER PRIMARY KEY, migration TEXT NOT NULL)"#,
    )
    .expect("CREATE TABLE _turbosql_migrations");
  }
  Err(err) => {
   panic!("Could not query sqlite_master table: {}", err);
  }
  Ok(_) => (),
 }

 let applied_migrations = conn
  .prepare("SELECT migration FROM _turbosql_migrations ORDER BY rowid")
  .unwrap()
  .query_map(params![], |row| {
   // let sql: String = row.get(0).unwrap();
   Ok(row.get(0).unwrap())
  })
  .unwrap()
  .map(|x: Result<String, _>| x.unwrap())
  .filter(|m| !m.starts_with("--"))
  .collect::<Vec<String>>();

 // println!("applied_migrations is: {:#?}", applied_migrations);
 // println!("target_migrations is: {:#?}", target_migrations);

 // execute migrations

 applied_migrations.iter().zip_longest(&target_migrations).for_each(|item| match item {
  Both(a, b) => {
   if a != b {
    panic!("Mismatch in Turbosql migrations! {:?} != {:?}", a, b)
   }
  }
  Left(_) => panic!("More migrations are applied than target"),
  Right(migration) => {
   // eprintln!("insert -> {:#?}", migration);
   if !migration.starts_with("--") {
    conn.execute(migration, params![]).unwrap();
   }
   conn
    .execute("INSERT INTO _turbosql_migrations(migration) VALUES(?)", params![migration])
    .unwrap();
  }
 });

 // TODO: verify schema against output_generated_schema_for_your_information_do_not_edit

 //    if sql != create_sql {
 //     println!("{}", sql);
 //     println!("{}", create_sql);
 //     panic!("Turbosql sqlite schema does not match! Delete database file to continue.");
 //    }

 conn.execute("COMMIT", params![]).unwrap();

 Mutex::new(conn)
});

/// Set the local path and filename where Turbosql will store the underlying SQLite database.
///
/// Must be called before any usage of Turbosql macros or will return an error.
pub fn set_db_path(path: &Path) -> Result<(), anyhow::Error> {
 let mut db_path = __DB_PATH.lock().unwrap();

 if db_path.opened {
  return Err(anyhow::anyhow!("Trying to set path when DB is already opened"));
 }

 db_path.path = path.to_owned();

 Ok(())
}
