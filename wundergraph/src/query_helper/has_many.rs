use crate::graphql_type::WundergraphGraphqlMapper;
use crate::scalar::WundergraphScalarValue;
use juniper::{meta, Registry};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct HasMany<T>(Vec<T>);

impl<T, DB, Ctx> WundergraphGraphqlMapper<DB, Ctx> for HasMany<T>
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
