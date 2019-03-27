use crate::diesel_ext::BoxableFilter;
use crate::filter::build_filter::BuildFilter;
use crate::filter::common_filter::FilterOption;
use crate::filter::filter_value::FilterValue;
use crate::filter::inner_filter::InnerFilter;
use crate::filter::nullable_filter::IsNull;
use crate::filter::reference_filter::ReferenceFilter;
use crate::filter::Filter;
use crate::helper::{FromLookAheadValue, NameBuilder, Nameable};
use crate::query_helper::placeholder::WundergraphBelongsTo;
use crate::query_helper::placeholder::{FieldListExtractor, NonTableFieldExtractor};
use crate::query_helper::tuple::ConcatTuples;
use crate::query_helper::{HasMany, HasOne};
use crate::scalar::WundergraphScalarValue;
use crate::LoadingHandler;
use diesel::associations::HasTable;
use diesel::backend::Backend;
use diesel::expression::NonAggregate;
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

#[derive(Debug)]
pub struct FilterConverter<L, DB, Ctx>(PhantomData<(L, DB, Ctx)>);

#[derive(Debug)]
pub struct ColumnFilterConverter<N, C, DB, Ctx>(PhantomData<(N, C, DB, Ctx)>);

#[derive(Debug)]
pub struct NonColumnFilterConveter<N, L, DB, Ctx>(PhantomData<(N, L, DB, Ctx)>);

pub trait CreateFilter {
    type Filter;
}

impl<L, DB, Ctx> CreateFilter for FilterConverter<L, DB, Ctx>
where
    DB: Backend + 'static,
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

pub trait AsColumnFilter<C, DB, Ctx> {
    type Filter;
}

pub trait AsNonColumnFilter<L, DB, Ctx> {
    type Filter;
}

impl<L, O, DB, Ctx> AsNonColumnFilter<L, DB, Ctx> for HasMany<O>
where
    L: HasTable,
    O: WundergraphBelongsTo<L::Table, DB, Ctx>,
    O::Table: 'static,
    DB: Backend + 'static,
    <O::Table as QuerySource>::FromClause: QueryFragment<DB>,
    DB::QueryBuilder: Default,
{
    type Filter = ReferenceFilter<
        <L::Table as Table>::PrimaryKey,
        Filter<O::Filter, O::Table>,
        O::ForeignKeyColumn,
        (),
    >;
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

#[cfg(feature = "chrono")]
impl<C, DB, Ctx> AsColumnFilter<C, DB, Ctx> for chrono::NaiveDateTime {
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
    DB: Backend + 'static,
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
    DB: Backend + 'static,
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
    DB: Backend + 'static,
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

pub trait BuildFilterHelper<DB, F, Ctx>
where
    DB: Backend,
{
    type Ret: Expression<SqlType = Bool> + NonAggregate + QueryFragment<DB>;
    const FIELD_COUNT: usize;

    fn into_filter(f: F) -> Option<Self::Ret>;

    fn from_inner_look_ahead(objs: &[(&str, LookAheadValue<'_, WundergraphScalarValue>)]) -> F;

    fn register_fields<'r>(
        _info: &NameBuilder<()>,
        registry: &mut Registry<'r, WundergraphScalarValue>,
    ) -> Vec<Argument<'r, WundergraphScalarValue>>;
}

impl<L, DB, Ctx> BuildFilter<DB> for FilterWrapper<L, DB, Ctx>
where
    DB: Backend + 'static,
    L::Table: 'static,
    L: LoadingHandler<DB, Ctx>,
    <L::Table as QuerySource>::FromClause: QueryFragment<DB>,
    DB::QueryBuilder: Default,
    FilterConverter<L, DB, Ctx>: CreateFilter,
    L::Table: BuildFilterHelper<DB, <FilterConverter<L, DB, Ctx> as CreateFilter>::Filter, Ctx>,
{
    type Ret =
        <L::Table as BuildFilterHelper<DB, <FilterConverter<L, DB, Ctx> as CreateFilter>::Filter, Ctx>>::Ret;

    fn into_filter(self) -> Option<Self::Ret> {
        <L::Table as BuildFilterHelper<DB, _, Ctx>>::into_filter(self.filter)
    }
}

#[derive(Debug)]
pub struct FilterBuildHelper<F, L, DB, Ctx>(pub F, PhantomData<(L, DB, Ctx)>);

impl<F, L, DB, Ctx> Nameable for FilterBuildHelper<F, L, DB, Ctx>
where
    DB: Backend + 'static,
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
    DB: Backend + 'static,
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
        _obj: IndexMap<&str, &InputValue<WundergraphScalarValue>>,
    ) -> Option<Self> {
        unimplemented!()
    }

    fn from_inner_look_ahead(objs: &[(&str, LookAheadValue<'_, WundergraphScalarValue>)]) -> Self {
        Self {
            filter: L::Table::from_inner_look_ahead(objs),
        }
    }

    fn to_inner_input_value(&self, _v: &mut IndexMap<&str, InputValue<WundergraphScalarValue>>) {
        unimplemented!()
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
                       use crate::filter::collector::{AndCollector, FilterCollector};

                    let mut and = AndCollector::<_, Back>::default();
                    $(
                        and.append_filter(self.$idx);
                    )*

                        and.into_filter()
                }
            }


            impl<$($T,)* Loading, Back, Ctx> InnerFilter for FilterBuildHelper<($(Option<$T>,)*), Loading, Back, Ctx>
            where Back: Backend + 'static,
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
                    dbg!(obj);
                    unimplemented!()
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
                    dbg!(v);
                    unimplemented!()
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
