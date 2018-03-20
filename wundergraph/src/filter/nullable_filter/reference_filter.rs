use filter::collector::{AndCollector, FilterCollector};
use filter::inner_filter::InnerFilter;
use filter::build_filter::BuildFilter;
use filter::transformator::{OnlyExclusive, OnlySelective, Transformator};

use diesel::{BoolExpressionMethods, BoxableExpression, Column, ExpressionMethods,
             NullableExpressionMethods, QueryDsl, SelectableExpression};
use diesel::query_dsl::methods::{BoxedDsl, FilterDsl, SelectDsl};
use diesel::query_builder::AsQuery;
use diesel::sql_types::{Bool, NotNull, SingleValue};
use diesel::backend::Backend;
use diesel::expression::NonAggregate;
use diesel::associations::HasTable;
use diesel::query_builder::QueryFragment;
use diesel::expression::array_comparison::AsInExpression;
use diesel::expression::nullable::Nullable;
use diesel::dsl::{self, EqAny, Filter, IntoBoxed, NeAny, Or, Select};

use juniper::{FromInputValue, GraphQLType, InputValue, LookAheadValue, Registry, ToInputValue};
use juniper::meta::{Argument, MetaType};

use ordermap::OrderMap;

use helper::{FromLookAheadValue, NameBuilder, Nameable};

use super::IsNull;

#[derive(Debug)]
pub struct NullableReferenceFilter<C, I, C2> {
    is_null: Option<IsNull<C>>,
    inner: Box<I>,
    p: ::std::marker::PhantomData<C2>,
}

