//! This module contains all functionality that is needed to implement a
//! wundergraph entity
//!
//! In general wundergraph entities should be implemented using the provided
//! custom derive. Only for cases where the default implementation does not
//! your requirements a manual implementation should be done.
//!
//! # Deriving
//! See the documentation of `WundergraphEntity` for possible options.
//!
//! # Manual implementation
//! **Double check that a manual implementation is really required and could not
//! be replaced by using the provided derive + `QueryModifier`**
//!
//! For a manual implementation of a wundergraph entity it is required to
//! implement at least the following traits for your type:
//!
//! * [`LoadingHandler`](trait.LoadingHandler.html)
//! * [`WundergraphGraphqlMapper`](../..//graphql_type/trait.WundergraphGraphqlMapper.html)
//! * For each foreign key field [`WundergraphBelongsTo`](fields/trait.WundergraphBelongsTo.html)
//! * [`BuildFilterHelper`](filter/trait.BuildFilterHelper.html) if you use your
//!   entity somewhere as part of a filter
//!
//! See the documentation of the corresponding traits on details about the
//! actual implementation
use crate::context::WundergraphContext;
use crate::error::{Result, WundergraphError};
use crate::helper::tuple::IsPrimaryKeyIndex;
use crate::helper::{PrimaryKeyArgument, UnRef};
use crate::juniper_ext::FromLookAheadValue;
use crate::query_builder::selection::order::BuildOrder;
use crate::query_builder::selection::select::BuildSelect;
use crate::query_builder::types::AsInputType;
use crate::scalar::WundergraphScalarValue;
use diesel::associations::HasTable;
use diesel::backend::Backend;
use diesel::dsl::SqlTypeOf;
use diesel::expression::NonAggregate;
use diesel::query_builder::{BoxedSelectStatement, QueryFragment};
use diesel::query_dsl::methods::BoxedDsl;
use diesel::query_dsl::methods::FilterDsl;
use diesel::query_dsl::methods::{LimitDsl, SelectDsl};
use diesel::sql_types::HasSqlType;
use diesel::BoxableExpression;
use diesel::EqAll;
use diesel::Identifiable;
use diesel::QuerySource;
use diesel::{AppearsOnTable, Connection, QueryDsl, Table};
use juniper::LookAheadValue;
use juniper::{Executor, LookAheadArgument, LookAheadSelection, Selection};

pub mod fields;
pub mod filter;
#[doc(hidden)]
pub mod offset;
#[doc(hidden)]
pub mod order;
pub(crate) mod query_modifier;
#[doc(hidden)]
pub mod query_resolver;
#[doc(hidden)]
pub mod select;

use self::fields::WundergraphFieldList;
use self::filter::build_filter::BuildFilter;
use self::filter::inner_filter::InnerFilter;
use self::filter::Filter;
use self::offset::ApplyOffset;

#[doc(inline)]
pub use self::query_resolver::SqlTypeOfPlaceholder;

#[doc(inline)]
pub use self::query_modifier::QueryModifier;

#[doc(inline)]
pub use wundergraph_derive::WundergraphEntity;

/// A helper type to simplify the select statement query type
/// for a given loading handler.
///
/// # Type parameters
/// * `L`: A type implementing `LoadingHandler`
/// * `DB`: A diesel backend type (`diesel::pg::Pg` or `diesel::sqlite::Sqlite`)
/// * `Ctx`: Used context type, should implement `WundergraphContext`
pub type BoxedQuery<'a, L, DB, Ctx> = BoxedSelectStatement<
    'a,
    SqlTypeOfPlaceholder<
        <L as LoadingHandler<DB, Ctx>>::FieldList,
        DB,
        <L as LoadingHandler<DB, Ctx>>::PrimaryKeyIndex,
        <L as HasTable>::Table,
        Ctx,
    >,
    <L as HasTable>::Table,
    DB,
>;

