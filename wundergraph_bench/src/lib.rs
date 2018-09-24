//#![deny(warnings, missing_debug_implementations, missing_copy_implementations)]
// Clippy lints
#![cfg_attr(feature = "clippy", allow(unstable_features))]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(
    feature = "clippy",
    plugin(clippy(conf_file = "../../clippy.toml"))
)]
#![cfg_attr(
    feature = "clippy",
    allow(
        option_map_unwrap_or_else,
        option_map_unwrap_or,
        match_same_arms,
        type_complexity,
        useless_attribute
    )
)]
#![cfg_attr(
    feature = "clippy",
    warn(
        option_unwrap_used,
        result_unwrap_used,
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
extern crate failure;
extern crate serde;
extern crate chrono;
extern crate serde_json;

use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};

use wundergraph::scalar::WundergraphScalarValue;

pub mod api;

pub type Schema = juniper::RootNode<
    'static,
    self::api::Query<Pool<ConnectionManager<PgConnection>>>,
    self::api::Mutation<Pool<ConnectionManager<PgConnection>>>,
    WundergraphScalarValue,
>;
