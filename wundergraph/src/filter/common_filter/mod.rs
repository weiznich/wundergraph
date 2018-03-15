use filter::inner_filter::InnerFilter;
use filter::filter_value::FilterValue;
use filter::build_filter::BuildFilter;
use filter::transformator::Transformator;
use filter::collector::{AndCollector, FilterCollector};

use diesel::expression::{operators, AsExpression, NonAggregate, SelectableExpression};
use diesel::expression::array_comparison::{In, Many};
use diesel::backend::Backend;
use diesel::{BoxableExpression, Column};
use diesel::sql_types::{Bool, SingleValue};
use diesel::query_builder::QueryFragment;

use juniper::{FromInputValue, GraphQLType, InputValue, LookAheadValue, Registry, ToInputValue};
use juniper::meta::{Argument, MetaType};

use ordermap::OrderMap;
use helper::{FromLookAheadValue, NameBuilder, Nameable};

mod eq;
mod eq_any;
mod not_eq;

use self::eq::Eq;
use self::eq_any::EqAny;
use self::not_eq::NotEq;

#[derive(Debug)]
pub struct FilterOption<T, C, DB>
where
    T: FilterValue<C, DB>,
{
    eq: Eq<T::RawValue, C, DB>,
    neq: NotEq<T::RawValue, C, DB>,
    eq_any: EqAny<T::RawValue, C, DB>,
    // TODO: implement more
    additional: T::AdditionalFilter,
}

impl<T, C, DB> Clone for FilterOption<T, C, DB>
where
    T: Clone + FilterValue<C, DB>,
    T::AdditionalFilter: Clone,
{
    fn clone(&self) -> Self {
        FilterOption {
            eq: self.eq.clone(),
            neq: self.neq.clone(),
            eq_any: self.eq_any.clone(),
            additional: self.additional.clone(),
        }
    }
}

impl<V, C, DB> InnerFilter for FilterOption<V, C, DB>
where
    V: GraphQLType<TypeInfo = ()>
        + FromInputValue
        + ToInputValue
        + FromLookAheadValue
        + FilterValue<C, DB>
        + 'static,
    Self: Nameable,
    DB: Backend + 'static,
    V::RawValue: AsExpression<C::SqlType>,
    V::AdditionalFilter: InnerFilter,
    <V::AdditionalFilter as BuildFilter>::Ret: SelectableExpression<C::Table> + QueryFragment<DB>,
    <V::RawValue as AsExpression<C::SqlType>>::Expression: NonAggregate
        + SelectableExpression<C::Table>
        + QueryFragment<DB>,
    C: QueryFragment<DB> + NonAggregate + Column + Default + 'static,
    C::SqlType: SingleValue,
    C::Table: 'static,
    Self: BuildFilter,
{
    type Context = V::Context;

    const FIELD_COUNT: usize = 3 + V::AdditionalFilter::FIELD_COUNT;

    fn from_inner_input_value(obj: OrderMap<&str, &InputValue>) -> Option<Self> {
        let eq = obj.get("eq")
            .map(|v| Option::from_input_value(*v))
            .unwrap_or_else(|| Option::from_input_value(&InputValue::Null));
        let eq = match eq {
            Some(eq) => Eq::new(eq),
            None => return None,
        };
        let neq = obj.get("not_eq")
            .map(|v| Option::from_input_value(*v))
            .unwrap_or_else(|| Option::from_input_value(&InputValue::Null));
        let neq = match neq {
            Some(neq) => NotEq::new(neq),
            None => return None,
        };
        let eq_any = obj.get("eq_any")
            .map(|v| Option::from_input_value(*v))
            .unwrap_or_else(|| Option::from_input_value(&InputValue::Null));
        let eq_any = match eq_any {
            Some(eq_any) => EqAny::new(eq_any),
            None => return None,
        };
        let additional = match V::AdditionalFilter::from_inner_input_value(obj) {
            Some(a) => a,
            None => return None,
        };
        Some(Self {
            eq,
            neq,
            eq_any,
            additional,
        })
    }

    fn from_inner_look_ahead(obj: &[(&str, LookAheadValue)]) -> Self {
        let eq = obj.iter()
            .find(|o| o.0 == "eq")
            .and_then(|o| V::RawValue::from_look_ahead(&o.1));
        let eq = Eq::new(eq);

        let neq = obj.iter()
            .find(|o| o.0 == "not_eq")
            .and_then(|o| V::RawValue::from_look_ahead(&o.1));
        let neq = NotEq::new(neq);

        let eq_any = obj.iter()
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

    fn to_inner_input_value(&self, map: &mut OrderMap<&str, InputValue>) {
        map.insert("eq", self.eq.to_input_value());
        map.insert("not_eq", self.neq.to_input_value());
        map.insert("eq_any", self.eq_any.to_input_value());
        self.additional.to_inner_input_value(map);
    }

    fn register_fields<'r>(
        _info: &NameBuilder<Self>,
        registry: &mut Registry<'r>,
    ) -> Vec<Argument<'r>> {
        let eq = registry.arg_with_default::<Option<V>>("eq", &None, &Default::default());
        let neq = registry.arg_with_default::<Option<V>>("not_eq", &None, &Default::default());
        let eq_any =
            registry.arg_with_default::<Option<Vec<V>>>("eq_any", &None, &Default::default());
        let mut ret = vec![eq, neq, eq_any];
        let additional = V::AdditionalFilter::register_fields(&Default::default(), registry);
        ret.extend(additional);
        ret
    }
}

