use super::Table;
use quote::quote_spanned;

pub(super) fn select(table: &Table) -> proc_macro2::TokenStream {
	let sql = makesql_select(table);
	// super::validate_sql_or_abort(&sql);

	let scoped_sql = makesql_scoped_select(table);
	// super::validate_sql_or_abort(&scoped_sql);

	quote_spanned! { table.span =>
		fn select_sql(&self) -> Result<String, ::sqlrender::Error> {
			let statement = #sql;
			Ok(statement.to_string())
		}

		fn scoped_select_sql(&self) -> Result<String, ::sqlrender::Error> {
			let statement = #scoped_sql;
			Ok(statement.to_string())
		}
	}
}

fn makesql_select(table: &Table) -> String {
	let sql = format!("SELECT `{0}`.* FROM `{0}`", table.name);

	sql
}

fn makesql_scoped_select(table: &Table) -> String {
	let sql = format!("SELECT `{0}`.* FROM `{0}` WHERE `{0}`.deleted_at is null", table.name);

	sql
}
