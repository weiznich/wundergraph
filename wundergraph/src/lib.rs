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
#[doc(hidden)]
pub extern crate diesel;
#[macro_use]
#[doc(hidden)]
pub extern crate juniper;
#[doc(hidden)]
pub extern crate ordermap;
#[macro_use]
pub extern crate failure;
#[cfg_attr(feature = "clippy", allow(useless_attribute))]
#[allow(unused_imports)]
#[macro_use]
extern crate wundergraph_derive;
#[doc(hidden)]
pub use wundergraph_derive::*;

pub mod filter;
pub mod helper;
pub mod mutations;
pub mod order;
pub mod query_helper;
pub mod error;
#[macro_use]
mod macros;

use diesel::Connection;
use diesel::Queryable;
use diesel::backend::Backend;
use diesel::associations::HasTable;
use diesel::query_builder::BoxedSelectStatement;
use failure::Error;

use juniper::LookAheadSelection;

pub trait WundergraphEntity: Sized + HasTable {
    type SqlType;
}

pub trait LoadingHandler<DB>
    : WundergraphEntity + Queryable<<Self as WundergraphEntity>::SqlType, DB>
where
    DB: Backend,
{
    fn load_item<'a, C>(
        select: &LookAheadSelection,
        conn: &C,
        source: BoxedSelectStatement<'a, Self::SqlType, Self::Table, DB>,
    ) -> Result<Vec<Self>, Error>
    where
        C: Connection<Backend = DB> + 'static;
}

#[macro_export]
#[doc(hidden)]
/// Used by `wundergraph_derives`, which can't access `$crate`
macro_rules! __wundergraph_use_everything {
    () => {
        pub use $crate::*;
    };
}
