//! This crate provides SqlRender's procedural macros.
//!
//! Please refer to the `sqlrender` crate for how to set this up.

#![forbid(unsafe_code)]

use once_cell::sync::Lazy;
use proc_macro2::Span;
use proc_macro_error::{abort, abort_call_site, proc_macro_error};
use quote::{quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Fields, FieldsNamed, Ident, Meta, Token};

mod ddl;
mod delete;
mod insert;
mod misc;
mod select;
mod update;

#[derive(Debug, Clone)]
struct Table {
	ident: Ident,
	span: Span,
	name: String,
	columns: Vec<Column>,
}

impl ToTokens for Table {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let ident = &self.ident;
		tokens.extend(quote!(#ident));
	}
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Column {
	ident: Ident,
	span: Span,
	name: String,
	rust_type: String,
	sql_type: &'static str,
}

static U8_ARRAY_RE: Lazy<regex::Regex> =
	Lazy::new(|| regex::Regex::new(r"^Option < \[u8 ; \d+\] >$").unwrap());

/// Derive this on a `struct` to create a corresponding table and `SqlRender` trait methods.
#[proc_macro_derive(SqlRender, attributes(sqlrender))]
#[proc_macro_error]
pub fn sqlrender_derive_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	if is_rust_analyzer() {
		return quote!().into();
	}

	// parse tokenstream and set up table struct

	let input = parse_macro_input!(input as DeriveInput);
	let table_span = input.span();
	let table_ident = input.ident;
	let table_name = table_ident.to_string().to_lowercase();

	let fields = match input.data {
		Data::Struct(ref data) => match data.fields {
			Fields::Named(ref fields) => fields,
			Fields::Unnamed(_) | Fields::Unit => unimplemented!(),
		},
		Data::Enum(_) | Data::Union(_) => unimplemented!(),
	};

	let table = Table {
		ident: table_ident,
		span: table_span,
		name: table_name.clone(),
		columns: extract_columns(fields),
	};

	// create trait functions

	let fn_select = select::select(&table);
	let fn_insert = insert::insert(&table);
	let fn_update = update::update(&table);
	let fn_delete = delete::delete(&table);
	let fn_ddl = ddl::ddl(&table);
	let fn_misc = misc::misc(&table);

	// output tokenstream

	let output = quote! {
		#[cfg(not(target_arch = "wasm32"))]
		impl ::sqlrender::SqlRender for #table {
			#fn_select
			#fn_insert
			#fn_update
			#fn_delete
			#fn_ddl
			#fn_misc
		}
	};

	output.into()
}

/// Convert syn::FieldsNamed to our Column type.
fn extract_columns(fields: &FieldsNamed) -> Vec<Column> {
	let columns = fields
		.named
		.iter()
		.filter_map(|f| {
			// Skip (skip) fields

			for attr in &f.attrs {
				if attr.path().is_ident("sqlrender") {
					for meta in attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated).unwrap() {
						match meta {
							Meta::Path(path) if path.is_ident("skip") => {
								// TODO: For skipped fields, Handle derive(Default) requirement better
								// require Option and manifest None values
								return None;
							}
							_ => ()
						}
					}
				}
			}

			let ident = &f.ident;
			let name = ident.as_ref().unwrap().to_string();

			let ty = &f.ty;
			let ty_str = quote!(#ty).to_string();

			let sql_type = match (
				name.as_str(),
				if U8_ARRAY_RE.is_match(&ty_str) { "Option < [u8; _] >" } else { ty_str.as_str() },
			) {
				("id", "Option < u64 >") => "BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY",
				(_, "Option < i8 >") => "INT",
				(_, "Option < u8 >") => "INT",
				(_, "Option < i16 >") => "INT",
				(_, "Option < u16 >") => "INT",
				(_, "Option < i32 >") => "INT",
				(_, "Option < u32 >") => "INT",
				(_, "Option < i64 >") => "BIGINT",
				(_, "Option < u64 >") => "BIGINT UNSIGNED",
				(_, "Option < f64 >") => "DOUBLE",
				(_, "Option < f32 >") => "DOUBLE",
				(_, "Option < bool >") => "TINYINT",
				(_, "Option < String >") => "LONGTEXT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci",
				(_, "Option < DateTime < FixedOffset > >") => "TIMESTAMP",
				// SELECT LENGTH(blob_column) ... will be null if blob is null
				(_, "Option < Blob >") => "BLOB",
				(_, "Option < Vec < u8 > >") => "BLOB",
				(_, "Option < [u8; _] >") => "BLOB",
				_ => {
					if ty_str.starts_with("Option < ") {
						"TEXT" // JSON-serialized
					} else {
						abort!(
							ty,
							"SqlRender types must be wrapped in Option for forward/backward schema compatibility. Try: Option<{}>",
							ty_str
						)
					}
				}
			};

			Some(Column {
				ident: ident.clone().unwrap(),
				span: ty.span(),
				rust_type: ty_str,
				name,
				sql_type,
			})
		})
		.collect::<Vec<_>>();

	if !matches!(
		columns.iter().find(|c| c.name == "id"),
		Some(Column { sql_type: "BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY", .. })
	) {
		abort_call_site!("derive(SqlRender) structs must include a 'id: Option<u64>' field")
	};

	columns
}

fn is_rust_analyzer() -> bool {
	std::env::current_exe()
		.unwrap()
		.file_stem()
		.unwrap()
		.to_string_lossy()
		.starts_with("rust-analyzer")
}

#[cfg(test)]
mod tests {
	use super::*;
	// use chrono::{DateTime, FixedOffset};
	use syn::parse_quote;

	#[test]
	fn test_extract_columns() {
		let fields_named = parse_quote!({
			id: Option<u64>,
			name: Option<String>,
			age: Option<u32>,
			awesomeness: Option<f64>,
			#[sqlrender(skip)]
			skipped: Option<bool>
			// deleted_at: DateTime<FixedOffset>
		});

		let columns = extract_columns(&fields_named);

		assert_eq!(columns.len(), 4);

		assert_eq!(columns[0].name, "id");
		assert_eq!(columns[0].rust_type, "Option < u64 >");
		assert_eq!(columns[0].sql_type, "BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY");

		assert_eq!(columns[1].name, "name");
		assert_eq!(columns[1].rust_type, "Option < String >");
		assert_eq!(columns[1].sql_type, "LONGTEXT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci");

		assert_eq!(columns[2].name, "age");
		assert_eq!(columns[2].rust_type, "Option < u32 >");
		assert_eq!(columns[2].sql_type, "INT");

		assert_eq!(columns[3].name, "awesomeness");
		assert_eq!(columns[3].rust_type, "Option < f64 >");
		assert_eq!(columns[3].sql_type, "DOUBLE");

		assert!(!columns.iter().any(|c| c.name == "skipped"));
	}
}