impl<C, I, C2> Clone for NullableReferenceFilter<C, I, C2>
where
    I: Clone,
{
    fn clone(&self) -> Self {
        Self {
            is_null: self.is_null.clone(),
            inner: self.inner.clone(),
            p: Default::default(),
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
    Filter<<C2::Table as AsQuery>::Query, I::Ret>: QueryDsl + SelectDsl<Nullable<C2>>,
    Select<Filter<<C2::Table as AsQuery>::Query, I::Ret>, Nullable<C2>>: QueryDsl
        + BoxedDsl<'static, DB>
        + 'static,
    IntoBoxed<'static, Select<Filter<<C2::Table as AsQuery>::Query, I::Ret>, Nullable<C2>>, DB>: AsInExpression<C::SqlType>,
    <IntoBoxed<'static, Select<Filter<<C2::Table as AsQuery>::Query, I::Ret>, Nullable<C2>>, DB> as AsInExpression<C::SqlType>>::InExpression: SelectableExpression<C::Table> + QueryFragment<DB>,
       IsNull<C>: BuildFilter<DB>,
<IsNull<C> as BuildFilter<DB>>::Ret: SelectableExpression<C::Table> + QueryFragment<DB>,
EqAny<C,
      IntoBoxed<'static, Select<Filter<<C2::Table as AsQuery>::Query, I::Ret>, Nullable<C2>>, DB>>: SelectableExpression<C::Table, SqlType = Bool>,
Or<NeAny<C,
      IntoBoxed<'static, Select<Filter<<C2::Table as AsQuery>::Query, I::Ret>, Nullable<C2>>, DB>>, dsl::IsNull<C>>: SelectableExpression<C::Table, SqlType = Bool>,
{
    type Ret = Box<BoxableExpression<C::Table, DB, SqlType = Bool>>;

    fn into_filter<F>(self, t: F) -> Option<Self::Ret>
    where
        F: Transformator,
    {
        let mut and = AndCollector::default();
        and.append_filter(self.is_null, t);

        let selective_inner = self.inner
            .clone()
            .into_filter(OnlySelective)
            .map(|f| <_ as QueryDsl>::filter(C2::Table::table(), f))
            .map(|f| <_ as QueryDsl>::select(f, C2::default().nullable()))
            .map(|f| f.into_boxed())
            .map(|q| Box::new(C::default().eq_any(q)) as Box<_>);
        and.append_filter(selective_inner, t);

        let exclusive_inner = self.inner
            .clone()
            .into_filter(OnlyExclusive)
            .map(|f| <_ as QueryDsl>::filter(C2::Table::table(), f))
            .map(|f| <_ as QueryDsl>::select(f, C2::default().nullable()))
            .map(|f| f.into_boxed())
            .map(|q| Box::new(C::default().ne_all(q).or(C::default().is_null())) as Box<_>);
        and.append_filter(exclusive_inner, t);

        and.into_filter(t)
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

impl<C, I, C2> FromInputValue for NullableReferenceFilter<C, I, C2>
where
    I: InnerFilter,
{
    fn from_input_value(v: &InputValue) -> Option<Self> {
        if let Some(obj) = v.to_object_value() {
            let is_null = obj.get("is_null")
                .map(|v| Option::from_input_value(*v))
                .unwrap_or_else(|| Option::from_input_value(&InputValue::Null));
            let is_null = match is_null {
                Some(Some(v)) => Some(IsNull::new(v)),
                Some(None) => None,
                None => return None,
            };
            I::from_inner_input_value(obj).map(|inner| Self {
                inner: Box::new(inner),
                is_null,
                p: Default::default(),
            })
        } else {
            None
        }
    }
}

impl<C, I, C2> ToInputValue for NullableReferenceFilter<C, I, C2>
where
    I: InnerFilter,
{
    fn to_input_value(&self) -> InputValue {
        let mut map = OrderMap::with_capacity(I::FIELD_COUNT + 1);
        self.inner.to_inner_input_value(&mut map);
        map.insert("is_null", self.is_null.as_ref().to_input_value());
        InputValue::object(map)
    }
}

impl<C, I, C2> FromLookAheadValue for NullableReferenceFilter<C, I, C2>
where
    I: InnerFilter,
{
    fn from_look_ahead(v: &LookAheadValue) -> Option<Self> {
        if let LookAheadValue::Object(ref obj) = *v {
            let is_null = obj.iter()
                .find(|o| o.0 == "is_null")
                .and_then(|o| bool::from_look_ahead(&o.1))
                .map(IsNull::new);
            let inner = I::from_inner_look_ahead(obj);
            Some(Self {
                inner: Box::new(inner),
                is_null,
                p: Default::default(),
            })
        } else {
            None
        }
    }
}

impl<C, I, C2> GraphQLType for NullableReferenceFilter<C, I, C2>
where
    I: InnerFilter,
{
    type Context = I::Context;
    type TypeInfo = NameBuilder<Self>;

    fn name(info: &Self::TypeInfo) -> Option<&str> {
        Some(info.name())
    }

    fn meta<'r>(info: &Self::TypeInfo, registry: &mut Registry<'r>) -> MetaType<'r> {
        let mut fields = I::register_fields(&Default::default(), registry);
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

    fn from_inner_input_value(obj: OrderMap<&str, &InputValue>) -> Option<Self> {
        let is_null = obj.get("is_null")
            .map(|v| Option::from_input_value(*v))
            .unwrap_or_else(|| Option::from_input_value(&InputValue::Null));
        let is_null = match is_null {
            Some(Some(v)) => Some(IsNull::new(v)),
            Some(None) => None,
            None => return None,
        };
        let inner = match I::from_inner_input_value(obj) {
            Some(inner) => Box::new(inner),
            None => return None,
        };
        Some(Self {
            is_null,
            inner,
            p: Default::default(),
        })
    }
    fn from_inner_look_ahead(obj: &[(&str, LookAheadValue)]) -> Self {
        let inner = I::from_inner_look_ahead(obj);
        let is_null = obj.iter()
            .find(|o| o.0 == "is_null")
            .and_then(|o| bool::from_look_ahead(&o.1))
            .map(IsNull::new);
        Self {
            inner: Box::new(inner),
            is_null,
            p: Default::default(),
        }
    }
    fn to_inner_input_value(&self, map: &mut OrderMap<&str, InputValue>) {
        self.inner.to_inner_input_value(map);
        map.insert("is_null", self.is_null.as_ref().to_input_value());
    }
    fn register_fields<'r>(
        _info: &NameBuilder<Self>,
        registry: &mut Registry<'r>,
    ) -> Vec<Argument<'r>> {
        let mut inner_fields = I::register_fields(&Default::default(), registry);
        let is_null = registry.arg_with_default::<Option<bool>>("is_null", &None, &());
        inner_fields.push(is_null);
        inner_fields
    }
}
