use filter::collector::{AndCollector, FilterCollector};
use filter::inner_filter::InnerFilter;
use filter::build_filter::BuildFilter;
use filter::transformator::{OnlyExclusive, OnlySelective, Transformator};

use diesel::{BoxableExpression, Column, Expression, ExpressionMethods, NullableExpressionMethods,
             QueryDsl, SelectableExpression};
use diesel::query_dsl::methods::{BoxedDsl, FilterDsl, SelectDsl};
use diesel::sql_types::{Bool, NotNull, SingleValue};
use diesel::backend::Backend;
use diesel::expression::NonAggregate;
use diesel::associations::HasTable;
use diesel::query_builder::{AsQuery, BoxedSelectStatement, QueryFragment};
use diesel::expression::array_comparison::AsInExpression;
use diesel::expression::nullable::Nullable;
use diesel::dsl::{EqAny, Filter, IsNotNull, NeAny, Select, SqlTypeOf};

use juniper::{FromInputValue, GraphQLType, InputValue, LookAheadValue, Registry, ToInputValue};
use juniper::meta::{Argument, MetaType};

use ordermap::OrderMap;

use helper::{FromLookAheadValue, NameBuilder, Nameable};

#[derive(Debug)]
pub struct ReverseNullableReferenceFilter<C, DB, I, C2> {
    inner: Box<I>,
    p: ::std::marker::PhantomData<(C, DB, C2)>,
}

impl<C, DB, I, C2> Clone for ReverseNullableReferenceFilter<C, DB, I, C2>
where
    I: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            p: Default::default(),
        }
    }
}

impl<C, DB, I, C2> BuildFilter for ReverseNullableReferenceFilter<C, DB, I, C2>
where
    C: Column + NonAggregate + QueryFragment<DB> + Default + 'static,
    C::SqlType: SingleValue + NotNull,
    Nullable<C>: SelectableExpression<C::Table> + ExpressionMethods,
    DB: Backend + 'static,
    I: BuildFilter + Clone + InnerFilter,
    C::Table: 'static,
    C2::Table: HasTable<Table = C2::Table> + 'static,
    C2: Column + ExpressionMethods + NonAggregate + QueryFragment<DB> + Default + 'static,
    <C2::Table as AsQuery>::Query: FilterDsl<I::Ret>,
    Filter<<C2::Table as AsQuery>::Query, I::Ret>: FilterDsl<IsNotNull<C2>> + SelectDsl<C2>,
    Filter<Filter<<C2::Table as AsQuery>::Query, I::Ret>, IsNotNull<C2>>: SelectDsl<C2>,
    Select<Filter<Filter<<C2::Table as AsQuery>::Query, I::Ret>, IsNotNull<C2>>, C2>: BoxedDsl<'static, DB, Output = BoxedSelectStatement<'static, C2::SqlType, C2::Table, DB>>
        + QueryDsl,
    Select<Filter<<C2::Table as AsQuery>::Query, I::Ret>, C2>: BoxedDsl<'static, DB, Output = BoxedSelectStatement<'static, C2::SqlType, C2::Table, DB>>
        + QueryDsl,
    BoxedSelectStatement<'static, C2::SqlType, C2::Table, DB>: AsInExpression<SqlTypeOf<Nullable<C>>>,
    EqAny<Nullable<C>, BoxedSelectStatement<'static, C2::SqlType, C2::Table, DB>>: SelectableExpression<C::Table>
        + QueryFragment<DB>
        + Expression<SqlType = Bool>,
    NeAny<Nullable<C>, BoxedSelectStatement<'static, C2::SqlType, C2::Table, DB>>: SelectableExpression<C::Table>
        + QueryFragment<DB>
        + Expression<SqlType = Bool>,
{
    type Ret = Box<BoxableExpression<C::Table, DB, SqlType = Bool>>;

    fn into_filter<F>(self, t: F) -> Option<Self::Ret>
    where
        F: Transformator,
    {
        let mut and = AndCollector::default();

        let selective_inner = self.inner
            .clone()
            .into_filter(OnlySelective)
            .map(|f| <_ as FilterDsl<I::Ret>>::filter(C2::Table::table(), f))
            .map(|f| <_ as FilterDsl<IsNotNull<C2>>>::filter(f, C2::default().is_not_null()))
            .map(|f| <_ as SelectDsl<C2>>::select(f, C2::default()))
            .map(|f| f.into_boxed())
            .map(|f| Box::new(C::default().nullable().eq_any(f)) as Box<_>);
        and.append_filter(selective_inner, t);

        let exclusive_inner = self.inner
            .clone()
            .into_filter(OnlyExclusive)
            .map(|f| <_ as FilterDsl<I::Ret>>::filter(C2::Table::table(), f))
            .map(|f| <_ as SelectDsl<C2>>::select(f, C2::default()))
            .map(|f| f.into_boxed())
            .map(|f| Box::new(C::default().nullable().ne_all(f)) as Box<_>);

        and.append_filter(exclusive_inner, t);

        and.into_filter(t)
    }
}

