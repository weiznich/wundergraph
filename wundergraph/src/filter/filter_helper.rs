use diesel::backend::Backend;
use diesel::query_builder::QueryFragment;
use diesel::sql_types::Bool;
use diesel::AppearsOnTable;
use diesel::Column;
use diesel::QuerySource;
use diesel::Table;
use diesel_ext::BoxableFilter;
use filter::build_filter::BuildFilter;
use filter::common_filter::FilterOption;
use filter::filter_value::FilterValue;
use filter::inner_filter::InnerFilter;
use filter::nullable_filter::NullableReferenceFilter;
use filter::reference_filter::ReferenceFilter;
use filter::Filter;
use helper::{FromLookAheadValue, NameBuilder, Nameable};
use indexmap::IndexMap;
use juniper::meta::Argument;
use juniper::{FromInputValue, GraphQLType, InputValue, LookAheadValue, Registry, ToInputValue};
use query_helper::placeholder::WundergraphBelongsTo;
use query_helper::{HasMany, HasOne};
use scalar::WundergraphScalarValue;
use std::marker::PhantomData;
use LoadingHandler;


#[derive(Debug, Clone)]
pub struct FilterWrapper<C, L, DB>(C, PhantomData<(DB, L)>);

#[derive(Debug)]
pub struct FilterConverter<C, T, L, DB>(PhantomData<(C, DB, L, T)>);

pub trait CreateFilter {
    type Filter;
}

pub trait AsFilter<C, DB> {
    type Filter;
}

impl<C, DB> AsFilter<C, DB> for i16 {
    type Filter = FilterOption<i16, C>;
}

impl<C, DB> AsFilter<C, DB> for i32 {
    type Filter = FilterOption<i32, C>;
}

impl<C, DB> AsFilter<C, DB> for i64 {
    type Filter = FilterOption<i64, C>;
}

impl<C, DB> AsFilter<C, DB> for bool {
    type Filter = FilterOption<bool, C>;
}

impl<C, DB> AsFilter<C, DB> for f32 {
    type Filter = FilterOption<f32, C>;
}

impl<C, DB> AsFilter<C, DB> for f64 {
    type Filter = FilterOption<f64, C>;
}

impl<C, DB> AsFilter<C, DB> for String {
    type Filter = FilterOption<String, C>;
}

impl<C, DB, T> AsFilter<C, DB> for Vec<T>
where
    T: FromLookAheadValue
        + FromInputValue<WundergraphScalarValue>
        + ToInputValue<WundergraphScalarValue>
        + FilterValue<C>
        + Clone,
{
    type Filter = FilterOption<Vec<T>, C>;
}

#[cfg(feature = "chrono")]
impl<C, DB> AsFilter<C, DB> for chrono::NaiveDateTime {
    type Filter = FilterOption<Self, C>;
}

impl<C, DB, T> AsFilter<C, DB> for Option<T>
where
    T: FilterValue<C>
        + Clone
        + FromInputValue<WundergraphScalarValue>
        + FromLookAheadValue
        + ToInputValue<WundergraphScalarValue>,
    T: AsFilter<C, DB, Filter = FilterOption<T, C>>,
{
    type Filter = FilterOption<Option<T>, C>;
}

impl<C, K, I, DB> AsFilter<C, DB> for HasOne<K, I>
where
    DB: Backend + 'static,
    I::Table: 'static,
    I: LoadingHandler<DB>,
    <I::Table as QuerySource>::FromClause: QueryFragment<DB>,
    DB::QueryBuilder: Default,
{
    type Filter = ReferenceFilter<C, Filter<I::Filter, I::Table>, <I::Table as Table>::PrimaryKey>;
}

impl<C, DB, O> AsFilter<C, DB> for HasMany<O>
where
    C: Column,
    DB: Backend + 'static,
    O::Table: 'static,
    O: LoadingHandler<DB>,
    O: WundergraphBelongsTo<C::Table, DB>,
    <O::Table as QuerySource>::FromClause: QueryFragment<DB>,
    DB::QueryBuilder: Default,
{
    type Filter = ReferenceFilter<
        <O::Table as Table>::PrimaryKey,
        Filter<<O as LoadingHandler<DB>>::Filter, O::Table>,
        <O as WundergraphBelongsTo<C::Table, DB>>::ForeignKeyColumn,
    >;
}

