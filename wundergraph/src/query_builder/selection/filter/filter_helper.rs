use super::build_filter::BuildFilter;
use super::common_filter::FilterOption;
use super::filter_value::FilterValue;
use super::inner_filter::InnerFilter;
use super::nullable_filter::IsNull;
use super::reference_filter::ReferenceFilter;
use super::Filter;
use crate::diesel_ext::BoxableFilter;
use crate::helper::tuple::ConcatTuples;
use crate::juniper_ext::{FromLookAheadValue, NameBuilder, Nameable};
use crate::query_builder::selection::fields::WundergraphBelongsTo;
use crate::query_builder::selection::fields::{FieldListExtractor, NonTableFieldExtractor};
use crate::query_builder::selection::offset::ApplyOffset;
use crate::query_builder::selection::LoadingHandler;
use crate::query_builder::types::{HasMany, HasOne};
use crate::scalar::WundergraphScalarValue;
use diesel::associations::HasTable;
use diesel::backend::Backend;
use diesel::expression::{NonAggregate, SelectableExpression};
use diesel::query_builder::QueryFragment;
use diesel::sql_types::Bool;
use diesel::Expression;
use diesel::QuerySource;
use diesel::Table;
use indexmap::IndexMap;
use juniper::meta::Argument;
use juniper::{FromInputValue, GraphQLType, InputValue, LookAheadValue, Registry, ToInputValue};
use std::fmt::{self, Debug};
use std::marker::PhantomData;

pub use wundergraph_derive::BuildFilterHelper;

#[doc(hidden)]
pub struct FilterWrapper<L, DB, Ctx>
where
    FilterConverter<L, DB, Ctx>: CreateFilter,
{
    filter: <FilterConverter<L, DB, Ctx> as CreateFilter>::Filter,
}

impl<L, DB, Ctx> Clone for FilterWrapper<L, DB, Ctx>
where
    FilterConverter<L, DB, Ctx>: CreateFilter,
    <FilterConverter<L, DB, Ctx> as CreateFilter>::Filter: Clone,
{
    fn clone(&self) -> Self {
        Self {
            filter: self.filter.clone(),
        }
    }
}

impl<L, DB, Ctx> Debug for FilterWrapper<L, DB, Ctx>
where
    FilterConverter<L, DB, Ctx>: CreateFilter,
    <FilterConverter<L, DB, Ctx> as CreateFilter>::Filter: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FilterWrapper")
            .field("inner", &self.filter)
            .finish()
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct FilterConverter<L, DB, Ctx>(PhantomData<(L, DB, Ctx)>);

#[doc(hidden)]
#[derive(Debug)]
pub struct ColumnFilterConverter<N, C, DB, Ctx>(PhantomData<(N, C, DB, Ctx)>);

#[doc(hidden)]
#[derive(Debug)]
pub struct NonColumnFilterConveter<N, L, DB, Ctx>(PhantomData<(N, L, DB, Ctx)>);

/// Helper trait to build a compound filter from filter nodes
/// that exist for each field
pub trait CreateFilter {
    /// Final filter type
    type Filter;
}

impl<L, DB, Ctx> CreateFilter for FilterConverter<L, DB, Ctx>
where
    DB: Backend + ApplyOffset + 'static,
    DB::QueryBuilder: Default,
    L::Table: 'static,
    <L::Table as QuerySource>::FromClause: QueryFragment<DB>,
    L: LoadingHandler<DB, Ctx>,
    L::FieldList: FieldListExtractor + NonTableFieldExtractor,
    ColumnFilterConverter<<L::FieldList as FieldListExtractor>::Out, L::Columns, DB, Ctx>: CreateFilter,
    NonColumnFilterConveter<<L::FieldList as NonTableFieldExtractor>::Out, L, DB, Ctx>: CreateFilter,
    <ColumnFilterConverter<<L::FieldList as FieldListExtractor>::Out, L::Columns, DB, Ctx> as CreateFilter>::Filter: ConcatTuples<<
    NonColumnFilterConveter<<L::FieldList as NonTableFieldExtractor>::Out, L, DB, Ctx> as CreateFilter>::Filter>
{
    type Filter = <<ColumnFilterConverter<<L::FieldList as FieldListExtractor>::Out, L::Columns, DB, Ctx> as CreateFilter>::Filter as ConcatTuples<<
    NonColumnFilterConveter<<L::FieldList as NonTableFieldExtractor>::Out, L, DB, Ctx> as CreateFilter>::Filter>>::Out;
}

