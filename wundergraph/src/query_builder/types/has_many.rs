use crate::graphql_type::WundergraphGraphqlMapper;
use crate::query_builder::types::wundergraph_value::{AssociatedValue, WundergraphValue};
use crate::scalar::WundergraphScalarValue;
use juniper::{meta, GraphQLType, Registry};
use std::marker::PhantomData;

/// Type used to indicate that a given field references multiple other entities
/// by a given id
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

    fn type_info() -> <Self::GraphQLType as GraphQLType<WundergraphScalarValue>>::TypeInfo {
        T::type_info()
    }
}

impl<T, FK> WundergraphValue for HasMany<T, FK> {
    type ValueType = AssociatedValue;
}
