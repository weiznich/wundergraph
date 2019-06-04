use crate::graphql_type::WundergraphGraphqlMapper;
use crate::scalar::WundergraphScalarValue;
use juniper::{meta, Registry};
use std::marker::PhantomData;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct HasMany<T, FK>(Vec<T>, PhantomData<FK>);

impl<T, DB, Ctx, FK> WundergraphGraphqlMapper<DB, Ctx> for HasMany<T, FK>
where
    T: WundergraphGraphqlMapper<DB, Ctx>,
{
    type GraphQLType = Vec<T::GraphQLType>;

    fn register_arguments<'r>(
        registry: &mut Registry<'r, WundergraphScalarValue>,
        field: meta::Field<'r, WundergraphScalarValue>,
    ) -> meta::Field<'r, WundergraphScalarValue> {
        T::register_arguments(registry, field)
    }
}
