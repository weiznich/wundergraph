use std::marker::PhantomData;

use filter::build_filter::BuildFilter;
use filter::collector::{AndCollector, FilterCollector};
use filter::inner_filter::InnerFilter;

use diesel;
use diesel::associations::HasTable;
use diesel::backend::Backend;
use diesel::dsl::{self, EqAny, Filter, NeAny, Select, SqlTypeOf};
use diesel::expression::array_comparison::AsInExpression;
use diesel::expression::NonAggregate;
use diesel::query_builder::AsQuery;
use diesel::query_builder::{BoxedSelectStatement, QueryFragment};
use diesel::query_dsl::methods::{BoxedDsl, FilterDsl, SelectDsl};
use diesel::sql_types::{Bool, NotNull, SingleValue};
use diesel::{AppearsOnTable, Column, ExpressionMethods, QueryDsl};
use diesel_ext::BoxableFilter;

use juniper::meta::{Argument, MetaType};
use juniper::{FromInputValue, GraphQLType, InputValue, LookAheadValue, Registry, ToInputValue};

use indexmap::IndexMap;

use helper::{FromLookAheadValue, NameBuilder, Nameable};
use scalar::WundergraphScalarValue;

use super::IsNull;

#[derive(Debug)]
pub struct NullableReferenceFilter<C, I, C2> {
    is_null: Option<IsNull<C>>,
    inner: Box<I>,
    p: PhantomData<C2>,
}

impl<C, I, C2> Clone for NullableReferenceFilter<C, I, C2>
where
    I: Clone,
{
    fn clone(&self) -> Self {
        Self {
            is_null: self.is_null.clone(),
            inner: self.inner.clone(),
            p: PhantomData,
        }
    }
}

impl<C, DB, I, C2> BuildFilter<DB> for NullableReferenceFilter<C, I, C2>
where
    C: Column + NonAggregate + QueryFragment<DB> + Default + 'static,
    C::Table: 'static,
    C::SqlType: SingleValue,
    DB: Backend + 'static,
    I: BuildFilter<DB> + Clone + InnerFilter,
    C2::Table: HasTable<Table = C2::Table>,
    C2: Column + NonAggregate + QueryFragment<DB> + Default + 'static,
    C2::SqlType: NotNull,
    <C2::Table as AsQuery>::Query: FilterDsl<I::Ret>,
    Filter<<C2::Table as AsQuery>::Query, I::Ret>: QueryDsl + SelectDsl<C2>,
    Select<Filter<<C2::Table as AsQuery>::Query, I::Ret>, C2>: QueryDsl
        + BoxedDsl<
            'static,
            DB,
            Output = BoxedSelectStatement<'static, SqlTypeOf<C2>, C2::Table, DB>,
        >
        + 'static,
    dsl::IsNull<C>: AppearsOnTable<C::Table, SqlType = Bool>,
    dsl::IsNotNull<C>: AppearsOnTable<C::Table, SqlType = Bool>,
BoxedSelectStatement<'static, diesel::sql_types::Nullable<SqlTypeOf<C2>>, C2::Table, DB>:
        AsInExpression<SqlTypeOf<C>>,
    <BoxedSelectStatement<'static, diesel::sql_types::Nullable<SqlTypeOf<C2>>, C2::Table, DB> as AsInExpression<
        SqlTypeOf<C>,
    >>::InExpression: AppearsOnTable<C::Table> + QueryFragment<DB>,
    EqAny<C, BoxedSelectStatement<'static, diesel::sql_types::Nullable<SqlTypeOf<C2>>, C2::Table, DB>>:
AppearsOnTable<C::Table, SqlType = Bool>,
    NeAny<C, BoxedSelectStatement<'static, diesel::sql_types::Nullable<SqlTypeOf<C2>>, C2::Table, DB>>:
        AppearsOnTable<C::Table, SqlType = Bool>,
{
    type Ret = Box<BoxableFilter<C::Table, DB, SqlType = Bool>>;

    fn into_filter(self) -> Option<Self::Ret>
    {
        let mut and = AndCollector::default();
        and.append_filter(self.is_null);

        let inner = self.inner
            .into_filter()
            .map(|f| <_ as QueryDsl>::filter(C2::Table::table(), f))
            .map(|f| <_ as QueryDsl>::select(f, C2::default()))
            .map(|f| f.into_boxed().nullable())
            .map(|q| Box::new(C::default().eq_any(q)) as Box<_>);
        and.append_filter(inner);
        and.into_filter()
    }
}

