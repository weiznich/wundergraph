use super::{FieldValueResolver, ResolveWundergraphFieldValue};
use crate::error::Result;
use crate::error::WundergraphError;
use crate::query_builder::types::WundergraphValue;
use crate::scalar::WundergraphScalarValue;
use diesel::backend::Backend;
use juniper::{Executor, FromContext, GraphQLType, Selection};

#[derive(Debug, Clone, Copy)]
pub struct DirectResolver;

impl<T, DB, Ctx> FieldValueResolver<T, DB, Ctx> for DirectResolver
where
    DB: Backend,
    T: GraphQLType<WundergraphScalarValue, TypeInfo = ()> + WundergraphValue,
    T::PlaceHolder: Into<Option<T>>,
    <T as GraphQLType<WundergraphScalarValue>>::Context: FromContext<Ctx>,
{
    fn new(_elements: usize) -> Self {
        Self
    }

    fn resolve_value(
        &mut self,
        value: T::PlaceHolder,
        _look_ahead: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        _selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    ) -> Result<Option<juniper::Value<WundergraphScalarValue>>> {
        Ok(Some(
            executor
                .resolve_with_ctx(&(), &value.into().expect("Loading should not fail"))
                .map_err(|inner| WundergraphError::JuniperError { inner })?,
        ))
    }

    fn finalize(
        self,
        _global_args: &[juniper::LookAheadArgument<WundergraphScalarValue>],
        _look_ahead: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        _selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        _executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    ) -> Result<Option<Vec<juniper::Value<WundergraphScalarValue>>>> {
        Ok(None)
    }
}

impl<T, DB, Ctx> ResolveWundergraphFieldValue<DB, Ctx> for T
where
    DB: Backend,
    T: GraphQLType<WundergraphScalarValue> + WundergraphValue,
    DirectResolver: FieldValueResolver<T, DB, Ctx>,
{
    type Resolver = DirectResolver;
}