impl<DB, Ctx> CreateFilter for ColumnFilterConverter<(), (), DB, Ctx> {
    type Filter = ();
}

impl<DB, L, Ctx> CreateFilter for NonColumnFilterConveter<(), L, DB, Ctx> {
    type Filter = ();
}

/// Marker trait indicating that some value has a filter that is
/// applied to a specific  database column
pub trait AsColumnFilter<C, DB, Ctx> {
    /// Type of the filter
    type Filter;
}

/// Marker trait indicating that some value has a filter
/// that is not attached to a column
pub trait AsNonColumnFilter<L, DB, Ctx> {
    /// Type of the filter
    type Filter;
}

impl<L, O, DB, Ctx, FK> AsNonColumnFilter<L, DB, Ctx> for HasMany<O, FK>
where
    L: HasTable,
    FK: Default + NonAggregate + QueryFragment<DB> + SelectableExpression<O::Table>,
    O: WundergraphBelongsTo<L::Table, DB, Ctx, FK>,
    O::Table: 'static,
    DB: Backend + ApplyOffset + 'static,
    <O::Table as QuerySource>::FromClause: QueryFragment<DB>,
    DB::QueryBuilder: Default,
{
    type Filter =
        ReferenceFilter<<L::Table as Table>::PrimaryKey, Filter<O::Filter, O::Table>, FK, ()>;
}

impl<C, DB, Ctx> AsColumnFilter<C, DB, Ctx> for i16 {
    type Filter = FilterOption<Self, C>;
}

impl<C, DB, Ctx> AsColumnFilter<C, DB, Ctx> for i32 {
    type Filter = FilterOption<Self, C>;
}

impl<C, DB, Ctx> AsColumnFilter<C, DB, Ctx> for i64 {
    type Filter = FilterOption<Self, C>;
}

impl<C, DB, Ctx> AsColumnFilter<C, DB, Ctx> for bool {
    type Filter = FilterOption<Self, C>;
}

impl<C, DB, Ctx> AsColumnFilter<C, DB, Ctx> for f32 {
    type Filter = FilterOption<Self, C>;
}

impl<C, DB, Ctx> AsColumnFilter<C, DB, Ctx> for f64 {
    type Filter = FilterOption<Self, C>;
}

impl<C, DB, Ctx> AsColumnFilter<C, DB, Ctx> for String {
    type Filter = FilterOption<Self, C>;
}

impl<C, DB, T, Ctx> AsColumnFilter<C, DB, Ctx> for Vec<T>
where
    T: FromLookAheadValue
        + FromInputValue<WundergraphScalarValue>
        + ToInputValue<WundergraphScalarValue>
        + FilterValue<C>
        + Clone,
{
    type Filter = FilterOption<Self, C>;
}

impl<C, DB, T, Ctx> AsColumnFilter<C, DB, Ctx> for Option<T>
where
    T: FilterValue<C>
        + Clone
        + FromInputValue<WundergraphScalarValue>
        + FromLookAheadValue
        + ToInputValue<WundergraphScalarValue>,
    T: AsColumnFilter<C, DB, Ctx, Filter = FilterOption<T, C>>,
{
    type Filter = FilterOption<Self, C>;
}

