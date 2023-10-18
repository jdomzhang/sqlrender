use super::Table;
use proc_macro_error::abort_call_site;
use quote::quote_spanned;

pub(super) fn ddl(table: &Table) -> proc_macro2::TokenStream {
	if table.columns[0].name != "id" {
		abort_call_site!("First field must be `id: Option<u64>`");
	}

	let sql = makesql_ddl(table);

	quote_spanned! { table.span =>
		fn ddl_sql(&self) -> Result<String, ::sqlrender::Error> {
			assert!(self.id.is_some());
			let statement = #sql;
			Ok(statement.to_string())
		}
	}
}

fn makesql_ddl(table: &Table) -> String {
	format!(
		r#"CREATE TABLE `{0}` (
{1},
	KEY `idx_{0}_deleted_at` (`deleted_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;"#,
		table.name,
		table
			.columns
			.iter()
			.map(|c| format!("\t`{}` {}", c.name, c.sql_type))
			.collect::<Vec<_>>()
			.join(",\n")
	)
}
