#![allow(warnings)]
use crate::diesel_ext::BoxableFilter;
use crate::juniper_ext::{FromLookAheadValue, NameBuilder, Nameable};
use crate::query_builder::selection::filter::build_filter::BuildFilter;
use crate::query_builder::selection::filter::collector::{AndCollector, FilterCollector};
use crate::query_builder::selection::filter::inner_filter::InnerFilter;
use crate::scalar::WundergraphScalarValue;
use diesel::backend::Backend;
use diesel::dsl::{EqAny, Filter, NullableSelect, Select, SqlTypeOf};
use diesel::expression::array_comparison::AsInExpression;
use diesel::expression::nullable::Nullable;
use diesel::expression::NonAggregate;
use diesel::query_builder::{AsQuery, BoxedSelectStatement, Query, QueryFragment};
use diesel::query_dsl::methods::{BoxedDsl, FilterDsl, SelectDsl, SelectNullableDsl};
use diesel::sql_types::{Bool, SingleValue};
use diesel::{associations::HasTable, Expression};
use diesel::{AppearsOnTable, Column, ExpressionMethods, NullableExpressionMethods, QueryDsl};
use indexmap::IndexMap;
use juniper::meta::{Argument, MetaType};
use juniper::{FromInputValue, GraphQLType, InputValue, LookAheadValue, Registry, ToInputValue};
use std::marker::PhantomData;

/// A filter node representing a filter over a referenced entity
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

pub trait AsReferenceFilterExpression<C2, DB: Backend, F> {
    type Expr: BoxableFilter<Self::Table, DB, SqlType = Bool>;
    type Table: 'static;

    fn as_filter(f: F) -> Self::Expr;
}

impl<C1, C2, DB, F> AsReferenceFilterExpression<C2, DB, F> for C1
where
    DB: Backend + 'static,
    C1: Column + Default + 'static,
    C2: Column + Default,
    C1::Table: 'static,
    C2::Table: 'static,
    C1::Table: HasTable<Table = C1::Table>,
    C2::Table: HasTable<Table = C2::Table>,
    Nullable<C1>: ExpressionMethods,
    <Nullable<C1> as Expression>::SqlType: 'static,
    <C2::Table as AsQuery>::Query: FilterDsl<F>,
    F: Expression<SqlType = Bool> + NonAggregate + QueryFragment<DB>,
    Filter<<C2::Table as AsQuery>::Query, F>: QueryDsl + SelectDsl<C2>,
    Select<Filter<<C2::Table as AsQuery>::Query, F>, C2>: QueryDsl + SelectNullableDsl,
    NullableSelect<Select<Filter<<C2::Table as AsQuery>::Query, F>, C2>>: QueryDsl + Query
        + BoxedDsl<
            'static,
        DB,
        Output = BoxedSelectStatement<
                'static,
            <NullableSelect<Select<Filter<<C2::Table as AsQuery>::Query, F>, C2>> as Query>::SqlType,
            C2::Table,
            DB,
            >,
    > + 'static,
        EqAny<Nullable<C1>, BoxedSelectStatement<
            'static,
            <NullableSelect<Select<Filter<<C2::Table as AsQuery>::Query, F>, C2>> as Query>::SqlType,
            C2::Table,
            DB,
            >>: BoxableFilter<C1::Table, DB, SqlType = Bool>,
        BoxedSelectStatement<
            'static,
            <NullableSelect<Select<Filter<<C2::Table as AsQuery>::Query, F>, C2>> as Query>::SqlType,
            C2::Table,
            DB,
        >: AsInExpression<SqlTypeOf<Nullable<C1>>>,
{
    type Expr = Box<dyn BoxableFilter<Self::Table, DB, SqlType = Bool>>;

    type Table = C1::Table;

    fn as_filter(f: F) -> Self::Expr {
        let f = <_ as QueryDsl>::filter(C2::Table::table(), f);
        let f = <_ as QueryDsl>::select(f, C2::default());
        let q = <_ as SelectNullableDsl>::nullable(f).into_boxed();
        Box::new(C1::default().nullable().eq_any(q)) as Box<_>
    }
}

impl<C, DB, I, C2, A> BuildFilter<DB> for ReferenceFilter<C, I, C2, A>
where
    C: AsReferenceFilterExpression<C2, DB, I::Ret>,
    C::Expr: BuildFilter<DB> + 'static,
    <C::Expr as BuildFilter<DB>>::Ret: AppearsOnTable<C::Table>,
    DB: Backend + 'static,
    I: BuildFilter<DB> + InnerFilter,
    A: BuildFilter<DB> + 'static,
    <A as BuildFilter<DB>>::Ret: AppearsOnTable<C::Table> + 'static,
{
    type Ret = Box<dyn BoxableFilter<C::Table, DB, SqlType = Bool>>;

    fn into_filter(self) -> Option<Self::Ret> {
        let mut and = AndCollector::default();
        dbg!(std::any::type_name::<I>());

        let inner = self.inner.into_filter();

        if inner.is_some() {
            dbg!("SOME")
        } else {
            dbg!("None")
        };

        let inner = inner.map(|f| C::as_filter(f));
        and.append_filter(inner);
        and.append_filter(self.additional);

        and.into_filter()
    }
}

impl<C, I, C2, A> Nameable for ReferenceFilter<C, I, C2, A>
where
    I: Nameable,
    A: Nameable,
{
    fn name() -> String {
        let a = A::name();
        let i = I::name();

        if a.is_empty() {
            i
        } else {
            format!("{}_{}", a, i)
        }
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
