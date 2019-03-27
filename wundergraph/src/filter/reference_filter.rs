use std::marker::PhantomData;

use crate::filter::build_filter::BuildFilter;
use crate::filter::collector::{AndCollector, FilterCollector};
use crate::filter::inner_filter::InnerFilter;

use crate::diesel_ext::BoxableFilter;
use diesel::associations::HasTable;
use diesel::backend::Backend;
use diesel::dsl::{EqAny, Filter, NullableSelect, Select, SqlTypeOf};
use diesel::expression::array_comparison::AsInExpression;
use diesel::expression::nullable::Nullable;
use diesel::expression::NonAggregate;
use diesel::query_builder::{AsQuery, BoxedSelectStatement, Query, QueryFragment};
use diesel::query_dsl::methods::{BoxedDsl, FilterDsl, SelectDsl, SelectNullableDsl};
use diesel::sql_types::{Bool, SingleValue};
use diesel::{AppearsOnTable, Column, ExpressionMethods, NullableExpressionMethods, QueryDsl};

use juniper::meta::{Argument, MetaType};
use juniper::{FromInputValue, GraphQLType, InputValue, LookAheadValue, Registry, ToInputValue};

use crate::scalar::WundergraphScalarValue;
use indexmap::IndexMap;

use crate::helper::{FromLookAheadValue, NameBuilder, Nameable};

#[derive(Debug)]
pub struct ReferenceFilter<C, I, C2, A = ()> {
    inner: Box<I>,
    additional: A,
    p: PhantomData<(C, I, C2)>,
}

impl<C, I, C2, A> Clone for ReferenceFilter<C, I, C2, A>
where
    I: Clone,
    A: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            additional: self.additional.clone(),
            p: PhantomData,
        }
    }
}

impl<C, DB, I, C2, A> BuildFilter<DB> for ReferenceFilter<C, I, C2, A>
where
    C: Column + NonAggregate + QueryFragment<DB> + Default + 'static,
    Nullable<C>: ExpressionMethods,
    C::SqlType: SingleValue,
    A: BuildFilter<DB> + 'static,
    C::Table: 'static,
    DB: Backend + 'static,
    I: BuildFilter<DB> + InnerFilter,
    C2: Column + NonAggregate + QueryFragment<DB> + Default + 'static,
    C2::Table: HasTable<Table = C2::Table>,
    <C2::Table as AsQuery>::Query: FilterDsl<I::Ret>,
    Filter<<C2::Table as AsQuery>::Query, I::Ret>: QueryDsl + SelectDsl<C2>,
    Select<Filter<<C2::Table as AsQuery>::Query, I::Ret>, C2>: QueryDsl + SelectNullableDsl,
NullableSelect<Select<Filter<<C2::Table as AsQuery>::Query, I::Ret>, C2>>: QueryDsl + Query
    + BoxedDsl<
        'static,
    DB,
    Output = BoxedSelectStatement<
            'static,
        <NullableSelect<Select<Filter<<C2::Table as AsQuery>::Query, I::Ret>, C2>> as Query>::SqlType,
        C2::Table,
        DB,
        >,
    > + 'static,
    BoxedSelectStatement<
        'static,
        <NullableSelect<Select<Filter<<C2::Table as AsQuery>::Query, I::Ret>, C2>> as Query>::SqlType,
        C2::Table,
        DB,
    >: AsInExpression<SqlTypeOf<Nullable<C>>>,
    <BoxedSelectStatement<
        'static,
        <NullableSelect<Select<Filter<<C2::Table as AsQuery>::Query, I::Ret>, C2>> as Query>::SqlType,
        C2::Table,
        DB,
        > as AsInExpression<SqlTypeOf<Nullable<C>>>>::InExpression: AppearsOnTable<C::Table> + QueryFragment<DB>,
    EqAny<Nullable<C>, BoxedSelectStatement<
        'static,
        <NullableSelect<Select<Filter<<C2::Table as AsQuery>::Query, I::Ret>, C2>> as Query>::SqlType,
        C2::Table,
        DB,
        >>: BoxableFilter<C::Table, DB, SqlType = Bool>,
        <A as BuildFilter<DB>>::Ret: AppearsOnTable<C::Table> + 'static,
{
    type Ret = Box<dyn BoxableFilter<C::Table, DB, SqlType = Bool>>;

    fn into_filter(self) -> Option<Self::Ret> {
        let mut and = AndCollector::default();

        let inner = self
            .inner
            .into_filter()
            .map(|f| <_ as QueryDsl>::filter(C2::Table::table(), f))
            .map(|f| <_ as QueryDsl>::select(f, C2::default()))
            .map(|f| <_ as SelectNullableDsl>::nullable(f).into_boxed())
            .map(|q| Box::new(C::default().nullable().eq_any(q)) as Box<_>);
        and.append_filter(inner);
        and.append_filter(self.additional);

        and.into_filter()
    }
}

