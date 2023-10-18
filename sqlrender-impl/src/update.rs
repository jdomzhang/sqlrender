use super::Table;
use proc_macro_error::abort_call_site;
use quote::quote_spanned;

/// UPDATE tablename SET name1=?, name2=?... WHERE id=?
pub(super) fn update(table: &Table) -> proc_macro2::TokenStream {
	if table.columns[0].name != "id" {
		abort_call_site!("First field must be `id: Option<u64>`");
	}

	let sql = makesql_update(table);

	// super::validate_sql_or_abort(&sql);

	quote_spanned! { table.span =>
		fn update_sql(&self) -> Result<String, ::sqlrender::Error> {
			assert!(self.id.is_some());
			let statement = #sql;
			Ok(statement.to_string())
		}
	}
}

fn makesql_update(table: &Table) -> String {
	format!(
		"UPDATE {} SET {} WHERE id=?",
		table.name,
		table.columns.iter().collect::<Vec<_>>()[1..]
			.iter()
			.map(|c| format!("{}=?", c.name.as_str()))
			.collect::<Vec<_>>()
			.join(", ")
	)
}
