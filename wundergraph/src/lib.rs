//! Wundergraph provides a platform to easily expose your database through
//! a GraphQL interface.
//!
//! ## Short example
//!
//! ```rust
//! # #[macro_use] extern crate diesel;
//! #
//! use wundergraph::prelude::*;
//!
//! table! {
//!     heros {
//!         id -> Integer,
//!         name -> Text,
//!         hair_color -> Nullable<Text>,
//!         species -> Integer,
//!     }
//! }
//!
//! table! {
//!     species {
//!         id -> Integer,
//!         name -> Text,
//!     }
//! }
//!
//! #[derive(Clone, Debug, Identifiable, WundergraphEntity)]
//! #[table_name = "heros"]
//! pub struct Hero {
//!     id: i32,
//!     name: String,
//!     hair_color: Option<String>,
//!     species: HasOne<i32, Species>,
//! }
//!
//! #[derive(Clone, Debug, Identifiable, WundergraphEntity)]
//! #[table_name = "species"]
//! pub struct Species {
//!     id: i32,
//!     name: String,
//!     heros: HasMany<Hero, heros::species>,
//! }
//!
//! wundergraph::query_object!{
//!     Query {
//!        Hero,
//!        Species,
//!     }
//! }
//!
//! # fn main() {}
//! ```
//!
//! ## Where to find things
//!
//! Everything required for basic usage of wundergraph is exposed through
//! [`wundergraph::prelude`](prelude/index.html).
//! [`wundergraph::query_builder::selection`](query_builder/selection/index.html)
//! contains functionality to manual extend or implement a query entity,
//! [`wundergraph::query_builder::mutations`](query_builder/mutations/index.html)
//! contains similar functionality for mutations.
//! [`wundergraph::scalar`](scalar/index.html) provides the implementation of
//! the internal used juniper scalar value type. [`wundergraph::error`](error/index.html)
//! contains the definition of the internal error type.
//! [`wundergraph::diesel_ext`](diesel_ext/index.html) and
//! [`wundergraph::juniper_ext`](juniper_ext/index.html) provide
//! extension traits and types for the corresponding crates.
//! [`wundergraph::helper`](helper/index.html) contains wundergraph
//! specific helper types.
//!
//!

#![deny(missing_debug_implementations, missing_copy_implementations)]
#![warn(
//    missing_docs,
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
#[doc(hidden)]
#[cfg(feature = "debug")]
pub extern crate log;
#[doc(hidden)]
pub extern crate paste;

pub use wundergraph_derive::WundergraphEntity;

pub mod diesel_ext;
pub mod error;
pub mod helper;
pub mod juniper_ext;
pub mod scalar;
#[macro_use]
mod macros;
pub(crate) mod context;
#[doc(hidden)]
pub mod graphql_type;
pub mod query_builder;

mod third_party_integrations;

pub mod prelude {
    //! Re-exports important traits and types. Meant to be glob imported
    //! when using wundergraph.

    #[doc(inline)]
    pub use super::context::WundergraphContext;

    #[doc(inline)]
    pub use super::query_builder::types::{HasMany, HasOne};

    #[doc(inline)]
    pub use crate::query_builder::selection::{BoxedQuery, QueryModifier};

    #[doc(inline)]
    pub use super::WundergraphEntity;

    #[doc(inline)]
    pub use super::query_object;

    #[doc(inline)]
    pub use super::mutation_object;
}

#[doc(hidden)]
pub use self::prelude::*;

#[macro_export]
#[doc(hidden)]
/// Used by `wundergraph_derives`, which can't access `$crate`
macro_rules! __use_everything {
    () => {
        pub use $crate::*;
    };
}
