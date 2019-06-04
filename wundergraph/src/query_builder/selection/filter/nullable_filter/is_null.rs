use crate::diesel_ext::BoxableFilter;
use crate::juniper_ext::{FromLookAheadValue, NameBuilder, Nameable};
use crate::query_builder::selection::filter::build_filter::BuildFilter;
use crate::query_builder::selection::filter::inner_filter::InnerFilter;
use crate::scalar::WundergraphScalarValue;
use diesel::backend::Backend;
use diesel::expression::{operators, NonAggregate};
use diesel::query_builder::QueryFragment;
use diesel::sql_types::Bool;
use diesel::{AppearsOnTable, Column, ExpressionMethods};
use indexmap::IndexMap;
use juniper::meta::Argument;
use juniper::{FromInputValue, InputValue, LookAheadValue, Registry, ToInputValue};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct IsNull<C>(bool, PhantomData<C>);

impl<C> IsNull<C> {
    pub(crate) fn new(v: bool) -> Self {
        Self(v, PhantomData)
    }
}

impl<C> Clone for IsNull<C> {
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}

impl<C, DB> BuildFilter<DB> for IsNull<C>
where
    C: Column + ExpressionMethods + NonAggregate + QueryFragment<DB> + Default + 'static,
    DB: Backend + 'static,
    C::Table: 'static,
    operators::IsNull<C>: AppearsOnTable<C::Table, SqlType = Bool>,
    operators::IsNotNull<C>: AppearsOnTable<C::Table, SqlType = Bool>,
{
    type Ret = Box<dyn BoxableFilter<C::Table, DB, SqlType = Bool>>;

    fn into_filter(self) -> Option<Self::Ret> {
        if self.0 {
            Some(Box::new(C::default().is_null()) as Box<_>)
        } else {
            Some(Box::new(C::default().is_not_null()) as Box<_>)
        }
    }
}

impl<C> ToInputValue<WundergraphScalarValue> for IsNull<C> {
    fn to_input_value(&self) -> InputValue<WundergraphScalarValue> {
        self.0.to_input_value()
    }
}

impl<C> Nameable for IsNull<C> {
    fn name() -> String {
        String::from("is_null")
    }
}

//That's a false positive by clippy
#[allow(clippy::use_self)]
impl<C> InnerFilter for Option<IsNull<C>> {
    type Context = ();

    const FIELD_COUNT: usize = 1;
    fn from_inner_input_value(
        obj: IndexMap<&str, &InputValue<WundergraphScalarValue>>,
    ) -> Option<Self> {
        let is_null = obj.get("is_null").map(|v| bool::from_input_value(v));
        match is_null {
            Some(Some(b)) => Some(Some(IsNull::new(b))),
            Some(None) => None,
            None => Some(None),
        }
    }

    fn from_inner_look_ahead(obj: &[(&str, LookAheadValue<'_, WundergraphScalarValue>)]) -> Self {
        obj.iter()
            .find(|o| o.0 == "is_null")
            .and_then(|o| bool::from_look_ahead(&o.1))
            .map(IsNull::new)
    }

    fn to_inner_input_value(&self, v: &mut IndexMap<&str, InputValue<WundergraphScalarValue>>) {
        v.insert("is_null", self.to_input_value());
    }

    fn register_fields<'r>(
        _info: &NameBuilder<Self>,
        registry: &mut Registry<'r, WundergraphScalarValue>,
    ) -> Vec<Argument<'r, WundergraphScalarValue>> {
        let is_null = registry.arg_with_default::<Option<bool>>("is_null", &None, &());
        vec![is_null]
    }
}