impl<C, DB, I, C2> Nameable for ReverseNullableReferenceFilter<C, DB, I, C2>
where
    I: Nameable,
{
    fn name() -> String {
        I::name()
    }
}

impl<C, I, DB, C2> FromInputValue for ReverseNullableReferenceFilter<C, DB, I, C2>
where
    I: InnerFilter,
{
    fn from_input_value(v: &InputValue) -> Option<Self> {
        if let Some(obj) = v.to_object_value() {
            I::from_inner_input_value(obj).map(|inner| Self {
                inner: Box::new(inner),
                p: Default::default(),
            })
        } else {
            None
        }
    }
}

impl<C, I, DB, C2> ToInputValue for ReverseNullableReferenceFilter<C, DB, I, C2>
where
    I: InnerFilter,
{
    fn to_input_value(&self) -> InputValue {
        let mut map = OrderMap::with_capacity(I::FIELD_COUNT);
        self.inner.to_inner_input_value(&mut map);
        InputValue::object(map)
    }
}

impl<C, I, DB, C2> FromLookAheadValue for ReverseNullableReferenceFilter<C, DB, I, C2>
where
    I: InnerFilter,
{
    fn from_look_ahead(v: &LookAheadValue) -> Option<Self> {
        if let LookAheadValue::Object(ref obj) = *v {
            let inner = I::from_inner_look_ahead(obj);
            Some(Self {
                inner: Box::new(inner),
                p: Default::default(),
            })
        } else {
            None
        }
    }
}

impl<C, I, DB, C2> GraphQLType for ReverseNullableReferenceFilter<C, DB, I, C2>
where
    I: InnerFilter,
{
    type Context = I::Context;
    type TypeInfo = NameBuilder<Self>;

    fn name(info: &Self::TypeInfo) -> Option<&str> {
        Some(info.name())
    }

    fn meta<'r>(info: &Self::TypeInfo, registry: &mut Registry<'r>) -> MetaType<'r> {
        let fields = I::register_fields(&Default::default(), registry);
        registry
            .build_input_object_type::<Self>(info, &fields)
            .into_meta()
    }
}

impl<C, I, DB, C2> InnerFilter for ReverseNullableReferenceFilter<C, DB, I, C2>
where
    C: Column + NonAggregate + QueryFragment<DB> + Default + 'static,
    C::SqlType: SingleValue + NotNull,
    Nullable<C>: SelectableExpression<C::Table> + ExpressionMethods,
    DB: Backend + 'static,
    I: BuildFilter + Clone + InnerFilter,
    C::Table: 'static,
    C2::Table: HasTable<Table = C2::Table>,
    C2: Column + ExpressionMethods + NonAggregate + QueryFragment<DB> + Default + 'static,
    <C2::Table as AsQuery>::Query: FilterDsl<I::Ret>,
    Filter<<C2::Table as AsQuery>::Query, I::Ret>: FilterDsl<IsNotNull<C2>> + SelectDsl<C2>,
    Filter<Filter<<C2::Table as AsQuery>::Query, I::Ret>, IsNotNull<C2>>: SelectDsl<C2>,
    Select<Filter<Filter<<C2::Table as AsQuery>::Query, I::Ret>, IsNotNull<C2>>, C2>: BoxedDsl<'static, DB, Output = BoxedSelectStatement<'static, C2::SqlType, C2::Table, DB>>
        + QueryDsl,
    Select<Filter<<C2::Table as AsQuery>::Query, I::Ret>, C2>: BoxedDsl<'static, DB, Output = BoxedSelectStatement<'static, C2::SqlType, C2::Table, DB>>
        + QueryDsl,
    ReverseNullableReferenceFilter<C, DB, I, C2>: BuildFilter,
{
    type Context = I::Context;

    const FIELD_COUNT: usize = I::FIELD_COUNT;

    fn from_inner_input_value(obj: OrderMap<&str, &InputValue>) -> Option<Self> {
        let inner = match I::from_inner_input_value(obj) {
            Some(inner) => Box::new(inner),
            None => return None,
        };
        Some(Self {
            inner,
            p: Default::default(),
        })
    }

    fn from_inner_look_ahead(obj: &[(&str, LookAheadValue)]) -> Self {
        let inner = I::from_inner_look_ahead(obj);
        Self {
            inner: Box::new(inner),
            p: Default::default(),
        }
    }

    fn to_inner_input_value(&self, map: &mut OrderMap<&str, InputValue>) {
        self.inner.to_inner_input_value(map);
    }

    fn register_fields<'r>(
        _info: &NameBuilder<Self>,
        registry: &mut Registry<'r>,
    ) -> Vec<Argument<'r>> {
        I::register_fields(&Default::default(), registry)
    }
}