/// Main entry point for loading database entities as GraphQL objects
///
///
/// # Deriving
///
/// This trait could be derived by using the
/// [`#[derive(WundergraphEntity)]`](derive.WundergraphEntity.html)
/// custom derive
///
/// # Manual implementation
///
/// ## Generic paramaters
/// * `DB` Diesel backend type, should be a concrete type
/// * `Ctx` Wundergraph context type. It's possible to use
///   a generic type parameter here
///
/// ```
/// # #[macro_use] extern crate diesel;
/// # #[cfg(feature = "postgres")]
/// # use diesel::pg::Pg;
/// # use diesel::Connection;
/// use wundergraph::helper::TupleIndex0;
/// use wundergraph::query_builder::selection::LoadingHandler;
/// use wundergraph::WundergraphContext;
///
/// table! {
///     heros {
///         id -> Integer,
///         name -> Text,
///     }
/// }
///
/// #[derive(Identifiable)]
/// struct Hero {
///     id: i32,
///     name: String,
/// }
///
/// # #[cfg(feature = "postgres")]
/// impl<Ctx> LoadingHandler<Pg, Ctx> for Hero
/// where
///     Ctx: WundergraphContext,
///     <Ctx as WundergraphContext>::Connection: Connection<Backend = Pg>,
/// {
///     type Columns = (heros::id, heros::name);
///     type FieldList = (i32, String);
///     type PrimaryKeyIndex = TupleIndex0;
///     type Filter = ();
///
///     const FIELD_NAMES: &'static [&'static str] = &["id", "name"];
///     const TYPE_NAME: &'static str = "Hero";
/// }
/// # fn main() {}
/// ```
pub trait LoadingHandler<DB, Ctx>: HasTable + Sized
where
    DB: Backend + ApplyOffset + 'static,
{
    /// List of columns to load
    ///
    /// This list is given as tuple of diesel column types.
    type Columns: BuildOrder<Self::Table, DB>
        + BuildSelect<
            Self::Table,
            DB,
            SqlTypeOfPlaceholder<Self::FieldList, DB, Self::PrimaryKeyIndex, Self::Table, Ctx>,
        >;

    /// List of all field types
    ///
    /// This list is given as tuple of rust types compatible to the sql type
    /// of the corresponding column. The order of those types is assumed to match
    /// the field order provided to `Columns`
    type FieldList: WundergraphFieldList<DB, Self::PrimaryKeyIndex, Self::Table, Ctx>;

    /// Index of the primary key as index into the `Columns` and `FieldList` tuples
    type PrimaryKeyIndex: Default + IsPrimaryKeyIndex;

    /// The filter type for this entity
    type Filter: InnerFilter + BuildFilter<DB> + 'static;

    /// List of graphql field names
    ///
    /// Those names are assumed to have the same order then the fields in `FieldList`
    const FIELD_NAMES: &'static [&'static str];
    /// The graphql name of the current type
    const TYPE_NAME: &'static str;
    /// The graphql description of the current type
    const TYPE_DESCRIPTION: Option<&'static str> = None;

    /// Main entry point to loading something from the database
    ///
    /// The default implementation passes the final query to the
    /// `QueryModifier`
    fn load<'a>(
        select: &LookAheadSelection<'_, WundergraphScalarValue>,
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
        query: BoxedQuery<'a, Self, DB, Ctx>,
    ) -> Result<Vec<juniper::Value<WundergraphScalarValue>>>
    where
        DB: HasSqlType<
            SqlTypeOfPlaceholder<Self::FieldList, DB, Self::PrimaryKeyIndex, Self::Table, Ctx>,
        >,
        Ctx: WundergraphContext + QueryModifier<Self, DB>,
        Ctx::Connection: Connection<Backend = DB>,
        DB::QueryBuilder: Default,
        <Self::Table as QuerySource>::FromClause: QueryFragment<DB>,
    {
        use diesel::RunQueryDsl;
        use juniper::LookAheadMethods;

        let ctx = executor.context();
        let conn = ctx.get_connection();
        let query = ctx.modify_query(select, query)?;
        #[cfg(feature = "debug")]
        {
            log::debug!("{:?}", diesel::debug_query(&query));
        }
        let placeholder = <_ as RunQueryDsl<_>>::load(query, conn)?;
        Ok(Self::FieldList::resolve(
            placeholder,
            select.arguments(),
            select,
            selection,
            Self::FIELD_NAMES,
            executor,
        )?)
    }

    /// Load a single entity by a given primary key
    ///
    /// The default implementation calls `load` internally
    fn load_by_primary_key<'a>(
        select: &LookAheadSelection<'_, WundergraphScalarValue>,
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
        mut query: BoxedQuery<'a, Self, DB, Ctx>,
    ) -> Result<Option<juniper::Value<WundergraphScalarValue>>>
    where
        Self: 'static,
        &'static Self: Identifiable,
        Ctx: WundergraphContext + QueryModifier<Self, DB>,
        Ctx::Connection: Connection<Backend = DB>,
        <&'static Self as Identifiable>::Id: UnRef<'static>,
        <<&'static Self as Identifiable>::Id as UnRef<'static>>::UnRefed: AsInputType,
        <Self::Table as Table>::PrimaryKey:
            EqAll<<<<&'static Self as Identifiable>::Id as UnRef<'static>>::UnRefed as AsInputType>::InputType> + Default,
        <<Self::Table as Table>::PrimaryKey as EqAll<
            <<<&'static Self as Identifiable>::Id as UnRef<'static>>::UnRefed as AsInputType>::InputType,
        >>::Output: AppearsOnTable<Self::Table> + NonAggregate + QueryFragment<DB>,
        PrimaryKeyArgument<'static, Self::Table, (), <&'static Self as Identifiable>::Id>:
            FromLookAheadValue,
        DB: HasSqlType<
            SqlTypeOfPlaceholder<Self::FieldList, DB, Self::PrimaryKeyIndex, Self::Table, Ctx>,
        >,
        DB::QueryBuilder: Default,
        <Self::Table as QuerySource>::FromClause: QueryFragment<DB>,
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
        query = <_ as QueryDsl>::filter(
            query,
            <Self::Table as Table>::PrimaryKey::default().eq_all(key.values),
        );
        query = <_ as QueryDsl>::limit(query, 1);
        let res = Self::load(select, selection, executor, query)?;
        Ok(res.into_iter().next())
    }

    /// Build a sql query to load this entity from a given graphql request
    ///
    /// The default implementation calls `get_select`, `apply_filter`,
    /// `apply_limit`, `apply_offset` and `apply_order` to construct the final
    /// query
    fn build_query<'a>(
        _global_arguments: &[LookAheadArgument<WundergraphScalarValue>],
        select: &LookAheadSelection<'_, WundergraphScalarValue>,
    ) -> Result<BoxedQuery<'a, Self, DB, Ctx>>
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
            > + 'static,
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

    /// Construct a select clause for the current entity from a given graphql request
    fn get_select(
        select: &LookAheadSelection<'_, WundergraphScalarValue>,
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
                    Ctx,
                >,
            >,
        >,
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

    /// Construct a where clause from a given graphql request
    fn apply_filter<'a>(
        query: BoxedQuery<'a, Self, DB, Ctx>,
        select: &LookAheadSelection<'_, WundergraphScalarValue>,
    ) -> Result<BoxedQuery<'a, Self, DB, Ctx>>
    where
        Self::Table: 'static,
        <Self::Filter as BuildFilter<DB>>::Ret: AppearsOnTable<Self::Table>,
    {
        use juniper::LookAheadMethods;
        if let Some(filter) = select.argument("filter") {
            if let Some(filter) =
                <Filter<Self::Filter, Self::Table> as FromLookAheadValue>::from_look_ahead(
                    filter.value(),
                )
                .and_then(<_ as BuildFilter<DB>>::into_filter)
            {
                Ok(<_ as FilterDsl<_>>::filter(query, filter))
            } else {
                Ok(query)
            }
        } else {
            Ok(query)
        }
    }

    /// Construct a order clause from a given graphql request
    fn apply_order<'a>(
        mut query: BoxedQuery<'a, Self, DB, Ctx>,
        select: &LookAheadSelection<'_, WundergraphScalarValue>,
    ) -> Result<BoxedQuery<'a, Self, DB, Ctx>>
    where
        Self::Table: 'static,
    {
        use juniper::LookAheadMethods;
        match select.argument("order").map(LookAheadArgument::value) {
            Some(LookAheadValue::List(order)) => {
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
            }
            Some(_) => Err(WundergraphError::CouldNotBuildFilterArgument),
            None => Ok(query),
        }
    }

    /// Construct a limit clause from a given graphql request
    fn apply_limit<'a>(
        query: BoxedQuery<'a, Self, DB, Ctx>,
        select: &LookAheadSelection<'_, WundergraphScalarValue>,
    ) -> Result<BoxedQuery<'a, Self, DB, Ctx>> {
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

    /// Construct a offset clause from a given grahpql request
    fn apply_offset<'a>(
        query: BoxedQuery<'a, Self, DB, Ctx>,
        select: &LookAheadSelection<'_, WundergraphScalarValue>,
    ) -> Result<BoxedQuery<'a, Self, DB, Ctx>> {
        <DB as ApplyOffset>::apply_offset::<Self, Ctx>(query, select)
    }

    #[doc(hidden)]
    fn field_description(_idx: usize) -> Option<&'static str> {
        None
    }

    #[doc(hidden)]
    #[allow(clippy::option_option)]
    fn field_deprecation(_idx: usize) -> Option<Option<&'static str>> {
        None
    }
}
