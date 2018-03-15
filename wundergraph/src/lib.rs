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

use diesel::{Connection, Table};
use diesel::associations::HasTable;
use diesel::query_builder::{AsQuery, BoxedSelectStatement};

use juniper::LookAheadSelection;

pub trait LoadingHandler<C>: Sized
where
    C: Connection + 'static,
{
    type Table: Table + AsQuery;
    type SqlType;

    fn load_item<'a>(
        select: &LookAheadSelection,
        conn: &C,
        source: BoxedSelectStatement<'a, Self::SqlType, Self::Table, C::Backend>,
    ) -> Result<Vec<Self>, self::error::Error>;

    fn table() -> Self::Table
    where
        Self::Table: HasTable<Table = Self::Table> + Table,
    {
        Self::Table::table()
    }
}

#[macro_export]
#[doc(hidden)]
/// Used by `diesel_derives`, which can't access `$crate`
macro_rules! __wundergraph_use_everything {
    () => {
        pub use $crate::*;
    };
}