impl<C, I, C2> Nameable for NullableReferenceFilter<C, I, C2>
where
    I: Nameable,
{
    fn name() -> String {
        format!("NullableReference_{}_", I::name())
    }
}

impl<C, I, C2> FromInputValue<WundergraphScalarValue> for NullableReferenceFilter<C, I, C2>
where
    I: InnerFilter,
{
    fn from_input_value(v: &InputValue<WundergraphScalarValue>) -> Option<Self> {
        if let Some(obj) = v.to_object_value() {
            let is_null = obj
                .get("is_null")
                .map_or_else(
                    || {
                        let v: &InputValue<WundergraphScalarValue> = &InputValue::Null;
                        Option::from_input_value(v)
                    },
                    |v| Option::from_input_value(*v),
                )?
                .map(IsNull::new);
            I::from_inner_input_value(obj).map(|inner| Self {
                inner: Box::new(inner),
                is_null,
                p: PhantomData,
            })
        } else {
            None
        }
    }
}

impl<C, I, C2> ToInputValue<WundergraphScalarValue> for NullableReferenceFilter<C, I, C2>
where
    I: InnerFilter,
{
    fn to_input_value(&self) -> InputValue<WundergraphScalarValue> {
        let mut map = IndexMap::with_capacity(I::FIELD_COUNT + 1);
        self.inner.to_inner_input_value(&mut map);
        map.insert("is_null", self.is_null.as_ref().to_input_value());
        InputValue::object(map)
    }
}

impl<C, I, C2> FromLookAheadValue for NullableReferenceFilter<C, I, C2>
where
    I: InnerFilter,
{
    fn from_look_ahead(v: &LookAheadValue<WundergraphScalarValue>) -> Option<Self> {
        if let LookAheadValue::Object(ref obj) = *v {
            let is_null = obj
                .iter()
                .find(|o| o.0 == "is_null")
                .and_then(|o| bool::from_look_ahead(&o.1))
                .map(IsNull::new);
            let inner = I::from_inner_look_ahead(obj);
            Some(Self {
                inner: Box::new(inner),
                is_null,
                p: PhantomData,
            })
        } else {
            None
        }
    }
}

impl<C, I, C2> GraphQLType<WundergraphScalarValue> for NullableReferenceFilter<C, I, C2>
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
        let mut fields = I::register_fields(&NameBuilder::default(), registry);
        let is_null = registry.arg_with_default::<Option<bool>>("is_null", &None, &());
        fields.push(is_null);
        registry
            .build_input_object_type::<Self>(info, &fields)
            .into_meta()
    }
}

impl<C, I, C2> InnerFilter for NullableReferenceFilter<C, I, C2>
where
    I: InnerFilter,
{
    type Context = I::Context;

    const FIELD_COUNT: usize = I::FIELD_COUNT + 1;

    fn from_inner_input_value(
        obj: IndexMap<&str, &InputValue<WundergraphScalarValue>>,
    ) -> Option<Self> {
        let is_null = obj
            .get("is_null")
            .map_or_else(
                || {
                    let v: &InputValue<WundergraphScalarValue> = &InputValue::Null;
                    Option::from_input_value(v)
                },
                |v| Option::from_input_value(*v),
            )?
            .map(IsNull::new);
        let inner = match I::from_inner_input_value(obj) {
            Some(inner) => Box::new(inner),
            None => return None,
        };
        Some(Self {
            is_null,
            inner,
            p: PhantomData,
        })
    }
    fn from_inner_look_ahead(obj: &[(&str, LookAheadValue<WundergraphScalarValue>)]) -> Self {
        let inner = I::from_inner_look_ahead(obj);
        let is_null = obj
            .iter()
            .find(|o| o.0 == "is_null")
            .and_then(|o| bool::from_look_ahead(&o.1))
            .map(IsNull::new);
        Self {
            inner: Box::new(inner),
            is_null,
            p: PhantomData,
        }
    }
    fn to_inner_input_value(&self, map: &mut IndexMap<&str, InputValue<WundergraphScalarValue>>) {
        self.inner.to_inner_input_value(map);
        map.insert("is_null", self.is_null.as_ref().to_input_value());
    }
    fn register_fields<'r>(
        _info: &NameBuilder<Self>,
        registry: &mut Registry<'r, WundergraphScalarValue>,
    ) -> Vec<Argument<'r, WundergraphScalarValue>> {
        let mut inner_fields = I::register_fields(&NameBuilder::default(), registry);
        let is_null = registry.arg_with_default::<Option<bool>>("is_null", &None, &());
        inner_fields.push(is_null);
        inner_fields
    }
}
