use super::{DirectResolveable, FieldValueResolver, ResolveWundergraphFieldValue};
use crate::error::Result;
use crate::error::WundergraphError;
use crate::query_builder::types::WundergraphSqlValue;
use crate::scalar::WundergraphScalarValue;
use diesel::backend::Backend;
use juniper::{Executor, FromContext, GraphQLType, Selection};

#[derive(Debug, Clone, Copy)]
pub struct DirectResolver;

impl<T, DB, Ctx> FieldValueResolver<T, DB, Ctx> for DirectResolver
where
    DB: Backend,
    T: GraphQLType<WundergraphScalarValue> + WundergraphSqlValue + DirectResolveable,
    T::TypeInfo: Default,
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
                .resolve_with_ctx(
                    &T::TypeInfo::default(),
                    &value.into().expect("Loading should not fail"),
                )
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

impl<T, DB, Ctx> FieldValueResolver<Option<T>, DB, Ctx> for DirectResolver
where
    DB: Backend,
    //    T: GraphQLType<WundergraphScalarValue>,
    Option<T>: GraphQLType<WundergraphScalarValue> + WundergraphSqlValue,
    <Option<T> as GraphQLType<WundergraphScalarValue>>::TypeInfo: Default,
    <Option<T> as WundergraphSqlValue>::PlaceHolder: Into<Option<Option<T>>>,
    <Option<T> as GraphQLType<WundergraphScalarValue>>::Context: FromContext<Ctx>,
{
    fn new(_elements: usize) -> Self {
        Self
    }

    fn resolve_value(
        &mut self,
        value: <Option<T> as WundergraphSqlValue>::PlaceHolder,
        _look_ahead: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        _selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    ) -> Result<Option<juniper::Value<WundergraphScalarValue>>> {
        Ok(Some(
            executor
                .resolve_with_ctx(
                    &<Option<T> as GraphQLType<WundergraphScalarValue>>::TypeInfo::default(),
                    &value.into().expect("Loading should not fail"),
                )
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
    T: GraphQLType<WundergraphScalarValue> + WundergraphSqlValue + DirectResolveable,
    DirectResolver: FieldValueResolver<T, DB, Ctx>,
{
    type Resolver = DirectResolver;
}

impl<T, DB, Ctx> ResolveWundergraphFieldValue<DB, Ctx> for Option<T>
where
    T: DirectResolveable,
    DB: Backend,
    T: GraphQLType<WundergraphScalarValue> + WundergraphSqlValue,
    DirectResolver: FieldValueResolver<Option<T>, DB, Ctx>,
{
    type Resolver = DirectResolver;
}