impl<C, K, I, DB, Ctx> AsColumnFilter<C, DB, Ctx> for HasOne<K, I>
where
    DB: Backend + ApplyOffset + 'static,
    I::Table: 'static,
    I: LoadingHandler<DB, Ctx>,
    <I::Table as QuerySource>::FromClause: QueryFragment<DB>,
    DB::QueryBuilder: Default,
{
    type Filter =
        ReferenceFilter<C, Filter<I::Filter, I::Table>, <I::Table as Table>::PrimaryKey, ()>;
}

// That's a false positve
#[allow(clippy::use_self)]
impl<C, K, I, DB, Ctx> AsColumnFilter<C, DB, Ctx> for Option<HasOne<K, I>>
where
    DB: Backend + ApplyOffset + 'static,
    I::Table: 'static,
    I: LoadingHandler<DB, Ctx>,
    <I::Table as QuerySource>::FromClause: QueryFragment<DB>,
    DB::QueryBuilder: Default,
{
    type Filter = ReferenceFilter<
        C,
        Filter<I::Filter, I::Table>,
        <I::Table as Table>::PrimaryKey,
        Option<IsNull<C>>,
    >;
}

impl<L, DB, Ctx> Nameable for FilterWrapper<L, DB, Ctx>
where
    DB: Backend + ApplyOffset + 'static,
    L::Table: 'static,
    L: LoadingHandler<DB, Ctx>,
    <L::Table as QuerySource>::FromClause: QueryFragment<DB>,
    DB::QueryBuilder: Default,
    FilterConverter<L, DB, Ctx>: CreateFilter,
{
    fn name() -> String {
        format!("{}Filter", L::TYPE_NAME)
    }
}

