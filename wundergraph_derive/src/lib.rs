//! A helper crate implementing a bunch of custom derives for wundergraph
#![deny(missing_debug_implementations, missing_copy_implementations)]
#![warn(
    missing_docs,
    clippy::option_unwrap_used,
    clippy::result_unwrap_used,
    clippy::print_stdout,
    clippy::wrong_pub_self_convention,
    clippy::mut_mut,
    clippy::non_ascii_literal,
    clippy::similar_names,
    clippy::unicode_not_nfc,
    clippy::enum_glob_use,
    clippy::if_not_else,
    clippy::items_after_statements,
    clippy::used_underscore_binding,
    clippy::cargo_common_metadata,
    clippy::dbg_macro,
    clippy::doc_markdown,
    clippy::filter_map,
    clippy::map_flatten,
    clippy::match_same_arms,
    clippy::needless_borrow,
    clippy::needless_pass_by_value,
    clippy::option_map_unwrap_or,
    clippy::option_map_unwrap_or_else,
    clippy::redundant_clone,
    clippy::result_map_unwrap_or_else,
    clippy::unnecessary_unwrap,
    clippy::unseparated_literal_suffix,
    clippy::wildcard_dependencies
)]
extern crate proc_macro;

mod diagnostic_shim;
mod field;
mod meta;
mod model;
mod resolved_at_shim;
mod utils;

pub(crate) mod belonging_to;
pub(crate) mod build_filter_helper;
mod wundergraph_entity;
mod wundergraph_filter;
pub(crate) mod wundergraph_value;

use self::diagnostic_shim::Diagnostic;
use proc_macro::TokenStream;

/// A custom derive to implement all wundergraph related traits for a entity
/// Using this trait implies internally `#[derive(WundergraphBelongsTo)]`
/// and `#[derive(BuildFilterHelper)]`
///
/// # Type attributes
/// * **Required**:
///     * `#[table_name = "diesel_table_mod"]`: Name of the underlying diesel table.
/// * Optional:
///     * `#[primary_key(primary_key_name)]`: Names the fields that represent
///       the primary key on the underlying database table. Set
///        to `id` if not given (In this case a primary key field named `id` must exist)
///     * `/// Documentation`/`#[doc = "Documentation"]`: Set as GraphQL
///       description text.
///
/// # Field attributes
/// All attributes are optional. If no attributes are given the field name needs to
/// match the name of the field in the corresponding diesel `table!` and is used
/// a GraphQL field name
///
/// * `#[column_name = "other_name"]`: Use the given name instead of the field
///    name as column name for calling into diesels `table!`
/// * `#[wundergraph(graphql_name = "Foo")]`: Set the GraphQL name of the field
///   to the given name. If not set the field name is used as name.
/// * `#[deprecated(note = "Some Text")]`: Set as GraphQL deprecation notice
/// * `/// Documentation`/`#[doc = "Documentation"]`: Set as GraphQL
///   description text.
#[proc_macro_derive(WundergraphEntity, attributes(wundergraph, table_name, primary_key))]
pub fn derive_wundergraph_entity(input: TokenStream) -> TokenStream {
    expand_derive(input, wundergraph_entity::derive)
}

/// A custom derive to add support for a custom enum type
///
/// # Type attributes
/// * **Required**:
///     * `#[sql_type = "DieselSqlType"]`: The sql type the enum maps on diesel side
///
/// # Variant attributes
/// All field attributes are optional if no attributes are given the variant
/// name is used as GraphQL name
///
/// * `#[graphql(name = "CustomVariantName")]`: Set the name of a enum variant
///   to the given custom name.
#[proc_macro_derive(WundergraphValue, attributes(sql_type, graphql))]
pub fn derive_wundergraph_value(input: TokenStream) -> TokenStream {
    expand_derive(input, wundergraph_value::derive)
}

/// A custom derive to implement the `BuildFilterHelper` trait
///
/// # Type attributes
/// * **Required**:
///     * `#[table_name = "diesel_table_mod"]`: Name of the underlying diesel table.
#[proc_macro_derive(BuildFilterHelper, attributes(table_name))]
pub fn derive_build_filter_helper(input: TokenStream) -> TokenStream {
    expand_derive(input, build_filter_helper::derive)
}

/// A custom derive to implement the `WundergraphBelongsTo` trait
/// for all `HasOne` fields of a given entity
///
/// # Type attributes
/// * **Required**:
///     * `#[table_name = "diesel_table_mod"]`: Name of the underlying diesel table.
///
/// # Field attributes
/// All attributes are optional. If no attributes are given the foreign key field
/// name needs to match the name of the field in the corresponding diesel `table!`
///
/// * `#[column_name = "other_name"]`: Use the given name instead of the field
#[proc_macro_derive(WundergraphBelongsTo, attributes(table_name))]
pub fn derive_belonging_to(input: TokenStream) -> TokenStream {
    expand_derive(input, belonging_to::derive)
}

#[doc(hidden)]
#[proc_macro_derive(WundergraphFilter, attributes(wundergraph, table_name))]
pub fn derive_wundergraph_filter(input: TokenStream) -> TokenStream {
    expand_derive(input, wundergraph_filter::derive)
}

fn expand_derive(
    input: TokenStream,
    f: fn(&syn::DeriveInput) -> Result<proc_macro2::TokenStream, Diagnostic>,
) -> TokenStream {
    let item = syn::parse(input).expect("Failed to parse item");
    match f(&item) {
        Ok(x) => x.into(),
        Err(e) => {
            e.emit();
            "".parse().expect("Failed to parse item")
        }
    }
}
