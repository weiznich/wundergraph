#![feature(trace_macros)]
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
pub mod query_modifier;
pub mod scalar;
#[macro_use]
mod macros;
mod graphql_type;

use self::error::WundergraphError;
use self::helper::primary_keys::{PrimaryKeyArgument, UnRef};
use self::helper::FromLookAheadValue;
use self::query_modifier::{BuildQueryModifier, QueryModifier};
use self::scalar::WundergraphScalarValue;

use diesel::associations::HasTable;
use diesel::backend::Backend;
use diesel::dsl::SqlTypeOf;
use diesel::expression::NonAggregate;
use diesel::query_builder::{BoxedSelectStatement, QueryFragment};
use diesel::query_dsl::methods::BoxedDsl;
use diesel::query_dsl::methods::{LimitDsl, OffsetDsl, SelectDsl};
use diesel::r2d2;
use diesel::QuerySource;
use diesel::{AppearsOnTable, Connection, EqAll, Identifiable, QueryDsl, Table};
use failure::Error;
use query_helper::placeholder::*;
use std::collections::HashMap;

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
use diesel::Queryable;
use diesel_ext::BoxableFilter;
use filter::build_filter::BuildFilter;
use filter::inner_filter::InnerFilter;
use filter::Filter;
use juniper::LookAheadValue;
use query_helper::order::BuildOrder;
use query_helper::select::BuildSelect;
use query_helper::tuple::*;

pub trait LoadingHandler2<DB>: HasTable + Sized
where
    DB: Backend + 'static,
    Self::Table: 'static,
    <Self::Table as QuerySource>::FromClause: QueryFragment<DB>,
{
    type Columns: BuildOrder<Self::Table, DB>
        + BuildSelect<
            Self::Table,
            DB,
            SqlTypeOfPlaceholder<Self::FieldList, DB, Self::PrimaryKeyIndex, Self::Table>,
        >;
    type FieldList: WundergraphFieldList<
            DB,
            Self::PrimaryKeyIndex,
            Self::Table,
            PlaceHolder = Self::PlaceHolder,
        > + TupleIndex<Self::PrimaryKeyIndex>;
    type PrimaryKeyIndex: Default + Into<usize>;
    type PlaceHolder: TupleIndex<Self::PrimaryKeyIndex>
        + Queryable<
            SqlTypeOfPlaceholder<Self::FieldList, DB, Self::PrimaryKeyIndex, Self::Table>,
            DB,
        > + 'static;
    type Filter: InnerFilter + BuildFilter<DB> + 'static;

    const FIELD_NAMES: &'static [&'static str];

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

        let placeholder = <_ as RunQueryDsl<_>>::load(query, conn)?;
        Ok(Self::FieldList::resolve(
            placeholder,
            select,
            Self::FIELD_NAMES,
            conn,
        )?)
    }

    // fn raw_single_load<'a>(
    //     select: &LookAheadSelection<WundergraphScalarValue>,
    //     conn: &impl Connection<Backend = DB>,
    //     mut query: BoxedSelectStatement<
    //         'a,
    //         SqlTypeOfPlaceholder<Self::PlaceHolder, DB, Self::PrimaryKey>,
    //         Self::Table,
    //         DB,
    //     >,
    // ) -> Result<<Self::PlaceHolder as WundergraphFieldList>::PlaceHolder, Error>
    // where
    //     DB: 'a,
    //     Self::Table: 'a,
    //     SqlTypeOfPlaceholder<Self::PlaceHolder, DB, Self::PrimaryKey>: 'a,
    //     <Self::Table as QuerySource>::FromClause: QueryFragment<DB>,
    //     DB: HasSqlType<SqlTypeOfPlaceholder<Self::PlaceHolder, DB, Self::PrimaryKey>>,
    //     <Self::PlaceHolder as WundergraphFieldList>::PlaceHolder:
    //         Queryable<SqlTypeOfPlaceholder<Self::PlaceHolder, DB, Self::PrimaryKey>, DB>,
    // {
    //     use diesel::RunQueryDsl;
    //     query = Self::apply_select(query, select)?;
    //     query = Self::apply_filter(query, select)?;
    //     query = Self::apply_limit(query, select)?;
    //     query = Self::apply_offset(query, select)?;
    //     query = Self::apply_order(query, select)?;

    //     Ok(<_ as RunQueryDsl<_>>::first(query, conn)?)
    // }

    // fn raw_load<'a>(
    //     conn: &impl Connection<Backend = DB>,
    //     query: BoxedSelectStatement<
    //         'a,
    //         SqlTypeOfPlaceholder<Self::FieldList, DB, Self::PrimaryKeyIndex, Self::Table>,
    //         Self::Table,
    //         DB,
    //     >,
    // ) -> Result<Vec<Self::PlaceHolder>, Error>
    // where
    //     DB: HasSqlType<
    //         SqlTypeOfPlaceholder<Self::FieldList, DB, Self::PrimaryKeyIndex, Self::Table>,
    //     >,
    // {
    //     use diesel::RunQueryDsl;

    //     Ok(?)
    // }

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
        <Self::Columns as BuildSelect<Self::Table, DB, _>>::build_select(
            select,
            Self::FIELD_NAMES,
            <Self::FieldList as WundergraphFieldList<DB, Self::PrimaryKeyIndex, Self::Table>>::SQL_NAME_INDICES,
            <Self::FieldList as WundergraphFieldList<DB, Self::PrimaryKeyIndex, Self::Table>>::NON_SQL_NAME_INDICES,
            Self::PrimaryKeyIndex::default().into()
        )
    }

    // fn apply_select<'a>(
    //     query: BoxedSelectStatement<
    //         'a,
    //         SqlTypeOfPlaceholder<Self::PlaceHolder, DB, Self::PrimaryKey>,
    //         Self::Table,
    //         DB,
    //     >,
    //     select: &LookAheadSelection<WundergraphScalarValue>,
    // ) -> Result<
    //     BoxedSelectStatement<'a, SqlTypeOfPlaceholder<Self::PlaceHolder, DB, Self::PrimaryKey>, Self::Table, DB>,
    //     Error,
    // >
    // where
    //     Self::Table: 'a,
    //     SqlTypeOfPlaceholder<Self::PlaceHolder, DB, Self::PrimaryKey>: 'a,
    //     DB: 'a,
    //     Self::Columns: BuildSelect<Self::Table, DB, SqlTypeOfPlaceholder<Self::PlaceHolder, DB, Self::PrimaryKey>>,
    // {
    //     Ok(<_ as SelectDsl<_>>::select(
    //         query,
    //         Self::get_select(select)?,
    //     ))
    // }

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
            let order_stmts =
                <Self::Columns as BuildOrder<Self::Table, DB>>::build_order(
                    order,
                    Self::FIELD_NAMES,
                    <Self::FieldList as WundergraphFieldList<
                        DB,
                        Self::PrimaryKeyIndex,
                        Self::Table,
                    >>::SQL_NAME_INDICES,
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

table! {
    foo {
        id -> Integer,
        name -> Text,
        hair_color -> Nullable<Text>,
    }
}

table! {
    bar {
        id -> Integer,
        foo_id -> Integer,
    }
}

#[derive(Clone, Copy, Debug, Identifiable, Associations)]
#[table_name = "bar"]
#[belongs_to(Foo)]
struct Bar {
    id: i32,
    foo_id: i32,
}

impl LoadingHandler2<Pg> for Bar {
    type Columns = <bar::table as Table>::AllColumns;
    type FieldList = (i32, HasOne<i32, Foo>);
    type PrimaryKeyIndex = TupleIndex0;
    type Filter = ();

    type PlaceHolder = <Self::FieldList as WundergraphFieldList<
        Pg,
        Self::PrimaryKeyIndex,
        Self::Table,
    >>::PlaceHolder;

    const FIELD_NAMES: &'static [&'static str] = &[bar::id::NAME, bar::foo_id::NAME];
}

