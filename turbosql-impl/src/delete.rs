use super::Table;
use proc_macro_error::abort_call_site;
use quote::quote_spanned;

pub(super) fn delete(table: &Table) -> proc_macro2::TokenStream {
	if table.columns[0].name != "id" {
		abort_call_site!("First field must be `id: Option<u64>`");
	}

	let sql = makesql_delete(table);
	let soft_sql = makesql_soft_delete(table);

	quote_spanned! { table.span =>
		fn delete_sql(&self) -> Result<String, ::turbosql::Error> {
			assert!(self.id.is_some());
			let statement = #sql;
			Ok(statement.to_string())
		}

		fn soft_delete_sql(&self) -> Result<String, ::turbosql::Error> {
			assert!(self.id.is_some());
			let statement = #soft_sql;
			Ok(statement.to_string())
		}
	}
}

fn makesql_soft_delete(table: &Table) -> String {
	format!("UPDATE {} SET deleted_at = now() WHERE deleted_at is null and id=?", table.name,)
}

fn makesql_delete(table: &Table) -> String {
	format!("DELETE FROM {} WHERE id=?", table.name,)
}
