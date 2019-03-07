use diesel::backend::Backend;
use diesel::query_builder::QueryFragment;
use diesel::QuerySource;
use crate::graphql_type::GraphqlOrderWrapper;
use crate::helper::FromLookAheadValue;
use juniper::LookAheadValue;
use juniper::{meta, FromInputValue, GraphQLType, Registry, ToInputValue};
use crate::scalar::WundergraphScalarValue;
use std::marker::PhantomData;
use crate::LoadingHandler;

#[derive(Debug, GraphQLEnum, Copy, Clone, PartialEq)]
pub enum Order {
    Asc,
    Desc,
}

impl FromLookAheadValue for Order {
    fn from_look_ahead(v: &LookAheadValue<'_, WundergraphScalarValue>) -> Option<Self> {
        if let LookAheadValue::Enum(e) = *v {
            match e {
                "ASC" => Some(Order::Asc),
                "DESC" => Some(Order::Desc),
                _ => None,
            }
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct OrderBy<L, DB>(PhantomData<(L, DB)>);

#[derive(Debug)]
pub struct OrderByTypeInfo<L, DB>(String, PhantomData<(L, DB)>);

impl<L, DB> Default for OrderByTypeInfo<L, DB>
where
    DB: Backend + 'static,
    L::Table: 'static,
    <L::Table as QuerySource>::FromClause: QueryFragment<DB>,
    L: LoadingHandler<DB>,
    DB::QueryBuilder: Default,
{
    fn default() -> Self {
        OrderByTypeInfo(format!("{}OrderBy", L::TYPE_NAME), PhantomData)
    }
}

impl<T, DB> FromInputValue<WundergraphScalarValue> for OrderBy<T, DB> {
    fn from_input_value(_: &juniper::InputValue<WundergraphScalarValue>) -> Option<Self> {
        Some(Self(PhantomData))
    }
}

impl<T, DB> ToInputValue<WundergraphScalarValue> for OrderBy<T, DB> {
    fn to_input_value(&self) -> juniper::InputValue<WundergraphScalarValue> {
        unimplemented!("That should not been called")
    }
}

impl<T, DB> GraphQLType<WundergraphScalarValue> for OrderBy<T, DB>
where
    DB: Backend + 'static,
    T::Table: 'static,
    <T::Table as QuerySource>::FromClause: QueryFragment<DB>,
    T: LoadingHandler<DB>,
    DB::QueryBuilder: Default,
    GraphqlOrderWrapper<T, DB>: GraphQLType<WundergraphScalarValue>,
    <GraphqlOrderWrapper<T, DB> as GraphQLType<WundergraphScalarValue>>::TypeInfo: Default,
{
    type Context = ();
    type TypeInfo = OrderByTypeInfo<T, DB>;

    fn name(info: &Self::TypeInfo) -> Option<&str> {
        Some(&info.0)
    }

    fn meta<'r>(
        info: &Self::TypeInfo,
        registry: &mut Registry<'r, WundergraphScalarValue>,
    ) -> meta::MetaType<'r, WundergraphScalarValue>
    where
        WundergraphScalarValue: 'r,
    {
        let args = &[
            registry.arg::<GraphqlOrderWrapper<T, DB>>("column", &Default::default()),
            registry.arg_with_default("direction", &Order::Asc, &()),
        ];

        let obj = registry.build_input_object_type::<Self>(info, args);
        meta::MetaType::InputObject(obj)
    }
}
