use std::marker::PhantomData;

use filter::build_filter::BuildFilter;
use filter::collector::{AndCollector, FilterCollector};
use filter::inner_filter::InnerFilter;
use filter::transformator::{OnlyExclusive, OnlySelective, Transformator};

use diesel::associations::HasTable;
use diesel::backend::Backend;
use diesel::dsl::{EqAny, Filter, IsNotNull, NeAny, Select, SqlTypeOf};
use diesel::expression::array_comparison::AsInExpression;
use diesel::expression::nullable::Nullable;
use diesel::expression::NonAggregate;
use diesel::query_builder::{AsQuery, BoxedSelectStatement, QueryFragment};
use diesel::query_dsl::methods::{BoxedDsl, FilterDsl, SelectDsl};
use diesel::sql_types::{Bool, NotNull, SingleValue};
use diesel::{
    AppearsOnTable, Column, Expression, ExpressionMethods, NullableExpressionMethods, QueryDsl,
};
use diesel_ext::BoxableFilter;

use juniper::meta::{Argument, MetaType};
use juniper::{FromInputValue, GraphQLType, InputValue, LookAheadValue, Registry, ToInputValue};

use indexmap::IndexMap;

use helper::{FromLookAheadValue, NameBuilder, Nameable};
use scalar::WundergraphScalarValue;

#[derive(Debug)]
pub struct ReverseNullableReferenceFilter<C, I, C2> {
    inner: Box<I>,
    p: PhantomData<(C, C2)>,
}

impl<C, I, C2> Clone for ReverseNullableReferenceFilter<C, I, C2>
where
    I: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            p: PhantomData,
        }
    }
}

