use super::Table;
use quote::quote_spanned;

/// INSERT INTO tablename (name1, name2...) VALUES (?1, ?2...)
pub(super) fn insert(table: &Table) -> proc_macro2::TokenStream {
	let sql = makesql_insert(table);

	// super::validate_sql_or_abort(&sql);

	quote_spanned! { table.span =>
		fn insert_sql(&self) -> Result<String, ::sqlrender::Error> {
			assert!(self.id.is_none());
			let statement = #sql;
			Ok(statement.to_string())
		}
	}
}

fn makesql_insert(table: &Table) -> String {
	let mut sql = format!("INSERT INTO {} (", table.name);
	sql += table.columns.iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join(", ").as_str();
	sql += ") VALUES (";
	sql += table.columns.iter().map(|_| "?").collect::<Vec<_>>().join(", ").as_str();
	sql += ")";

	sql
}
