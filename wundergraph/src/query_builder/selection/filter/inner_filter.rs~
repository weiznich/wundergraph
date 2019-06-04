use crate::helper::{NameBuilder, Nameable};
use crate::scalar::WundergraphScalarValue;

use indexmap::IndexMap;
use juniper::meta::Argument;
use juniper::{InputValue, LookAheadValue, Registry};

/// A trait marking that some type is part of a filter
///
/// The main objective of this trait is to allow adding a
/// new filter type without implementing multiple traits.
pub trait InnerFilter: Sized + Nameable {
    /// The used context type
    type Context;

    /// The number of fields created by this filter
    const FIELD_COUNT: usize;

    /// Create the given filter from a graphql input value
    fn from_inner_input_value(
        v: IndexMap<&str, &InputValue<WundergraphScalarValue>>,
    ) -> Option<Self>;
    /// Create the given filter from a graphql lookahead value
    fn from_inner_look_ahead(v: &[(&str, LookAheadValue<'_, WundergraphScalarValue>)]) -> Self;
    /// Covert the given filter into a graphql value
    fn to_inner_input_value(&self, v: &mut IndexMap<&str, InputValue<WundergraphScalarValue>>);
    /// Register all fields of the the filter in a given graphql schema
    ///
    /// This method should register exactly `FIELD_COUNT` new fields
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

    fn from_inner_look_ahead(_v: &[(&str, LookAheadValue<'_, WundergraphScalarValue>)]) -> Self {}
    fn to_inner_input_value(&self, _v: &mut IndexMap<&str, InputValue<WundergraphScalarValue>>) {}
    fn register_fields<'r>(
        _info: &NameBuilder<Self>,
        _registry: &mut Registry<'r, WundergraphScalarValue>,
    ) -> Vec<Argument<'r, WundergraphScalarValue>> {
        vec![]
    }
}
