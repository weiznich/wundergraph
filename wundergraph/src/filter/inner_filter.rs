use helper::{NameBuilder, Nameable};
use scalar::WundergraphScalarValue;

use indexmap::IndexMap;
use juniper::meta::Argument;
use juniper::{InputValue, LookAheadValue, Registry};

pub trait InnerFilter: Sized + Nameable {
    type Context;

    const FIELD_COUNT: usize;

    fn from_inner_input_value(
        v: IndexMap<&str, &InputValue<WundergraphScalarValue>>,
    ) -> Option<Self>;
    fn from_inner_look_ahead(v: &[(&str, LookAheadValue<WundergraphScalarValue>)]) -> Self;
    fn to_inner_input_value(&self, v: &mut IndexMap<&str, InputValue<WundergraphScalarValue>>);
    fn register_fields<'r>(
        info: &NameBuilder<Self>,
        registry: &mut Registry<'r, WundergraphScalarValue>,
    ) -> Vec<Argument<'r, WundergraphScalarValue>>;
}

impl InnerFilter for () {
    type Context = ();

    const FIELD_COUNT: usize = 0;
    fn from_inner_input_value(
        _v: IndexMap<&str, &InputValue<WundergraphScalarValue>>,
    ) -> Option<Self> {
        Some(())
    }
    fn from_inner_look_ahead(_v: &[(&str, LookAheadValue<WundergraphScalarValue>)]) -> Self {
        ()
    }
    fn to_inner_input_value(&self, _v: &mut IndexMap<&str, InputValue<WundergraphScalarValue>>) {}
    fn register_fields<'r>(
        _info: &NameBuilder<Self>,
        _registry: &mut Registry<'r, WundergraphScalarValue>,
    ) -> Vec<Argument<'r, WundergraphScalarValue>> {
        vec![]
    }
}
