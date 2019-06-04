use super::WundergraphValue;
use crate::scalar::WundergraphScalarValue;
use diesel::backend::Backend;
use failure::Error;
use juniper::{Executor, Selection};

mod direct_resolver;
mod has_one_resolver;

pub trait ResolveWundergraphFieldValue<DB: Backend, Ctx>: WundergraphValue + Sized {
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
    ) -> Result<Option<juniper::Value<WundergraphScalarValue>>, Error>;

    fn finalize(
        self,
        look_ahead: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    ) -> Result<Option<Vec<juniper::Value<WundergraphScalarValue>>>, Error>;
}
