use diesel::associations::Identifiable;
use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::expression::bound::Bound;
use diesel::expression::AsExpression;
use diesel::Queryable;
use helper::FromLookAheadValue;
use juniper::{FromInputValue, InputValue, LookAheadValue, ToInputValue};

use graphql_type::WundergraphGraphqlMapper;
use std::hash::Hash;

use scalar::WundergraphScalarValue;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum HasOne<R, T> {
    Id(R),
    Item(T),
}

impl<'a, K, I> Into<Option<&'a K>> for &'a HasOne<K, I>
where
    &'a I: Identifiable<Id = &'a K>,
    K: Eq + Hash,
{
    fn into(self) -> Option<&'a K> {
        match *self {
            HasOne::Id(ref k) => Some(k),
            HasOne::Item(ref i) => Some(i.id()),
        }
    }
}

impl<'a, K, I> Into<Option<&'a K>> for &'a HasOne<Option<K>, Option<I>>
where
    &'a I: Identifiable<Id = &'a K>,
    K: Eq + Hash,
{
    fn into(self) -> Option<&'a K> {
        match *self {
            HasOne::Id(Some(ref k)) => Some(k),
            HasOne::Item(Some(ref i)) => Some(i.id()),
            HasOne::Id(None) | HasOne::Item(None) => None,
        }
    }
}

impl<R, T> FromInputValue<WundergraphScalarValue> for HasOne<R, T>
where
    R: FromInputValue<WundergraphScalarValue>,
{
    fn from_input_value(v: &InputValue<WundergraphScalarValue>) -> Option<Self> {
        R::from_input_value(v).map(HasOne::Id)
    }
}

impl<R, T> FromLookAheadValue for HasOne<R, T>
where
    R: FromLookAheadValue,
{
    fn from_look_ahead(v: &LookAheadValue<WundergraphScalarValue>) -> Option<Self> {
        R::from_look_ahead(v).map(HasOne::Id)
    }
}

impl<R, T> ToInputValue for HasOne<R, T>
where
    R: ToInputValue,
    T: ToInputValue,
{
    fn to_input_value(&self) -> InputValue {
        match *self {
            HasOne::Id(ref i) => i.to_input_value(),
            HasOne::Item(ref i) => i.to_input_value(),
        }
    }
}

impl<R, T, DB, ST> FromSql<ST, DB> for HasOne<R, T>
where
    DB: Backend,
    R: FromSql<ST, DB>,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        <R as FromSql<ST, DB>>::from_sql(bytes).map(HasOne::Id)
    }
}

impl<R, T, DB, ST> Queryable<ST, DB> for HasOne<R, T>
where
    DB: Backend,
    R: Queryable<ST, DB>,
{
    type Row = <R as Queryable<ST, DB>>::Row;

    fn build(row: Self::Row) -> Self {
        let row = Queryable::build(row);
        HasOne::Id(row)
    }
}

impl<R, T> HasOne<R, T> {
    pub fn expect_item(&self, msg: &str) -> &T {
        if let HasOne::Item(ref i) = *self {
            i
        } else {
            panic!("{}", msg)
        }
    }

    pub fn expect_id(&self, msg: &str) -> &R {
        if let HasOne::Id(ref i) = *self {
            i
        } else {
            panic!("{}", msg)
        }
    }
}

impl<'expr, R, T, ST> AsExpression<ST> for &'expr HasOne<R, T>
where
    &'expr R: AsExpression<ST>,
{
    type Expression = Bound<ST, Self>;

    fn as_expression(self) -> Self::Expression {
        Bound::new(self)
    }
}

impl<'expr2, 'expr, R, T, ST> AsExpression<ST> for &'expr2 &'expr HasOne<R, T>
where
    &'expr2 &'expr R: AsExpression<ST>,
{
    type Expression = Bound<ST, Self>;

    fn as_expression(self) -> Self::Expression {
        Bound::new(self)
    }
}

impl<R, T, DB> WundergraphGraphqlMapper<DB> for HasOne<R, T>
where
    T: WundergraphGraphqlMapper<DB>,
{
    type GraphQLType = T::GraphQLType;
}

impl<R, T, DB> WundergraphGraphqlMapper<DB> for Option<HasOne<R, T>>
where
    T: WundergraphGraphqlMapper<DB>,
{
    type GraphQLType = Option<T::GraphQLType>;
}

// impl<R, T> GraphQLType<WundergraphScalarValue> for HasOne<R, T>
// where
//     GraphqlWrapper<T>: GraphQLType<WundergraphScalarValue>,
// {
//     type Context = <GraphqlWrapper<T> as GraphQLType<WundergraphScalarValue>>::Context;
//     type TypeInfo = <GraphqlWrapper<T> as GraphQLType<WundergraphScalarValue>>::TypeInfo;

//     fn name(info: &Self::TypeInfo) -> Option<&str> {
//         <GraphqlWrapper<T> as GraphQLType<WundergraphScalarValue>>::name(info)
//     }

//     fn meta<'r>(
//         info: &Self::TypeInfo,
//         registry: &mut Registry<'r, WundergraphScalarValue>,
//     ) -> MetaType<'r, WundergraphScalarValue>
//     where
//         WundergraphScalarValue: 'r,
//     {
//         <GraphqlWrapper<T> as GraphQLType<WundergraphScalarValue>>::meta(info, registry)
//     }

//     fn resolve_field(
//         &self,
//         info: &Self::TypeInfo,
//         field_name: &str,
//         arguments: &Arguments<WundergraphScalarValue>,
//         executor: &Executor<Self::Context, WundergraphScalarValue>,
//     ) -> ExecutionResult<WundergraphScalarValue> {
//         match *self {
//             HasOne::Id(_) => Err(FieldError::new("HasOne relation not loaded", Value::Null)),
//             HasOne::Item(ref i) => {
//                 GraphqlWrapper::new(i).resolve_field(info, field_name, arguments, executor)
//             }
//         }
//     }

//     fn resolve_into_type(
//         &self,
//         info: &Self::TypeInfo,
//         type_name: &str,
//         selection_set: Option<&[Selection<WundergraphScalarValue>]>,
//         executor: &Executor<Self::Context, WundergraphScalarValue>,
//     ) -> ExecutionResult<WundergraphScalarValue> {
//         match *self {
//             HasOne::Id(_) => Err(FieldError::new("HasOne relation not loaded", Value::Null)),
//             HasOne::Item(ref i) => {
//                 GraphqlWrapper::new(i).resolve_into_type(info, type_name, selection_set, executor)
//             }
//         }
//     }

//     fn concrete_type_name(&self, context: &Self::Context, info: &Self::TypeInfo) -> String {
//         match *self {
//             HasOne::Id(_) => unreachable!(),
//             HasOne::Item(ref i) => GraphqlWrapper::new(i).concrete_type_name(context, info),
//         }
//     }

//     fn resolve(
//         &self,
//         info: &Self::TypeInfo,
//         selection_set: Option<&[Selection<WundergraphScalarValue>]>,
//         executor: &Executor<Self::Context, WundergraphScalarValue>,
//     ) -> Value<WundergraphScalarValue> {
//         match *self {
//             HasOne::Id(_) => unreachable!(),
//             HasOne::Item(ref i) => GraphqlWrapper::new(i).resolve(info, selection_set, executor),
//         }
//     }
// }
