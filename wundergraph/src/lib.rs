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
pub mod query_modifier;
#[macro_use]
mod macros;

use diesel::{Connection, QueryDsl};
use diesel::query_dsl::methods::BoxedDsl;
use diesel::backend::Backend;
use diesel::associations::HasTable;
use diesel::query_builder::BoxedSelectStatement;
use diesel::r2d2;
use failure::Error;
use self::query_modifier::{BuildQueryModifier, QueryModifier};

use juniper::LookAheadSelection;

pub trait WundergraphContext<DB>
where
    DB: Backend,
{
    type Connection: Connection<Backend = DB> + 'static;
    fn get_connection(&self) -> &Self::Connection;
}

impl<Conn> WundergraphContext<Conn::Backend>
    for r2d2::PooledConnection<r2d2::ConnectionManager<Conn>>
where
    Conn: Connection<TransactionManager = ::diesel::connection::AnsiTransactionManager> + 'static,
    Conn::Backend: ::diesel::backend::UsesAnsiSavepointSyntax,
{
    type Connection = Self;

    fn get_connection(&self) -> &r2d2::PooledConnection<r2d2::ConnectionManager<Conn>> {
        self
    }
}

pub trait LoadingHandler<DB>: Sized + HasTable
where
    DB: Backend,
{
    type SqlType;
    type QueryModifier: BuildQueryModifier<Self, Context = Self::Context>
        + QueryModifier<DB, Entity = Self>;
    type Context: WundergraphContext<DB>;
    type Query: QueryDsl
        + BoxedDsl<
        'static,
        DB,
        Output = BoxedSelectStatement<'static, Self::SqlType, Self::Table, DB>,
    >;

    fn load_item<'a>(
        select: &LookAheadSelection,
        ctx: &Self::Context,
        source: BoxedSelectStatement<'a, Self::SqlType, Self::Table, DB>,
    ) -> Result<Vec<Self>, Error>;

    fn default_query() -> Self::Query;
}

#[macro_export]
#[doc(hidden)]
/// Used by `wundergraph_derives`, which can't access `$crate`
macro_rules! __wundergraph_use_everything {
    () => {
        pub use $crate::*;
    };
}
