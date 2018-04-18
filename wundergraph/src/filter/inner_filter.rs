use helper::{NameBuilder, Nameable};

use juniper::meta::Argument;
use juniper::{InputValue, LookAheadValue, Registry};
use ordermap::OrderMap;

pub trait InnerFilter: Sized + Nameable {
    type Context;

    const FIELD_COUNT: usize;

    fn from_inner_input_value(v: OrderMap<&str, &InputValue>) -> Option<Self>;
    fn from_inner_look_ahead(v: &[(&str, LookAheadValue)]) -> Self;
    fn to_inner_input_value(&self, v: &mut OrderMap<&str, InputValue>);
    fn register_fields<'r>(
        info: &NameBuilder<Self>,
        registry: &mut Registry<'r>,
    ) -> Vec<Argument<'r>>;
}

impl InnerFilter for () {
    type Context = ();

    const FIELD_COUNT: usize = 0;
    fn from_inner_input_value(_v: OrderMap<&str, &InputValue>) -> Option<Self> {
        Some(())
    }
    fn from_inner_look_ahead(_v: &[(&str, LookAheadValue)]) -> Self {
        ()
    }
    fn to_inner_input_value(&self, _v: &mut OrderMap<&str, InputValue>) {}
    fn register_fields<'r>(
        _info: &NameBuilder<Self>,
        _registry: &mut Registry<'r>,
    ) -> Vec<Argument<'r>> {
        vec![]
    }
}
