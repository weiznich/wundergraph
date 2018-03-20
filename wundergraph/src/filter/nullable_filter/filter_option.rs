use filter::build_filter::BuildFilter;
use filter::filter_value::FilterValue;
use filter::inner_filter::InnerFilter;
use filter::transformator::Transformator;
use filter::collector::{AndCollector, FilterCollector};

use diesel::{BoxableExpression, Column, SelectableExpression};
use diesel::sql_types::{Bool, SingleValue};
use diesel::backend::Backend;
use diesel::expression::{AsExpression, NonAggregate};
use diesel::query_builder::QueryFragment;

use juniper::{FromInputValue, InputValue, LookAheadValue, Registry};
use juniper::meta::Argument;

use ordermap::OrderMap;

use helper::{FromLookAheadValue, NameBuilder, Nameable};

use super::IsNull;

#[derive(Debug)]
pub struct NullableFilter<V, C>
where
    V: FilterValue<C>,
{
    is_null: Option<IsNull<C>>,
    additional: V::AdditionalFilter,
}

impl<V, C> Clone for NullableFilter<V, C>
where
    V: FilterValue<C>,
    V::AdditionalFilter: Clone,
{
    fn clone(&self) -> Self {
        NullableFilter {
            is_null: self.is_null.clone(),
            additional: self.additional.clone(),
        }
    }
}

impl<V, C, DB> BuildFilter<DB> for NullableFilter<V, C>
where
    C: Column + NonAggregate + QueryFragment<DB> + Default + 'static,
    C::SqlType: SingleValue,
    C::Table: 'static,
    DB: Backend + 'static,
    V: FilterValue<C> + 'static,
    V::AdditionalFilter: BuildFilter<DB>,
    <V::AdditionalFilter as BuildFilter<DB>>::Ret: SelectableExpression<C::Table>
        + QueryFragment<DB>,
    V::RawValue: AsExpression<C::SqlType> + 'static,
    <V::RawValue as AsExpression<C::SqlType>>::Expression: SelectableExpression<C::Table>
        + NonAggregate
        + QueryFragment<DB>
        + 'static,
    IsNull<C>: BuildFilter<DB>,
    <IsNull<C> as BuildFilter<DB>>::Ret: SelectableExpression<C::Table> + QueryFragment<DB>,
{
    type Ret = Box<BoxableExpression<C::Table, DB, SqlType = Bool>>;

    fn into_filter<F>(self, t: F) -> Option<Self::Ret>
    where
        F: Transformator,
    {
        let mut combinator = AndCollector::default();
        combinator.append_filter(self.is_null, t);
        combinator.append_filter(self.additional, t);
        combinator.into_filter(t)
    }
}

impl<V, C> Nameable for NullableFilter<V, C>
where
    V: Nameable + FilterValue<C>,
{
    fn name() -> String {
        format!("NullableFilter_{}_", V::name())
    }
}

impl<V, C> InnerFilter for NullableFilter<V, C>
where
    V: FilterValue<C> + Nameable,
    V::AdditionalFilter: InnerFilter,
{
    type Context = ();

    const FIELD_COUNT: usize = 1 + V::AdditionalFilter::FIELD_COUNT;
    fn from_inner_input_value(obj: OrderMap<&str, &InputValue>) -> Option<Self> {
        let is_null = obj.get("is_null").map(|v| bool::from_input_value(v));
        let is_null = match is_null {
            Some(Some(b)) => Some(IsNull::new(b)),
            Some(None) => return None,
            None => None,
        };
        let additional = match V::AdditionalFilter::from_inner_input_value(obj) {
            Some(a) => a,
            None => return None,
        };
        Some(Self {
            is_null,
            additional,
        })
    }

    fn from_inner_look_ahead(obj: &[(&str, LookAheadValue)]) -> Self {
        let is_null = obj.iter()
            .find(|o| o.0 == "is_null")
            .and_then(|o| bool::from_look_ahead(&o.1))
            .map(IsNull::new);
        let additional = V::AdditionalFilter::from_inner_look_ahead(obj);
        Self {
            is_null,
            additional,
        }
    }

    fn to_inner_input_value(&self, _v: &mut OrderMap<&str, InputValue>) {}

    fn register_fields<'r>(
        _info: &NameBuilder<Self>,
        registry: &mut Registry<'r>,
    ) -> Vec<Argument<'r>> {
        let is_null = registry.arg_with_default::<Option<bool>>("is_null", &None, &());
        let additional = V::AdditionalFilter::register_fields(&Default::default(), registry);
        let mut ret = vec![is_null];
        ret.extend(additional);
        ret
    }
}