impl<T, C, DB> FromInputValue for FilterOption<T, C, DB>
where
    T: FilterValue<C, DB>,
    Self: InnerFilter,
{
    fn from_input_value(v: &InputValue) -> Option<Self> {
        if let Some(obj) = v.to_object_value() {
            <Self as InnerFilter>::from_inner_input_value(obj)
        } else {
            None
        }
    }
}

impl<T, C, DB> ToInputValue for FilterOption<T, C, DB>
where
    T: FilterValue<C, DB>,
    Self: InnerFilter,
{
    fn to_input_value(&self) -> InputValue {
        let mut map = OrderMap::with_capacity(3);
        self.to_inner_input_value(&mut map);
        InputValue::object(map)
    }
}

impl<T, C, DB> GraphQLType for FilterOption<T, C, DB>
where
    T: FilterValue<C, DB>,
    T: GraphQLType,
    Self: InnerFilter<Context = T::Context> + Nameable,
{
    type Context = T::Context;
    type TypeInfo = NameBuilder<Self>;

    fn name(info: &Self::TypeInfo) -> Option<&str> {
        Some(info.name())
    }

    fn meta<'r>(info: &Self::TypeInfo, registry: &mut Registry<'r>) -> MetaType<'r> {
        let fields = Self::register_fields(info, registry);
        registry
            .build_input_object_type::<Self>(info, &fields)
            .into_meta()
    }
}

impl<V, C, DB> Nameable for FilterOption<V, C, DB>
where
    V: Nameable + FilterValue<C, DB>,
{
    fn name() -> String {
        format!("Filter_{}_", V::name())
    }
}

impl<T, C, DB> FromLookAheadValue for FilterOption<T, C, DB>
where
    T: FromLookAheadValue + FilterValue<C, DB>,
    C: Column,
    Self: InnerFilter,
{
    fn from_look_ahead(a: &LookAheadValue) -> Option<Self> {
        if let LookAheadValue::Object(ref obj) = *a {
            Some(Self::from_inner_look_ahead(obj))
        } else {
            None
        }
    }
}

impl<T, C, DB> BuildFilter for FilterOption<T, C, DB>
where
    DB: Backend + 'static,
    T: FilterValue<C, DB>,
    T::AdditionalFilter: BuildFilter + 'static,
    <T::AdditionalFilter as BuildFilter>::Ret: SelectableExpression<C::Table>
        + QueryFragment<DB>
        + 'static,
    T::RawValue: AsExpression<C::SqlType> + 'static,
    <T::RawValue as AsExpression<C::SqlType>>::Expression: NonAggregate
        + SelectableExpression<C::Table>
        + QueryFragment<DB>
        + 'static,
    C: Column + NonAggregate + QueryFragment<DB> + Default + 'static,
    C::SqlType: SingleValue,
    C::Table: 'static,
    operators::Eq<C, <T::RawValue as AsExpression<C::SqlType>>::Expression>: SelectableExpression<C::Table, SqlType = Bool>,
    operators::NotEq<C, <T::RawValue as AsExpression<C::SqlType>>::Expression>: SelectableExpression<C::Table, SqlType = Bool>,
    In<C, Many<<T::RawValue as AsExpression<C::SqlType>>::Expression>>: SelectableExpression<C::Table, SqlType = Bool>,
{
    type Ret = Box<BoxableExpression<C::Table, DB, SqlType = Bool>>;

    fn into_filter<F>(self, t: F) -> Option<Self::Ret>
    where
        F: Transformator,
    {
        let mut combinator = AndCollector::default();
        combinator.append_filter(self.eq, t);
        combinator.append_filter(self.neq, t);
        combinator.append_filter(self.eq_any, t);
        combinator.append_filter(self.additional, t);
        combinator.into_filter(t)
    }
}