impl WundergraphBelongsTo<foo::table, i32, Pg> for Bar {
    type ForeignKeyColumn = bar::foo_id;

    fn resolve(
        selection: &juniper::LookAheadSelection<WundergraphScalarValue>,
        keys: &[i32],
        conn: &impl Connection<Backend = Pg>,
    ) -> Result<HashMap<i32, Vec<juniper::Value<WundergraphScalarValue>>>, Error> {
        use diesel::{ExpressionMethods, RunQueryDsl};

        let query = <_ as QueryDsl>::filter(
            <_ as QueryDsl>::select(
                Self::build_query(selection)?,
                (
                    Self::ForeignKeyColumn::default(),
                    Self::get_select(selection)?,
                ),
            ),
            Self::ForeignKeyColumn::default().eq_any(keys),
        );

        Self::build_response(query.load(conn)?, selection, conn)
    }
}

#[derive(Identifiable, Debug)]
#[table_name = "foo"]
struct Foo {
    id: i32,
    name: String,
    hair_color: Option<String>,
}

use diesel::pg::Pg;
use diesel::Column;
use query_helper::{HasMany, HasOne};

impl LoadingHandler2<Pg> for Foo {
    type Columns = <foo::table as Table>::AllColumns;
    type FieldList = (i32, String, Option<String>, HasMany<Bar>);
    type PrimaryKeyIndex = TupleIndex0;
    type Filter = ();

    type PlaceHolder = <Self::FieldList as WundergraphFieldList<
        Pg,
        Self::PrimaryKeyIndex,
        Self::Table,
    >>::PlaceHolder;

    const FIELD_NAMES: &'static [&'static str] = &[
        foo::id::NAME,
        foo::name::NAME,
        foo::hair_color::NAME,
        "bars",
    ];
}

#[allow(dead_code)]
fn test_foo(conn: &diesel::PgConnection, select: &LookAheadSelection<WundergraphScalarValue>) {
    let _r = Foo::load(select, conn, Foo::build_query(select).unwrap()).unwrap();
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
        _ctx: &Self::Context,
        _source: BoxedSelectStatement<'a, Self::SqlType, Self::Table, DB>,
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
        let _key = PrimaryKeyArgument::<
            Self::Table,
            Self::Context,
            <&'static Self as Identifiable>::Id,
        >::from_look_ahead(v.value())
        .ok_or(WundergraphError::NoPrimaryKeyArgumentFound)?;
        unimplemented!()
        // let query = source
        //     .filter(Self::table().primary_key().eq_all(key.values))
        //     .limit(1);
        // Self::load_items(select, ctx, query).map(|i| i.into_iter().next())
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
