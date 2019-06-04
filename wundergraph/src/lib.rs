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
#![allow(clippy::type_complexity)]

#[doc(hidden)]
#[macro_use]
pub extern crate diesel;
#[doc(hidden)]
pub extern crate indexmap;
#[doc(hidden)]
pub extern crate juniper;
#[macro_use]
pub extern crate failure;
#[doc(hidden)]
pub extern crate paste;

#[allow(unused_imports)]
#[macro_use]
extern crate wundergraph_derive;
#[doc(hidden)]
pub use wundergraph_derive::{WundergraphEntity, WundergraphFilter, WundergraphValue};

pub mod diesel_ext;
mod error;
pub mod helper;
pub mod juniper_ext;
pub mod scalar;
#[macro_use]
mod macros;
pub mod context;
pub mod graphql_type;
pub mod query_builder;

mod third_party_integrations;

#[macro_export]
#[doc(hidden)]
/// Used by `wundergraph_derives`, which can't access `$crate`
macro_rules! __use_everything {
    () => {
        pub use $crate::*;
    };
}
