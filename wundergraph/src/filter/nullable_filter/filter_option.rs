use filter::build_filter::BuildFilter;
use filter::collector::{AndCollector, FilterCollector};
use filter::filter_value::FilterValue;
use filter::inner_filter::InnerFilter;

use diesel::backend::Backend;
use diesel::expression::{AsExpression, NonAggregate};
use diesel::query_builder::QueryFragment;
use diesel::sql_types::{Bool, SingleValue};
use diesel::{AppearsOnTable, Column};
use diesel_ext::BoxableFilter;

use juniper::meta::Argument;
use juniper::{FromInputValue, InputValue, LookAheadValue, Registry};

use indexmap::IndexMap;

use helper::{FromLookAheadValue, NameBuilder, Nameable};
use scalar::WundergraphScalarValue;

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
        Self {
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
    <V::AdditionalFilter as BuildFilter<DB>>::Ret: AppearsOnTable<C::Table> + QueryFragment<DB>,
    V::RawValue: AsExpression<C::SqlType> + 'static,
    <V::RawValue as AsExpression<C::SqlType>>::Expression:
        AppearsOnTable<C::Table> + NonAggregate + QueryFragment<DB> + 'static,
    IsNull<C>: BuildFilter<DB>,
    <IsNull<C> as BuildFilter<DB>>::Ret: AppearsOnTable<C::Table> + QueryFragment<DB>,
{
    type Ret = Box<BoxableFilter<C::Table, DB, SqlType = Bool>>;

    fn into_filter(self) -> Option<Self::Ret> {
        let mut combinator = AndCollector::default();
        combinator.append_filter(self.is_null);
        combinator.append_filter(self.additional);
        combinator.into_filter()
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
    fn from_inner_input_value(
        obj: IndexMap<&str, &InputValue<WundergraphScalarValue>>,
    ) -> Option<Self> {
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

    fn from_inner_look_ahead(obj: &[(&str, LookAheadValue<WundergraphScalarValue>)]) -> Self {
        let is_null = obj
            .iter()
            .find(|o| o.0 == "is_null")
            .and_then(|o| bool::from_look_ahead(&o.1))
            .map(IsNull::new);
        let additional = V::AdditionalFilter::from_inner_look_ahead(obj);
        Self {
            is_null,
            additional,
        }
    }

    fn to_inner_input_value(&self, _v: &mut IndexMap<&str, InputValue<WundergraphScalarValue>>) {}

    fn register_fields<'r>(
        _info: &NameBuilder<Self>,
        registry: &mut Registry<'r, WundergraphScalarValue>,
    ) -> Vec<Argument<'r, WundergraphScalarValue>> {
        let is_null = registry.arg_with_default::<Option<bool>>("is_null", &None, &());
        let additional = V::AdditionalFilter::register_fields(&NameBuilder::default(), registry);
        let mut ret = vec![is_null];
        ret.extend(additional);
        ret
    }
}
