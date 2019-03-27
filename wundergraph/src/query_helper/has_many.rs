use crate::graphql_type::WundergraphGraphqlMapper;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct HasMany<T>(Vec<T>);

impl<T, DB, Ctx> WundergraphGraphqlMapper<DB, Ctx> for HasMany<T>
where
    T: WundergraphGraphqlMapper<DB, Ctx>,
{
    type GraphQLType = Vec<T::GraphQLType>;
}
