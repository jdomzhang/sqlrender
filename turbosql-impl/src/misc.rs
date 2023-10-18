use super::Table;
use quote::quote_spanned;

pub(super) fn misc(table: &Table) -> proc_macro2::TokenStream {
	let sql = &table.name;

	quote_spanned! { table.span =>
		fn table_name(&self) -> &'static str {
			#sql
		}
	}
}
