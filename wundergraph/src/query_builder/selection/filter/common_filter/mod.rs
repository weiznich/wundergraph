use super::build_filter::BuildFilter;
use super::collector::{AndCollector, FilterCollector};
use super::filter_value::FilterValue;
use super::inner_filter::InnerFilter;
use crate::diesel_ext::BoxableFilter;
use crate::juniper_ext::{NameBuilder, Nameable, FromLookAheadValue};
use crate::scalar::WundergraphScalarValue;
use diesel::backend::Backend;
use diesel::expression::array_comparison::{In, Many};
use diesel::expression::{operators, AppearsOnTable, AsExpression, NonAggregate};
use diesel::query_builder::QueryFragment;
use diesel::serialize::ToSql;
use diesel::sql_types::{Bool, HasSqlType, SingleValue};
use diesel::Column;
use indexmap::IndexMap;
use juniper::meta::{Argument, MetaType};
use juniper::{FromInputValue, GraphQLType, InputValue, LookAheadValue, Registry, ToInputValue};

mod eq;
mod eq_any;
mod not_eq;

use self::eq::Eq;
use self::eq_any::EqAny;
use self::not_eq::NotEq;

/// This struct summarize all possible filter operations for a given graphql
/// field
///
/// There are two generic parameter
/// * T is a generic type that represents the (rust) type of the field the
///   should be applied to
/// * C is the column type from diesel that matches the field in the database
///
/// Both types must be compatible
#[derive(Debug)]
pub struct FilterOption<T, C>
where
    T: FilterValue<C>,
{
    eq: Eq<T::RawValue, C>,
    neq: NotEq<T::RawValue, C>,
    eq_any: EqAny<T::RawValue, C>,
    // TODO: implement more
    additional: T::AdditionalFilter,
}

impl<T, C> Clone for FilterOption<T, C>
where
    T: Clone + FilterValue<C>,
    T::AdditionalFilter: Clone,
{
    fn clone(&self) -> Self {
        Self {
            eq: self.eq.clone(),
            neq: self.neq.clone(),
            eq_any: self.eq_any.clone(),
            additional: self.additional.clone(),
        }
    }
}

impl<V, C> InnerFilter for FilterOption<V, C>
where
    V: GraphQLType<WundergraphScalarValue, TypeInfo = ()>
        + FromInputValue<WundergraphScalarValue>
        + ToInputValue<WundergraphScalarValue>
        + FromLookAheadValue
        + FilterValue<C>
        + 'static,
    Self: Nameable,
    V::AdditionalFilter: InnerFilter,
{
    type Context = V::Context;

    const FIELD_COUNT: usize = 3 + V::AdditionalFilter::FIELD_COUNT;

    fn from_inner_input_value(
        obj: IndexMap<&str, &InputValue<WundergraphScalarValue>>,
    ) -> Option<Self> {
        let eq = Eq::new(obj.get("eq").map_or_else(
            || Option::from_input_value(&InputValue::Null),
            |v| Option::from_input_value(*v),
        )?);
        let neq = NotEq::new(obj.get("not_eq").map_or_else(
            || Option::from_input_value(&InputValue::Null),
            |v| Option::from_input_value(*v),
        )?);
        let eq_any = EqAny::new(obj.get("eq_any").map_or_else(
            || Option::from_input_value(&InputValue::Null),
            |v| Option::from_input_value(*v),
        )?);
        let additional = V::AdditionalFilter::from_inner_input_value(obj)?;
        Some(Self {
            eq,
            neq,
            eq_any,
            additional,
        })
    }

    fn from_inner_look_ahead(obj: &[(&str, LookAheadValue<'_, WundergraphScalarValue>)]) -> Self {
        let eq = obj
            .iter()
            .find(|o| o.0 == "eq")
            .and_then(|o| V::RawValue::from_look_ahead(&o.1));
        let eq = Eq::new(eq);

        let neq = obj
            .iter()
            .find(|o| o.0 == "not_eq")
            .and_then(|o| V::RawValue::from_look_ahead(&o.1));
        let neq = NotEq::new(neq);

        let eq_any = obj
            .iter()
            .find(|o| o.0 == "eq_any")
            .and_then(|o| Vec::from_look_ahead(&o.1));
        let eq_any = EqAny::new(eq_any);

        let additional = V::AdditionalFilter::from_inner_look_ahead(obj);

        Self {
            eq,
            neq,
            eq_any,
            additional,
        }
    }

    fn to_inner_input_value(&self, map: &mut IndexMap<&str, InputValue<WundergraphScalarValue>>) {
        map.insert("eq", self.eq.to_input_value());
        map.insert("not_eq", self.neq.to_input_value());
        map.insert("eq_any", self.eq_any.to_input_value());
        self.additional.to_inner_input_value(map);
    }

    fn register_fields<'r>(
        _info: &NameBuilder<Self>,
        registry: &mut Registry<'r, WundergraphScalarValue>,
    ) -> Vec<Argument<'r, WundergraphScalarValue>> {
        let eq = registry.arg_with_default::<Option<V>>("eq", &None, &Default::default());
        let neq = registry.arg_with_default::<Option<V>>("not_eq", &None, &Default::default());
        let eq_any =
            registry.arg_with_default::<Option<Vec<V>>>("eq_any", &None, &Default::default());
        let mut ret = vec![eq, neq, eq_any];
        let additional = V::AdditionalFilter::register_fields(&NameBuilder::default(), registry);
        ret.extend(additional);
        ret
    }
}

