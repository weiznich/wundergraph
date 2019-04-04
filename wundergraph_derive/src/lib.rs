#![recursion_limit = "1024"]
#![deny(missing_debug_implementations, missing_copy_implementations)]
#![warn(
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
use proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

mod diagnostic_shim;
mod field;
mod meta;
mod model;
mod resolved_at_shim;
mod utils;

mod wundergraph_entity;
mod wundergraph_filter;
pub(crate) mod wundergraph_value;

use self::diagnostic_shim::Diagnostic;
use proc_macro::TokenStream;

#[proc_macro_derive(WundergraphEntity, attributes(wundergraph, table_name, primary_key))]
pub fn derive_wundergraph_entity(input: TokenStream) -> TokenStream {
    expand_derive(input, wundergraph_entity::derive)
}

#[proc_macro_derive(WundergraphValue, attributes(sql_type, graphql))]
pub fn derive_wundergraph_value(input: TokenStream) -> TokenStream {
    expand_derive(input, wundergraph_value::derive)
}

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
