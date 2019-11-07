use super::WundergraphValue;
use crate::error::Result;
use crate::scalar::WundergraphScalarValue;
use diesel::backend::Backend;
use juniper::{Executor, Selection};

mod direct_resolver;
mod has_one_resolver;

/// A internal helper trait indicating how to resolve a given type while query
/// execution
pub trait ResolveWundergraphFieldValue<DB: Backend, Ctx>: WundergraphValue + Sized {
    /// A type implementing `FieldValueResolver` used to resolve values of
    /// this type during query execution
    type Resolver: FieldValueResolver<Self, DB, Ctx>;
}

pub trait FieldValueResolver<T, DB, Ctx>
where
    T: WundergraphValue,
    DB: Backend,
{
    fn new(elements: usize) -> Self;

    fn resolve_value(
        &mut self,
        value: T::PlaceHolder,
        look_ahead: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    ) -> Result<Option<juniper::Value<WundergraphScalarValue>>>;

    fn finalize(
        self,
        global_args: &[juniper::LookAheadArgument<WundergraphScalarValue>],
        look_ahead: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    ) -> Result<Option<Vec<juniper::Value<WundergraphScalarValue>>>>;
}
