//#![feature(trace_macros, nll)]
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

//#![warn(missing_docs)]
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

#[doc(hidden)]
pub mod diesel_ext;
pub mod error;
pub mod filter;
pub mod helper;
pub mod mutations;
pub mod order;
pub mod query_helper;
//pub mod query_modifier;
pub mod scalar;
#[macro_use]
mod macros;
pub mod graphql_type;

use self::error::WundergraphError;
use self::helper::FromLookAheadValue;
//use self::query_modifier::{BuildQueryModifier, QueryModifier};
use self::scalar::WundergraphScalarValue;

use diesel::associations::HasTable;
use diesel::backend::Backend;
use diesel::dsl::SqlTypeOf;
use diesel::expression::NonAggregate;
use diesel::query_builder::{BoxedSelectStatement, QueryFragment};
use diesel::query_dsl::methods::BoxedDsl;
use diesel::query_dsl::methods::{LimitDsl, OffsetDsl, SelectDsl};
use diesel::r2d2;
use diesel::EqAll;
use diesel::Identifiable;
use diesel::QuerySource;
use diesel::{AppearsOnTable, Connection, QueryDsl, Table};
use failure::Error;
use helper::primary_keys::{PrimaryKeyArgument, UnRef};
use query_helper::placeholder::*;

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
use diesel::query_dsl::methods::FilterDsl;
use diesel::sql_types::{Bool, HasSqlType};
use diesel::BoxableExpression;
use diesel_ext::BoxableFilter;
use filter::build_filter::BuildFilter;
use filter::inner_filter::InnerFilter;
use filter::Filter;
use juniper::LookAheadValue;
use query_helper::order::BuildOrder;
use query_helper::select::BuildSelect;
use query_helper::tuple::IsPrimaryKeyIndex;