impl<C, I, C2, A> Nameable for ReferenceFilter<C, I, C2, A>
where
    I: Nameable,
{
    fn name() -> String {
        I::name()
    }
}

impl<C, I, C2, A> FromInputValue<WundergraphScalarValue> for ReferenceFilter<C, I, C2, A>
where
    I: InnerFilter,
    A: InnerFilter,
{
    fn from_input_value(v: &InputValue<WundergraphScalarValue>) -> Option<Self> {
        let inner = Box::new(I::from_inner_input_value(v.to_object_value()?)?);
        let additional = A::from_inner_input_value(v.to_object_value()?)?;
        Some(Self {
            inner,
            additional,
            p: PhantomData,
        })
    }
}

impl<C, I, C2, A> ToInputValue<WundergraphScalarValue> for ReferenceFilter<C, I, C2, A>
where
    I: InnerFilter,
    A: InnerFilter,
{
    fn to_input_value(&self) -> InputValue<WundergraphScalarValue> {
        let mut map = IndexMap::with_capacity(I::FIELD_COUNT + A::FIELD_COUNT);
        self.inner.to_inner_input_value(&mut map);
        self.additional.to_inner_input_value(&mut map);
        InputValue::object(map)
    }
}

impl<C, I, C2, A> FromLookAheadValue for ReferenceFilter<C, I, C2, A>
where
    I: InnerFilter,
    A: InnerFilter,
{
    fn from_look_ahead(v: &LookAheadValue<'_, WundergraphScalarValue>) -> Option<Self> {
        if let LookAheadValue::Object(ref obj) = *v {
            let inner = Box::new(I::from_inner_look_ahead(obj));
            let additional = A::from_inner_look_ahead(obj);
            Some(Self {
                inner,
                additional,
                p: PhantomData,
            })
        } else {
            None
        }
    }
}

impl<C, I, C2, A> GraphQLType<WundergraphScalarValue> for ReferenceFilter<C, I, C2, A>
where
    I: InnerFilter,
    A: InnerFilter,
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
        let mut fields = I::register_fields(&NameBuilder::default(), registry);
        fields.extend(A::register_fields(&NameBuilder::default(), registry));
        registry
            .build_input_object_type::<Self>(info, &fields)
            .into_meta()
    }
}

impl<C, I, C2, A> InnerFilter for ReferenceFilter<C, I, C2, A>
where
    I: InnerFilter,
    A: InnerFilter,
{
    type Context = I::Context;

    const FIELD_COUNT: usize = I::FIELD_COUNT + A::FIELD_COUNT;

    fn from_inner_input_value(
        obj: IndexMap<&str, &InputValue<WundergraphScalarValue>>,
    ) -> Option<Self> {
        let inner = Box::new(I::from_inner_input_value(obj.clone())?);
        let additional = A::from_inner_input_value(obj)?;
        Some(Self {
            inner,
            additional,
            p: PhantomData,
        })
    }

    fn from_inner_look_ahead(obj: &[(&str, LookAheadValue<'_, WundergraphScalarValue>)]) -> Self {
        let inner = I::from_inner_look_ahead(obj);
        let additional = A::from_inner_look_ahead(obj);
        Self {
            inner: Box::new(inner),
            additional,
            p: PhantomData,
        }
    }

    fn to_inner_input_value(&self, map: &mut IndexMap<&str, InputValue<WundergraphScalarValue>>) {
        self.inner.to_inner_input_value(map);
        self.additional.to_inner_input_value(map);
    }

    fn register_fields<'r>(
        _info: &NameBuilder<Self>,
        registry: &mut Registry<'r, WundergraphScalarValue>,
    ) -> Vec<Argument<'r, WundergraphScalarValue>> {
        let mut fields = I::register_fields(&NameBuilder::default(), registry);
        fields.extend(A::register_fields(&NameBuilder::default(), registry));
        fields
    }
}
