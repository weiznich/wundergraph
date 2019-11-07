#![deny(missing_debug_implementations, missing_copy_implementations)]
#![warn(
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
    clippy::option_map_unwrap_or,
    clippy::option_map_unwrap_or_else,
    clippy::redundant_clone,
    clippy::result_map_unwrap_or_else,
    clippy::unnecessary_unwrap,
    clippy::unseparated_literal_suffix,
    clippy::wildcard_dependencies
)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate juniper;
#[macro_use]
extern crate wundergraph;

use diesel::r2d2::{ConnectionManager, PooledConnection};

use wundergraph::scalar::WundergraphScalarValue;

pub mod api;

pub type Schema<Connection> = juniper::RootNode<
    'static,
    self::api::Query<PooledConnection<ConnectionManager<Connection>>>,
    self::api::Mutation<PooledConnection<ConnectionManager<Connection>>>,
    WundergraphScalarValue,
>;
