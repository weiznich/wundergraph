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

#[doc(hidden)]
#[macro_use]
pub extern crate diesel;
#[macro_use]
#[doc(hidden)]
pub extern crate juniper;
#[doc(hidden)]
pub extern crate indexmap;
#[macro_use]
pub extern crate failure;
#[doc(hidden)]
#[macro_use]
pub extern crate log;
extern crate serde;
#[allow(unused_imports)]
#[macro_use]
extern crate wundergraph_derive;
#[doc(hidden)]
pub use wundergraph_derive::*;

pub mod diesel_ext;
pub mod error;
pub mod filter;
pub mod helper;
pub mod mutations;
pub mod order;
pub mod query_helper;
pub mod query_modifier;
pub mod scalar;
#[macro_use]
mod macros;

use self::error::WundergraphError;
use self::helper::primary_keys::{PrimaryKeyArgument, UnRef};
use self::helper::FromLookAheadValue;
use self::query_modifier::{BuildQueryModifier, QueryModifier};
use self::scalar::WundergraphScalarValue;

use diesel::associations::HasTable;
use diesel::backend::Backend;
use diesel::expression::NonAggregate;
use diesel::query_builder::{BoxedSelectStatement, QueryFragment};
use diesel::query_dsl::methods::BoxedDsl;
use diesel::r2d2;
use diesel::{AppearsOnTable, Connection, EqAll, Identifiable, QueryDsl, Table};
use failure::Error;

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

    fn get_connection(&self) -> &Self {
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

    fn load_items<'a>(
        select: &LookAheadSelection<WundergraphScalarValue>,
        ctx: &Self::Context,
        source: BoxedSelectStatement<'a, Self::SqlType, Self::Table, DB>,
    ) -> Result<Vec<Self>, Error>;

    fn load_item<'a>(
        select: &LookAheadSelection<WundergraphScalarValue>,
        ctx: &Self::Context,
        source: BoxedSelectStatement<'a, Self::SqlType, Self::Table, DB>,
    ) -> Result<Option<Self>, Error>
    where
        Self: 'static,
        &'static Self: Identifiable,
        <&'static Self as Identifiable>::Id: UnRef<'static>,
        <Self::Table as Table>::PrimaryKey:
            EqAll<<<&'static Self as Identifiable>::Id as UnRef<'static>>::UnRefed>,
        <<Self::Table as Table>::PrimaryKey as EqAll<
            <<&'static Self as Identifiable>::Id as UnRef<'static>>::UnRefed,
        >>::Output: AppearsOnTable<Self::Table> + NonAggregate + QueryFragment<DB>,
        PrimaryKeyArgument<
            'static,
            Self::Table,
            Self::Context,
            <&'static Self as Identifiable>::Id,
        >: FromLookAheadValue,
    {
        use juniper::LookAheadMethods;
        let v = select
            .argument("primaryKey")
            .ok_or(WundergraphError::NoPrimaryKeyArgumentFound)?;
        let key = PrimaryKeyArgument::<
            Self::Table,
            Self::Context,
            <&'static Self as Identifiable>::Id,
        >::from_look_ahead(v.value()).ok_or(WundergraphError::NoPrimaryKeyArgumentFound)?;
        let query = source
            .filter(Self::table().primary_key().eq_all(key.values))
            .limit(1);
        Self::load_items(select, ctx, query).map(|i| i.into_iter().next())
    }

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