impl<T, C> FromInputValue<WundergraphScalarValue> for FilterOption<T, C>
where
    T: FilterValue<C>,
    Self: InnerFilter,
{
    fn from_input_value(v: &InputValue<WundergraphScalarValue>) -> Option<Self> {
        if let Some(obj) = v.to_object_value() {
            <Self as InnerFilter>::from_inner_input_value(obj)
        } else {
            None
        }
    }
}

impl<T, C> ToInputValue<WundergraphScalarValue> for FilterOption<T, C>
where
    T: FilterValue<C>,
    Self: InnerFilter,
{
    fn to_input_value(&self) -> InputValue<WundergraphScalarValue> {
        let mut map = IndexMap::with_capacity(3);
        self.to_inner_input_value(&mut map);
        InputValue::object(map)
    }
}

impl<T, C> GraphQLType<WundergraphScalarValue> for FilterOption<T, C>
where
    T: FilterValue<C>,
    T: GraphQLType<WundergraphScalarValue>,
    Self: InnerFilter<Context = T::Context> + Nameable,
{
    type Context = T::Context;
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
        let fields = Self::register_fields(info, registry);
        registry
            .build_input_object_type::<Self>(info, &fields)
            .into_meta()
    }
}

impl<V, C> Nameable for FilterOption<V, C>
where
    V: Nameable + FilterValue<C>,
{
    fn name() -> String {
        format!("Filter_{}_", V::name())
    }
}

impl<T, C> FromLookAheadValue for FilterOption<T, C>
where
    T: FromLookAheadValue + FilterValue<C>,
    C: Column,
    Self: InnerFilter,
{
    fn from_look_ahead(a: &LookAheadValue<'_, WundergraphScalarValue>) -> Option<Self> {
        if let LookAheadValue::Object(ref obj) = *a {
            Some(Self::from_inner_look_ahead(obj))
        } else {
            None
        }
    }
}

impl<T, C, DB> BuildFilter<DB> for FilterOption<T, C>
where
    DB: Backend + HasSqlType<C::SqlType> + 'static,
    T: FilterValue<C>,
    T::AdditionalFilter: BuildFilter<DB> + 'static,
    <T::AdditionalFilter as BuildFilter<DB>>::Ret:
        AppearsOnTable<C::Table> + QueryFragment<DB> + 'static,
    T::RawValue: AsExpression<C::SqlType> + ToSql<C::SqlType, DB> + 'static,
    <T::RawValue as AsExpression<C::SqlType>>::Expression:
        NonAggregate + AppearsOnTable<C::Table> + QueryFragment<DB> + 'static,
    C: Column + NonAggregate + QueryFragment<DB> + Default + 'static,
    C::SqlType: SingleValue,
    C::Table: 'static,
    operators::Eq<C, <T::RawValue as AsExpression<C::SqlType>>::Expression>:
        AppearsOnTable<C::Table, SqlType = Bool>,
    operators::NotEq<C, <T::RawValue as AsExpression<C::SqlType>>::Expression>:
        AppearsOnTable<C::Table, SqlType = Bool>,
    In<C, Many<<T::RawValue as AsExpression<C::SqlType>>::Expression>>:
        AppearsOnTable<C::Table, SqlType = Bool>,
{
    type Ret = Box<dyn BoxableFilter<C::Table, DB, SqlType = Bool>>;

    fn into_filter(self) -> Option<Self::Ret> {
        let mut combinator = AndCollector::default();
        combinator.append_filter(self.eq);
        combinator.append_filter(self.neq);
        combinator.append_filter(self.eq_any);
        combinator.append_filter(self.additional);
        combinator.into_filter()
    }
}
