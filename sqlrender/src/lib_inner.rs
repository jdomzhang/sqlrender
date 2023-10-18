#[doc(hidden)]
pub use once_cell::sync::Lazy;

#[doc(hidden)]
pub use serde::Serialize;
#[doc(hidden)]
pub use serde_json;
pub use sqlrender_impl::SqlRender;

// /// Wrapper for `Vec<u8>` that may one day impl `Read`, `Write` and `Seek` traits.
// pub type Blob = Vec<u8>;

/// `#[derive(SqlRender)]` generates impls for this trait.
pub trait SqlRender {
	// get select sql
	fn select_sql(&self) -> Result<String, Error>;

	// get scoped select sql
	fn scoped_select_sql(&self) -> Result<String, Error>;

	// get insert sql
	fn insert_sql(&self) -> Result<String, Error>;

	// get update sql
	fn update_sql(&self) -> Result<String, Error>;

	// get delete sql
	fn delete_sql(&self) -> Result<String, Error>;

	// get soft delete sql
	fn soft_delete_sql(&self) -> Result<String, Error>;

	// get ddl sql
	fn ddl_sql(&self) -> Result<String, Error>;

	// get table name
	fn table_name(&self) -> &'static str;
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error(transparent)]
	SerdeJson(#[from] serde_json::Error),
	#[error("SqlRender Error: {0}")]
	OtherError(&'static str),
}
