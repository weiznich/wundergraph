use crate::graphql_type::GraphqlOrderWrapper;
use crate::helper::FromLookAheadValue;
use crate::scalar::WundergraphScalarValue;
use crate::{LoadingHandler, ApplyOffset};
use diesel::backend::Backend;
use diesel::query_builder::QueryFragment;
use diesel::QuerySource;
use juniper::LookAheadValue;
use juniper::{meta, FromInputValue, GraphQLType, Registry, ToInputValue};
use std::marker::PhantomData;

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
pub struct OrderBy<L, DB, Ctx>(PhantomData<(L, DB, Ctx)>);

#[derive(Debug)]
pub struct OrderByTypeInfo<L, DB, Ctx>(String, PhantomData<(L, DB, Ctx)>);

impl<L, DB, Ctx> Default for OrderByTypeInfo<L, DB, Ctx>
where
    DB: Backend + ApplyOffset + 'static,
    L::Table: 'static,
    <L::Table as QuerySource>::FromClause: QueryFragment<DB>,
    L: LoadingHandler<DB, Ctx>,
    DB::QueryBuilder: Default,
{
    fn default() -> Self {
        Self(format!("{}OrderBy", L::TYPE_NAME), PhantomData)
    }
}

impl<T, DB, Ctx> FromInputValue<WundergraphScalarValue> for OrderBy<T, DB, Ctx> {
    fn from_input_value(_: &juniper::InputValue<WundergraphScalarValue>) -> Option<Self> {
        Some(Self(PhantomData))
    }
}

impl<T, DB, Ctx> ToInputValue<WundergraphScalarValue> for OrderBy<T, DB, Ctx> {
    fn to_input_value(&self) -> juniper::InputValue<WundergraphScalarValue> {
        juniper::InputValue::Null
    }
}

impl<T, DB, Ctx> GraphQLType<WundergraphScalarValue> for OrderBy<T, DB, Ctx>
where
    DB: Backend + ApplyOffset + 'static,
    T::Table: 'static,
    <T::Table as QuerySource>::FromClause: QueryFragment<DB>,
    T: LoadingHandler<DB, Ctx>,
    DB::QueryBuilder: Default,
    GraphqlOrderWrapper<T, DB, Ctx>: GraphQLType<WundergraphScalarValue>,
    <GraphqlOrderWrapper<T, DB, Ctx> as GraphQLType<WundergraphScalarValue>>::TypeInfo: Default,
{
    type Context = ();
    type TypeInfo = OrderByTypeInfo<T, DB, Ctx>;

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
            registry.arg::<GraphqlOrderWrapper<T, DB, Ctx>>("column", &Default::default()),
            registry.arg_with_default("direction", &Order::Asc, &()),
        ];

        let obj = registry.build_input_object_type::<Self>(info, args);
        meta::MetaType::InputObject(obj)
    }
}
