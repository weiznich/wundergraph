#![deny(missing_debug_implementations, missing_copy_implementations)]
#![cfg_attr(feature = "cargo-clippy", allow(renamed_and_removed_lints))]
#![cfg_attr(feature = "cargo-clippy", warn(clippy))]
// Clippy lints
#![cfg_attr(feature = "cargo-clippy", allow(type_complexity))]
#![cfg_attr(
    feature = "cargo-clippy",
    warn(
        wrong_pub_self_convention,
        used_underscore_binding,
        use_self,
        use_debug,
        unseparated_literal_suffix,
        unnecessary_unwrap,
        unimplemented,
        single_match_else,
        shadow_unrelated,
        option_map_unwrap_or_else,
        option_map_unwrap_or,
        needless_continue,
        mutex_integer,
        needless_borrow,
        items_after_statements,
        filter_map,
        expl_impl_clone_on_copy,
        else_if_without_else,
        doc_markdown,
        default_trait_access,
        option_unwrap_used,
        result_unwrap_used,
        print_stdout,
        wrong_pub_self_convention,
        mut_mut,
        non_ascii_literal,
        similar_names,
        unicode_not_nfc,
        enum_glob_use,
        if_not_else,
        items_after_statements,
        used_underscore_binding
    )
)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate juniper;
#[macro_use]
extern crate wundergraph;
extern crate chrono;
extern crate failure;
extern crate serde;
extern crate serde_json;

use diesel::r2d2::{ConnectionManager, PooledConnection};

use wundergraph::scalar::WundergraphScalarValue;

pub mod api;

pub type Schema<Connection> = juniper::RootNode<
    'static,
    self::api::Query<PooledConnection<ConnectionManager<Connection>>>,
    self::api::Mutation<PooledConnection<ConnectionManager<Connection>>>,
    WundergraphScalarValue,
>;