impl<C, DB, I, C2> BuildFilter<DB> for ReverseNullableReferenceFilter<C, I, C2>
where
    C: Column + NonAggregate + QueryFragment<DB> + Default + 'static,
    C::SqlType: SingleValue + NotNull,
    Nullable<C>: ExpressionMethods,
    DB: Backend + 'static,
    I: BuildFilter<DB> + Clone + InnerFilter,
    C::Table: 'static,
    C2::Table: HasTable<Table = C2::Table> + 'static,
    C2: Column + ExpressionMethods + NonAggregate + QueryFragment<DB> + Default + 'static,
    <C2::Table as AsQuery>::Query: FilterDsl<I::Ret>,
    Filter<<C2::Table as AsQuery>::Query, I::Ret>: FilterDsl<IsNotNull<C2>> + SelectDsl<C2>,
    Filter<Filter<<C2::Table as AsQuery>::Query, I::Ret>, IsNotNull<C2>>: SelectDsl<C2>,
    Select<Filter<Filter<<C2::Table as AsQuery>::Query, I::Ret>, IsNotNull<C2>>, C2>:
        BoxedDsl<'static, DB, Output = BoxedSelectStatement<'static, C2::SqlType, C2::Table, DB>>
            + QueryDsl,
    Select<Filter<<C2::Table as AsQuery>::Query, I::Ret>, C2>:
        BoxedDsl<'static, DB, Output = BoxedSelectStatement<'static, C2::SqlType, C2::Table, DB>>
            + QueryDsl,
    BoxedSelectStatement<'static, C2::SqlType, C2::Table, DB>:
        AsInExpression<SqlTypeOf<Nullable<C>>>,
    EqAny<Nullable<C>, BoxedSelectStatement<'static, C2::SqlType, C2::Table, DB>>:
        AppearsOnTable<C::Table> + QueryFragment<DB> + Expression<SqlType = Bool>,
    NeAny<Nullable<C>, BoxedSelectStatement<'static, C2::SqlType, C2::Table, DB>>:
        AppearsOnTable<C::Table> + QueryFragment<DB> + Expression<SqlType = Bool>,
{
    type Ret = Box<BoxableFilter<C::Table, DB, SqlType = Bool>>;

    fn into_filter<F>(self, t: F) -> Option<Self::Ret>
    where
        F: Transformator,
    {
        let mut and = AndCollector::default();

        let selective_inner = self
            .inner
            .clone()
            .into_filter(OnlySelective)
            .map(|f| <_ as FilterDsl<I::Ret>>::filter(C2::Table::table(), f))
            .map(|f| <_ as FilterDsl<IsNotNull<C2>>>::filter(f, C2::default().is_not_null()))
            .map(|f| <_ as SelectDsl<C2>>::select(f, C2::default()))
            .map(|f| f.into_boxed())
            .map(|f| Box::new(C::default().nullable().eq_any(f)) as Box<_>);
        and.append_filter(selective_inner, t);

        let exclusive_inner = self
            .inner
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

impl<C, I, C2> Nameable for ReverseNullableReferenceFilter<C, I, C2>
where
    I: Nameable,
{
    fn name() -> String {
        I::name()
    }
}

impl<C, I, C2> FromInputValue<WundergraphScalarValue> for ReverseNullableReferenceFilter<C, I, C2>
where
    I: InnerFilter,
{
    fn from_input_value(v: &InputValue<WundergraphScalarValue>) -> Option<Self> {
        if let Some(obj) = v.to_object_value() {
            I::from_inner_input_value(obj).map(|inner| Self {
                inner: Box::new(inner),
                p: PhantomData,
            })
        } else {
            None
        }
    }
}

impl<C, I, C2> ToInputValue<WundergraphScalarValue> for ReverseNullableReferenceFilter<C, I, C2>
where
    I: InnerFilter,
{
    fn to_input_value(&self) -> InputValue<WundergraphScalarValue> {
        let mut map = IndexMap::with_capacity(I::FIELD_COUNT);
        self.inner.to_inner_input_value(&mut map);
        InputValue::object(map)
    }
}

impl<C, I, C2> FromLookAheadValue for ReverseNullableReferenceFilter<C, I, C2>
where
    I: InnerFilter,
{
    fn from_look_ahead(v: &LookAheadValue<WundergraphScalarValue>) -> Option<Self> {
        if let LookAheadValue::Object(ref obj) = *v {
            let inner = I::from_inner_look_ahead(obj);
            Some(Self {
                inner: Box::new(inner),
                p: PhantomData,
            })
        } else {
            None
        }
    }
}

impl<C, I, C2> GraphQLType<WundergraphScalarValue> for ReverseNullableReferenceFilter<C, I, C2>
where
    I: InnerFilter,
{
    type Context = I::Context;
    type TypeInfo = NameBuilder<Self>;

    fn name(info: &Self::TypeInfo) -> Option<&str> {
        Some(info.name())
    }

    fn meta<'r>(
        info: &Self::TypeInfo,
        registry: &mut Registry<'r, WundergraphScalarValue>,
    ) -> MetaType<'r, WundergraphScalarValue>
    where
        WundergraphScalarValue: 'r,
    {
        let fields = I::register_fields(&NameBuilder::default(), registry);
        registry
            .build_input_object_type::<Self>(info, &fields)
            .into_meta()
    }
}

impl<C, I, C2> InnerFilter for ReverseNullableReferenceFilter<C, I, C2>
where
    I: InnerFilter,
{
    type Context = I::Context;

    const FIELD_COUNT: usize = I::FIELD_COUNT;

    fn from_inner_input_value(
        obj: IndexMap<&str, &InputValue<WundergraphScalarValue>>,
    ) -> Option<Self> {
        let inner = match I::from_inner_input_value(obj) {
            Some(inner) => Box::new(inner),
            None => return None,
        };
        Some(Self {
            inner,
            p: PhantomData,
        })
    }

    fn from_inner_look_ahead(obj: &[(&str, LookAheadValue<WundergraphScalarValue>)]) -> Self {
        let inner = I::from_inner_look_ahead(obj);
        Self {
            inner: Box::new(inner),
            p: PhantomData,
        }
    }

    fn to_inner_input_value(&self, map: &mut IndexMap<&str, InputValue<WundergraphScalarValue>>) {
        self.inner.to_inner_input_value(map);
    }

    fn register_fields<'r>(
        _info: &NameBuilder<Self>,
        registry: &mut Registry<'r, WundergraphScalarValue>,
    ) -> Vec<Argument<'r, WundergraphScalarValue>> {
        I::register_fields(&NameBuilder::default(), registry)
    }
}