pub trait LoadingHandler<DB>: HasTable + Sized
where
    DB: Backend + 'static,
    Self::Table: 'static,
    <Self::Table as QuerySource>::FromClause: QueryFragment<DB>,
    DB::QueryBuilder: Default,
{
    type Columns: BuildOrder<Self::Table, DB>
        + BuildSelect<
            Self::Table,
            DB,
            SqlTypeOfPlaceholder<Self::FieldList, DB, Self::PrimaryKeyIndex, Self::Table>,
        >;
    type FieldList: WundergraphFieldList<DB, Self::PrimaryKeyIndex, Self::Table>;

    type PrimaryKeyIndex: Default + IsPrimaryKeyIndex;
    type Filter: InnerFilter + BuildFilter<DB> + 'static;

    const FIELD_NAMES: &'static [&'static str];
    const TYPE_NAME: &'static str;

    fn load<'a>(
        select: &LookAheadSelection<WundergraphScalarValue>,
        conn: &impl Connection<Backend = DB>,
        query: BoxedSelectStatement<
            'a,
            SqlTypeOfPlaceholder<Self::FieldList, DB, Self::PrimaryKeyIndex, Self::Table>,
            Self::Table,
            DB,
        >,
    ) -> Result<Vec<juniper::Value<WundergraphScalarValue>>, Error>
    where
        DB: HasSqlType<
            SqlTypeOfPlaceholder<Self::FieldList, DB, Self::PrimaryKeyIndex, Self::Table>,
        >,
    {
        use diesel::RunQueryDsl;
        if cfg!(feature = "debug") {
            println!("{:?}", diesel::debug_query(&query));
        }
        let placeholder = <_ as RunQueryDsl<_>>::load(query, conn)?;
        Ok(Self::FieldList::resolve(
            placeholder,
            select,
            Self::FIELD_NAMES,
            conn,
        )?)
    }

    fn load_by_primary_key<'a>(
        select: &LookAheadSelection<WundergraphScalarValue>,
        conn: &impl Connection<Backend = DB>,
        mut query: BoxedSelectStatement<
            'a,
            SqlTypeOfPlaceholder<Self::FieldList, DB, Self::PrimaryKeyIndex, Self::Table>,
            Self::Table,
            DB,
        >,
    ) -> Result<Option<juniper::Value<WundergraphScalarValue>>, Error>
    where
        Self: 'static,
        &'static Self: Identifiable,
        <&'static Self as Identifiable>::Id: UnRef<'static>,
        <Self::Table as Table>::PrimaryKey:
            EqAll<<<&'static Self as Identifiable>::Id as UnRef<'static>>::UnRefed>,
        <<Self::Table as Table>::PrimaryKey as EqAll<
            <<&'static Self as Identifiable>::Id as UnRef<'static>>::UnRefed,
        >>::Output: AppearsOnTable<Self::Table> + NonAggregate + QueryFragment<DB>,
        PrimaryKeyArgument<'static, Self::Table, (), <&'static Self as Identifiable>::Id>:
            FromLookAheadValue,
        DB: HasSqlType<
            SqlTypeOfPlaceholder<Self::FieldList, DB, Self::PrimaryKeyIndex, Self::Table>,
        >,
    {
        use juniper::LookAheadMethods;
        let v = select
            .argument("primaryKey")
            .ok_or(WundergraphError::NoPrimaryKeyArgumentFound)?;
        let key = PrimaryKeyArgument::<
            Self::Table,
            _,
            <&'static Self as Identifiable>::Id,
            >::from_look_ahead(v.value())
            .ok_or(WundergraphError::NoPrimaryKeyArgumentFound)?;
        query = <_ as QueryDsl>::filter(query, Self::table().primary_key().eq_all(key.values));
        query = <_ as QueryDsl>::limit(query, 1);
        let res = Self::load(select, conn, query)?;
        Ok(res.into_iter().next())
    }

    fn build_query<'a>(
        select: &LookAheadSelection<WundergraphScalarValue>,
    ) -> Result<
        BoxedSelectStatement<
            'a,
            SqlTypeOfPlaceholder<Self::FieldList, DB, Self::PrimaryKeyIndex, Self::Table>,
            Self::Table,
            DB,
        >,
        Error,
    >
    where
        Self::Table: BoxedDsl<
            'a,
            DB,
            Output = BoxedSelectStatement<
                'a,
                SqlTypeOf<<Self::Table as Table>::AllColumns>,
                Self::Table,
                DB,
            >,
        >,
        <Self::Filter as BuildFilter<DB>>::Ret: AppearsOnTable<Self::Table>,
    {
        let mut query =
            <_ as SelectDsl<_>>::select(Self::table().into_boxed(), Self::get_select(select)?);

        query = Self::apply_filter(query, select)?;
        query = Self::apply_limit(query, select)?;
        query = Self::apply_offset(query, select)?;
        query = Self::apply_order(query, select)?;

        Ok(query)
    }

    fn get_select(
        select: &LookAheadSelection<WundergraphScalarValue>,
    ) -> Result<
        Box<
            dyn BoxableExpression<
                Self::Table,
                DB,
                SqlType = SqlTypeOfPlaceholder<
                    Self::FieldList,
                    DB,
                    Self::PrimaryKeyIndex,
                    Self::Table,
                >,
            >,
        >,
        Error,
    > {
        use juniper::LookAheadMethods;
        <Self::Columns as BuildSelect<Self::Table, DB, _>>::build_select(
            select,
            |local_index| {
                Self::FieldList::map_table_field(local_index, |global| Self::FIELD_NAMES[global])
                    .expect("Field is there")
            },
            Self::PrimaryKeyIndex::is_index,
            (0..Self::FieldList::NON_TABLE_FIELD_COUNT).any(|i| {
                Self::FieldList::map_non_table_field(i, |global| {
                    select.has_child(Self::FIELD_NAMES[global])
                })
                .unwrap_or(false)
            }),
        )
    }

    fn get_filter(
        input: &LookAheadValue<WundergraphScalarValue>,
    ) -> Result<Option<Box<dyn BoxableFilter<Self::Table, DB, SqlType = Bool>>>, Error>
    where
        <Self::Filter as BuildFilter<DB>>::Ret: AppearsOnTable<Self::Table>,
    {
        Ok(
            <Filter<Self::Filter, Self::Table> as FromLookAheadValue>::from_look_ahead(input)
                .and_then(|f| <_ as BuildFilter<DB>>::into_filter(f)),
        )
    }

    fn apply_filter<'a>(
        query: BoxedSelectStatement<
            'a,
            SqlTypeOfPlaceholder<Self::FieldList, DB, Self::PrimaryKeyIndex, Self::Table>,
            Self::Table,
            DB,
        >,
        select: &LookAheadSelection<WundergraphScalarValue>,
    ) -> Result<
        BoxedSelectStatement<
            'a,
            SqlTypeOfPlaceholder<Self::FieldList, DB, Self::PrimaryKeyIndex, Self::Table>,
            Self::Table,
            DB,
        >,
        Error,
    >
    where
        <Self::Filter as BuildFilter<DB>>::Ret: AppearsOnTable<Self::Table>,
    {
        use juniper::LookAheadMethods;
        if let Some(filter) = select.argument("filter") {
            if let Some(filter) = Self::get_filter(filter.value())? {
                Ok(<_ as FilterDsl<_>>::filter(query, filter))
            } else {
                Ok(query)
            }
        } else {
            Ok(query)
        }
    }

    fn apply_order<'a>(
        mut query: BoxedSelectStatement<
            'a,
            SqlTypeOfPlaceholder<Self::FieldList, DB, Self::PrimaryKeyIndex, Self::Table>,
            Self::Table,
            DB,
        >,
        select: &LookAheadSelection<WundergraphScalarValue>,
    ) -> Result<
        BoxedSelectStatement<
            'a,
            SqlTypeOfPlaceholder<Self::FieldList, DB, Self::PrimaryKeyIndex, Self::Table>,
            Self::Table,
            DB,
        >,
        Error,
    > {
        use juniper::{LookAheadMethods, LookAheadValue};
        if let Some(LookAheadValue::<WundergraphScalarValue>::List(order)) =
            select.argument("order").map(|o| o.value())
        {
            let order_stmts = <Self::Columns as BuildOrder<Self::Table, DB>>::build_order(
                order,
                |local_index| {
                    Self::FieldList::map_table_field(local_index, |global| {
                        Self::FIELD_NAMES[global]
                    })
                    .expect("Field is there")
                },
            )?;
            for s in order_stmts {
                query = query.then_order_by(s);
            }
            Ok(query)
        } else {
            Ok(query)
        }
    }

    fn apply_limit<'a>(
        query: BoxedSelectStatement<
            'a,
            SqlTypeOfPlaceholder<Self::FieldList, DB, Self::PrimaryKeyIndex, Self::Table>,
            Self::Table,
            DB,
        >,
        select: &LookAheadSelection<WundergraphScalarValue>,
    ) -> Result<
        BoxedSelectStatement<
            'a,
            SqlTypeOfPlaceholder<Self::FieldList, DB, Self::PrimaryKeyIndex, Self::Table>,
            Self::Table,
            DB,
        >,
        Error,
    > {
        use juniper::LookAheadMethods;
        if let Some(limit) = select.argument("limit") {
            Ok(<_ as LimitDsl>::limit(
                query,
                i64::from_look_ahead(limit.value())
                    .ok_or(WundergraphError::CouldNotBuildFilterArgument)?,
            ))
        } else {
            Ok(query)
        }
    }

    fn apply_offset<'a>(
        query: BoxedSelectStatement<
            'a,
            SqlTypeOfPlaceholder<Self::FieldList, DB, Self::PrimaryKeyIndex, Self::Table>,
            Self::Table,
            DB,
        >,
        select: &LookAheadSelection<WundergraphScalarValue>,
    ) -> Result<
        BoxedSelectStatement<
            'a,
            SqlTypeOfPlaceholder<Self::FieldList, DB, Self::PrimaryKeyIndex, Self::Table>,
            Self::Table,
            DB,
        >,
        Error,
    > {
        use juniper::LookAheadMethods;
        if let Some(offset) = select.argument("offset") {
            Ok(<_ as OffsetDsl>::offset(
                query,
                i64::from_look_ahead(offset.value())
                    .ok_or(WundergraphError::CouldNotBuildFilterArgument)?,
            ))
        } else {
            Ok(query)
        }
    }
}

#[macro_export]
#[doc(hidden)]
/// Used by `wundergraph_derives`, which can't access `$crate`
macro_rules! __wundergraph_use_everything {
    () => {
        pub use $crate::*;
    };
}