impl<C, K, I, DB> AsFilter<C, DB> for Option<HasOne<K, I>>
where
    DB: Backend + 'static,
    I::Table: 'static,
    I: LoadingHandler<DB>,
    <I::Table as QuerySource>::FromClause: QueryFragment<DB>,
    DB::QueryBuilder: Default,
{
    type Filter =
        NullableReferenceFilter<C, Filter<I::Filter, I::Table>, <I::Table as Table>::PrimaryKey>;
}

macro_rules! __impl_build_filter_for_tuples {
    ($(
        $Tuple:tt {
            $(($idx:tt) -> $T:ident, $ST: ident, $TT: ident,) +
        }
    )+) => {
        $(
            impl<$($T,)* Back, Loading> BuildFilter<Back> for FilterWrapper<($($T,)*), Loading, Back>
            where $(
                $T: BuildFilter<Back> + 'static,
                $T::Ret: AppearsOnTable<Loading::Table>,
            )*
                Back: Backend + 'static,
                Loading::Table: 'static,
                Loading: LoadingHandler<Back>,
                <Loading::Table as QuerySource>::FromClause: QueryFragment<Back>,
                Back::QueryBuilder: Default,

            {
                type Ret = Box<BoxableFilter<Loading::Table, Back, SqlType = Bool>>;

                fn into_filter(self) -> Option<Self::Ret> {
                    use filter::collector::{AndCollector, FilterCollector};

                    let mut and = AndCollector::<_, Back>::default();
                    $(
                        and.append_filter((self.0).$idx);
                    )*

                        and.into_filter()
                }
            }

            impl<$($T,)* Loading, Back> Nameable for FilterWrapper<($($T,)*), Loading, Back>
            where
                Back: Backend + 'static,
                Loading::Table: 'static,
                Loading: LoadingHandler<Back>,
                <Loading::Table as QuerySource>::FromClause: QueryFragment<Back>,
                Back::QueryBuilder: Default,
            {
                fn name() -> String {
                    format!("{}Filter", Loading::TYPE_NAME)
                }
            }

            impl<$($T,)* Back, Loading> InnerFilter for FilterWrapper<($(Option<$T>,)*), Loading, Back>
            where Back: Backend + 'static,
                Loading::Table: 'static,
                Loading: LoadingHandler<Back>,
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
                    objs: &[(&str, LookAheadValue<WundergraphScalarValue>)]
                ) -> Self {
                    use query_helper::placeholder::WundergraphFieldList;
                    let mut values = ($(Option::<$T>::None,)*);
                    for (name, value) in objs {
                        match name {
                            $(
                                n if *n == Loading::FieldList::map_table_field($idx, |i| Loading::FIELD_NAMES[i]).expect("Field name is there") => {
                                    values.$idx = <$T as FromLookAheadValue>::from_look_ahead(value);
                                }
                            )*
                            _  => {}
                        }
                    }
                    FilterWrapper(values, Default::default())
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
                    use query_helper::placeholder::WundergraphFieldList;
                    vec![
                        $(
                            registry.arg_with_default::<Option<$T>>(
                                Loading::FieldList::map_table_field($idx, |i| Loading::FIELD_NAMES[i]).expect("Field name is there"),
                                &None,
                                &NameBuilder::default()
                            ),
                        )*
                    ]
                }
            }

            impl<$($T,)* $($ST,)* Back, Loading> CreateFilter for FilterConverter<($($T,)*), ($($ST,)*), Loading, Back>
                where $($T: AsFilter<$ST, Back>,)*
            {
                type Filter = ($(Option<<$T as AsFilter<$ST, Back>>::Filter>,)*);
            }
        )*
    }
}

__diesel_for_each_tuple!(__impl_build_filter_for_tuples);