/// A helper trait describing a single node of a filter argument
///
/// # Type paramaters
/// * `Self`:
/// * `DB`: Backend from diesel, so one of `Pg` or `Sqlite`
/// * `F`:
/// * `Ctx`: The used wundergraph context type
///
/// # Deriving
/// An implementation of this trait is automatically generated by
/// [`#[derive(WundergraphEntity)]`](../derive.WundergraphEntity.html)
/// or [`#[derive(BuildFilterHelper)]`](derive.BuildFilterHelper.html)
///
/// # Manual implementation
/// Manually implementing this trait should only be the last resort if none of
/// the provided derives generate compatible code. Below an expanded version of
/// the generated implementation is shown.
///
/// ```
/// # #[macro_use] extern crate diesel;
/// # use wundergraph::helper::TupleIndex0;
/// # use wundergraph::query_builder::selection::LoadingHandler;
/// # use wundergraph::WundergraphEntity;
/// # use wundergraph::query_builder::selection::filter::{
/// #    CreateFilter, FilterBuildHelper, FilterConverter
/// # };
/// # use crate::wundergraph::query_builder::selection::filter::InnerFilter;
///
/// use wundergraph::query_builder::selection::filter::{BuildFilter, BuildFilterHelper};
/// use wundergraph::query_builder::types::HasOne;
/// use wundergraph::diesel_ext::BoxableFilter;
/// use wundergraph::juniper_ext::NameBuilder;
/// use wundergraph::query_builder::selection::fields::WundergraphBelongsTo;
/// use wundergraph::WundergraphContext;
/// use wundergraph::scalar::WundergraphScalarValue;
/// use wundergraph::error::Result;
/// use juniper::{LookAheadValue, Selection, Executor, InputValue, meta, Registry};
/// # #[cfg(feature = "postgres")]
/// use diesel::pg::Pg;
/// use diesel::prelude::*;
/// use diesel::sql_types::Bool;
/// use std::collections::HashMap;
/// use indexmap::IndexMap;
///
/// table! {
///     heros {
///         id -> Integer,
///         name -> Text,
///         species -> Integer,
///     }
/// }
///
/// #[derive(Identifiable)]
/// struct Hero {
///   id: i32,
///   name: String,
/// }
///
/// # #[cfg(feature = "postgres")]
/// # impl<Ctx> LoadingHandler<Pg, Ctx> for Hero
/// # where
/// #    Ctx: WundergraphContext + 'static,
/// #    <Ctx as WundergraphContext>::Connection: Connection<Backend = Pg>,
/// # {
/// #    type Columns = (heros::id, heros::name);
/// #    type FieldList = (i32, String);
/// #    type PrimaryKeyIndex = TupleIndex0;
/// #    type Filter = ();
/// #
/// #    const FIELD_NAMES: &'static [&'static str] = &["id", "name"];
/// #    const TYPE_NAME: &'static str = "Hero";
/// # }
///
/// type Filter<DB, Ctx> = <FilterConverter<Hero, DB, Ctx> as CreateFilter>::Filter;
///
/// # #[cfg(feature = "postgres")]
/// impl<Ctx> BuildFilterHelper<Pg, Filter<Pg, Ctx>, Ctx> for heros::table
/// where
///     Ctx: WundergraphContext + 'static,
///     Ctx::Connection: Connection<Backend = Pg>,
/// {
///     type Ret = Box<dyn BoxableFilter<heros::table, Pg, SqlType = Bool>>;
///     const FIELD_COUNT: usize = 2;
///
///      fn into_filter(
///          f: Filter<Pg, Ctx>
///      ) -> Option<Self::Ret> {
///          BuildFilter::<Pg>::into_filter(f)
///      }
///
///      fn from_inner_look_ahead(
///          objs: &[(&str, LookAheadValue<WundergraphScalarValue>)]
///      ) -> Filter<Pg, Ctx> {
///          FilterBuildHelper::<Filter<Pg, Ctx>, Hero, Pg, Ctx>::from_inner_look_ahead(objs).0
///      }
///
///      fn from_inner_input_value(
///          obj: IndexMap<&str, &InputValue<WundergraphScalarValue>>
///      ) -> Option<Filter<Pg, Ctx>> {
///          Some(FilterBuildHelper::<Filter<Pg, Ctx>, Hero, Pg, Ctx>::from_inner_input_value(obj)?.0)
///      }
///
///      fn to_inner_input_value(
///          f: &Filter<Pg, Ctx>,
///          v: &mut IndexMap<&str, InputValue<WundergraphScalarValue>>
///      ) {}
///
///      fn register_fields<'r>(
///          info: &NameBuilder<()>,
///          registry: &mut Registry<'r, WundergraphScalarValue>,
///      ) -> Vec<meta::Argument<'r, WundergraphScalarValue>> {
///          FilterBuildHelper::<Filter<Pg, Ctx>, Hero, Pg, Ctx>::register_fields(
///              &Default::default(),
///              registry
///          )
///      }
/// }
///
/// # fn main() {}
/// ```
pub trait BuildFilterHelper<DB, F, Ctx>
where
    DB: Backend,
{
    /// Diesel expression type returned by this filter node
    type Ret: Expression<SqlType = Bool> + NonAggregate + QueryFragment<DB>;
    /// Number of fields written by `to_inner_input_value`
    const FIELD_COUNT: usize;

    /// Convert the filter in a expression
    ///
    /// Returns the expression if the filter represents a query part, otherwise null
    fn into_filter(f: F) -> Option<Self::Ret>;

    /// Build the filter from a set of juniper look ahead values
    fn from_inner_look_ahead(objs: &[(&str, LookAheadValue<'_, WundergraphScalarValue>)]) -> F;
    /// Build the filter from a set of juniper input values
    fn from_inner_input_value(
        obj: IndexMap<&str, &InputValue<WundergraphScalarValue>>,
    ) -> Option<F>;

    /// Serialize the filter into a juniper input value
    fn to_inner_input_value(f: &F, _v: &mut IndexMap<&str, InputValue<WundergraphScalarValue>>);

    /// Register the filter as graphql argument
    fn register_fields<'r>(
        _info: &NameBuilder<()>,
        registry: &mut Registry<'r, WundergraphScalarValue>,
    ) -> Vec<Argument<'r, WundergraphScalarValue>>;
}

impl<L, DB, Ctx> BuildFilter<DB> for FilterWrapper<L, DB, Ctx>
where
    DB: Backend + ApplyOffset + 'static,
    L::Table: 'static,
    L: LoadingHandler<DB, Ctx>,
    <L::Table as QuerySource>::FromClause: QueryFragment<DB>,
    DB::QueryBuilder: Default,
    FilterConverter<L, DB, Ctx>: CreateFilter,
    L::Table: BuildFilterHelper<DB, <FilterConverter<L, DB, Ctx> as CreateFilter>::Filter, Ctx>,
{
    type Ret = <L::Table as BuildFilterHelper<
        DB,
        <FilterConverter<L, DB, Ctx> as CreateFilter>::Filter,
        Ctx,
    >>::Ret;

    fn into_filter(self) -> Option<Self::Ret> {
        <L::Table as BuildFilterHelper<DB, _, Ctx>>::into_filter(self.filter)
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct FilterBuildHelper<F, L, DB, Ctx>(pub F, PhantomData<(L, DB, Ctx)>);

impl<F, L, DB, Ctx> Nameable for FilterBuildHelper<F, L, DB, Ctx>
where
    DB: Backend + ApplyOffset + 'static,
    L::Table: 'static,
    L: LoadingHandler<DB, Ctx>,
    <L::Table as QuerySource>::FromClause: QueryFragment<DB>,
    DB::QueryBuilder: Default,
{
    fn name() -> String {
        format!("{}Filter", L::TYPE_NAME)
    }
}

impl<L, DB, Ctx> InnerFilter for FilterWrapper<L, DB, Ctx>
where
    DB: Backend + ApplyOffset + 'static,
    L::Table: 'static,
    L: LoadingHandler<DB, Ctx>,
    <L::Table as QuerySource>::FromClause: QueryFragment<DB>,
    DB::QueryBuilder: Default,
    FilterConverter<L, DB, Ctx>: CreateFilter,
    L::Table: BuildFilterHelper<DB, <FilterConverter<L, DB, Ctx> as CreateFilter>::Filter, Ctx>,
{
    type Context = ();

    const FIELD_COUNT: usize = L::Table::FIELD_COUNT;

    fn from_inner_input_value(
        obj: IndexMap<&str, &InputValue<WundergraphScalarValue>>,
    ) -> Option<Self> {
        Some(Self {
            filter: L::Table::from_inner_input_value(obj)?,
        })
    }

    fn from_inner_look_ahead(objs: &[(&str, LookAheadValue<'_, WundergraphScalarValue>)]) -> Self {
        Self {
            filter: L::Table::from_inner_look_ahead(objs),
        }
    }

    fn to_inner_input_value(&self, v: &mut IndexMap<&str, InputValue<WundergraphScalarValue>>) {
        L::Table::to_inner_input_value(&self.filter, v)
    }

    fn register_fields<'r>(
        _info: &NameBuilder<Self>,
        registry: &mut Registry<'r, WundergraphScalarValue>,
    ) -> Vec<Argument<'r, WundergraphScalarValue>> {
        L::Table::register_fields(&NameBuilder::default(), registry)
    }
}

macro_rules! __impl_build_filter_for_tuples {
    ($(
        $Tuple:tt {
            $(($idx:tt) -> $T:ident, $ST: ident, $TT: ident,) +
        }
    )+) => {
        $(
            impl<$($T,)* Back, Table> BuildFilter<Back> for ($($T,)*)
            where $($T: BuildFilter<Back, Ret = Box<dyn BoxableFilter<Table, Back, SqlType = Bool>>> + 'static,)*
                  Back: Backend + 'static,
                  Table: 'static
            {
                type Ret = Box<dyn BoxableFilter<Table, Back, SqlType = Bool>>;

                fn into_filter(self) -> Option<Self::Ret> {
                    use crate::query_builder::selection::filter::collector::{AndCollector, FilterCollector};

                    let mut and = AndCollector::<_, Back>::default();
                    $(
                        and.append_filter(self.$idx);
                    )*

                        and.into_filter()
                }
            }

            impl<$($T,)* Loading, Back, Ctx> InnerFilter for FilterBuildHelper<($(Option<$T>,)*), Loading, Back, Ctx>
            where Back: Backend + ApplyOffset + 'static,
                Loading::Table: 'static,
                Loading: LoadingHandler<Back, Ctx>,
                <Loading::Table as QuerySource>::FromClause: QueryFragment<Back>,
                Back::QueryBuilder: Default,
                $($T: GraphQLType<WundergraphScalarValue, TypeInfo = NameBuilder<$T>> + ToInputValue<WundergraphScalarValue> + FromInputValue<WundergraphScalarValue> + Nameable + FromLookAheadValue,)*
            {
                type Context = ();

                const FIELD_COUNT: usize = $Tuple;

                fn from_inner_input_value(
                    obj: IndexMap<&str, &InputValue<WundergraphScalarValue>>
                ) -> Option<Self> {
                    let mut values = ($(Option::<$T>::default(),)*);
                    for (name, value) in obj {
                        match name {
                            $(
                                n if n == Loading::FIELD_NAMES[$idx] => {
                                    values.$idx = <$T as FromInputValue<WundergraphScalarValue>>::from_input_value(value);
                                }
                            )*
                                _  => {}
                        }
                    }
                    Some(FilterBuildHelper(values, PhantomData))
                }

                fn from_inner_look_ahead(
                    objs: &[(&str, LookAheadValue<'_, WundergraphScalarValue>)]
                ) -> Self {
                    let mut values = ($(Option::<$T>::default(),)*);
                    for (name, value) in objs {
                        match name {
                            $(
                                n if *n == Loading::FIELD_NAMES[$idx] => {
                                    values.$idx = <$T as FromLookAheadValue>::from_look_ahead(value);
                                }
                            )*
                            _  => {}
                        }
                    }
                    FilterBuildHelper(values, PhantomData)
                }

                fn to_inner_input_value(
                    &self, v: &mut IndexMap<&str, InputValue<WundergraphScalarValue>>
                ) {
                    let inner = &self.0;
                    $(
                        let value = <Option<$T> as ToInputValue<WundergraphScalarValue>>::to_input_value(&inner.$idx);
                        v.insert(Loading::FIELD_NAMES[$idx], value);
                    )*
                }

                fn register_fields<'r>(
                    _info: &NameBuilder<Self>,
                    registry: &mut Registry<'r, WundergraphScalarValue>,
                ) -> Vec<Argument<'r, WundergraphScalarValue>> {
                    vec![
                        $(
                            registry.arg_with_default::<Option<$T>>(
                                Loading::FIELD_NAMES[$idx],
                                &None,
                                &NameBuilder::default()
                            ),
                        )*
                    ]
                }
            }

            impl<$($T,)* $($ST,)* Back, Ctx> CreateFilter for ColumnFilterConverter<($($T,)*), ($($ST,)*), Back, Ctx>
                where $($T: AsColumnFilter<$ST, Back, Ctx>,)*
            {
                type Filter = ($(Option<<$T as AsColumnFilter<$ST, Back, Ctx>>::Filter>,)*);
            }

            impl<$($T,)* Loading, Back, Ctx> CreateFilter for NonColumnFilterConveter<($($T,)*), Loading, Back, Ctx>
                where $($T: AsNonColumnFilter<Loading, Back, Ctx>,)*
            {
                type Filter = ($(Option<<$T as AsNonColumnFilter<Loading, Back, Ctx>>::Filter>,)*);
            }
        )*
    }
}

__diesel_for_each_tuple!(__impl_build_filter_for_tuples);
