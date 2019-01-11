use super::BuildFilter;
use diesel::backend::Backend;
use diesel::dsl;
use diesel::helper_types;
use helper::FromLookAheadValue;
use juniper::meta::MetaType;
use juniper::{FromInputValue, GraphQLType, InputValue, LookAheadValue, Registry, ToInputValue};
use scalar::WundergraphScalarValue;

#[derive(Debug)]
pub struct Not<I>(I);

impl<DB, I> BuildFilter<DB> for Not<I>
where
    DB: Backend,
    I: BuildFilter<DB>,
{
    type Ret = helper_types::not<I::Ret>;

    fn into_filter(self) -> Option<Self::Ret> {
        self.0.into_filter().map(dsl::not)
    }
}

impl<I> FromInputValue<WundergraphScalarValue> for Not<I>
where
    I: FromInputValue<WundergraphScalarValue>,
{
    fn from_input_value(v: &InputValue<WundergraphScalarValue>) -> Option<Self> {
        I::from_input_value(v).map(Not)
    }
}

impl<I> FromLookAheadValue for Not<I>
where
    I: FromLookAheadValue,
{
    fn from_look_ahead(v: &LookAheadValue<WundergraphScalarValue>) -> Option<Self> {
        I::from_look_ahead(v).map(Not)
    }
}

impl<I> ToInputValue<WundergraphScalarValue> for Not<I>
where
    I: ToInputValue<WundergraphScalarValue>,
{
    fn to_input_value(&self) -> InputValue<WundergraphScalarValue> {
        I::to_input_value(&self.0)
    }
}

impl<F> GraphQLType<WundergraphScalarValue> for Not<F>
where
    F: GraphQLType<WundergraphScalarValue>,
{
    type Context = F::Context;
    type TypeInfo = F::TypeInfo;

    fn name(_info: &Self::TypeInfo) -> Option<&str> {
        Some("not")
    }

    fn meta<'r>(
        info: &Self::TypeInfo,
        registry: &mut Registry<'r, WundergraphScalarValue>,
    ) -> MetaType<'r, WundergraphScalarValue>
    where
        WundergraphScalarValue: 'r,
    {
        F::meta(info, registry)
    }
}
