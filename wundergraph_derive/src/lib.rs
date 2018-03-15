#![recursion_limit = "1024"]
#![cfg_attr(feature = "nightly", feature(proc_macro))]
#![deny(warnings, missing_debug_implementations, missing_copy_implementations)]
// Clippy lints
#![cfg_attr(feature = "clippy", allow(unstable_features))]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy(conf_file = "../../clippy.toml")))]
#![cfg_attr(feature = "clippy",
            allow(option_map_unwrap_or_else, option_map_unwrap_or, match_same_arms,
                  type_complexity))]
#![cfg_attr(feature = "clippy",
            warn(option_unwrap_used, result_unwrap_used, wrong_pub_self_convention, mut_mut,
                 non_ascii_literal, similar_names, unicode_not_nfc, enum_glob_use, if_not_else,
                 items_after_statements, used_underscore_binding))]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

mod utils;
mod diagnostic_shim;
mod field;
mod meta;
mod model;

mod nameable;
mod filter_value;
mod inner_filter;
mod build_filter;
mod wundergraph_entity;
mod filter;
mod from_lookahead;

use proc_macro::TokenStream;
use self::diagnostic_shim::Diagnostic;

#[proc_macro_derive(Nameable)]
pub fn derive_nameable(input: TokenStream) -> TokenStream {
    expand_derive(input, nameable::derive)
}

#[proc_macro_derive(FilterValue)]
pub fn derive_filter_value(input: TokenStream) -> TokenStream {
    expand_derive(input, filter_value::derive)
}

#[proc_macro_derive(InnerFilter)]
pub fn derive_inner_filter(input: TokenStream) -> TokenStream {
    expand_derive(input, inner_filter::derive)
}

#[proc_macro_derive(BuildFilter, attributes(wundergraph))]
pub fn derive_build_filter(input: TokenStream) -> TokenStream {
    expand_derive(input, build_filter::derive)
}

#[proc_macro_derive(WundergraphEntity, attributes(wundergraph, table_name))]
pub fn derive_wundergraph_entity(input: TokenStream) -> TokenStream {
    expand_derive(input, wundergraph_entity::derive)
}

#[proc_macro_derive(WundergraphFilter, attributes(wundergraph, table_name))]
pub fn derive_wundergraph_filter(input: TokenStream) -> TokenStream {
    expand_derive(input, filter::derive)
}

#[proc_macro_derive(FromLookAhead)]
pub fn derive_from_lookahead(input: TokenStream) -> TokenStream {
    expand_derive(input, from_lookahead::derive)
}

fn expand_derive(
    input: TokenStream,
    f: fn(&syn::DeriveInput) -> Result<quote::Tokens, Diagnostic>,
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
