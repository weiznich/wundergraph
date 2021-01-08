use crate::graphql_type::WundergraphGraphqlMapper;
use crate::helper::*;
use crate::juniper_ext::FromLookAheadValue;
use crate::scalar::WundergraphScalarValue;
use diesel::associations::Identifiable;
use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::expression::bound::Bound;
use diesel::expression::AsExpression;
use diesel::Queryable;
use juniper::meta::Argument;
use juniper::{FromInputValue, GraphQLType, InputValue, LookAheadValue, Registry};
use std::hash::{Hash, Hasher};

/// Type used to indicate that a given field references a single
/// other entity by id
#[derive(Debug, Clone)]
pub enum HasOne<K, O> {
    #[doc(hidden)]
    Id(K),
    #[doc(hidden)]
    Item(O),
}

impl<R, T> PartialEq for HasOne<R, T>
where
    R: PartialEq + Hash + Eq,
    for<'a> &'a T: Identifiable<Id = &'a R>,
{
    fn eq(&self, other: &Self) -> bool {
        let k = match self {
            HasOne::Id(ref i) => i,
            HasOne::Item(ref i) => i.id(),
        };
        let other = match other {
            HasOne::Id(ref i) => i,
            HasOne::Item(ref i) => i.id(),
        };
        <_ as PartialEq>::eq(k, other)
    }
}

impl<R, T> Eq for HasOne<R, T> where Self: PartialEq {}

impl<R, T> Hash for HasOne<R, T>
where
    R: Hash + Eq,
    for<'a> &'a T: Identifiable<Id = &'a R>,
{
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        match self {
            HasOne::Id(ref i) => i.hash(hasher),
            HasOne::Item(ref i) => i.id().hash(hasher),
        }
    }
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
    fn from_look_ahead(v: &LookAheadValue<'_, WundergraphScalarValue>) -> Option<Self> {
        R::from_look_ahead(v).map(HasOne::Id)
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

use diesel::serialize::{self, ToSql};
use std::io::Write;

use super::AsInputType;

impl<R, T, DB, ST> ToSql<ST, DB> for HasOne<R, T>
where
    DB: Backend,
    R: ToSql<ST, DB> + Eq + Hash,
    for<'a> &'a T: Identifiable<Id = &'a R>,
    T: std::fmt::Debug,
{
    fn to_sql<W: Write>(&self, out: &mut serialize::Output<W, DB>) -> serialize::Result {
        match self {
            HasOne::Id(ref i) => i.to_sql(out),
            HasOne::Item(ref i) => i.id().to_sql(out),
        }
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

impl<'expr, R, T, ST> AsExpression<ST> for &'expr HasOne<R, T>
where
    &'expr R: AsExpression<ST>,
{
    type Expression = Bound<ST, Self>;

    fn as_expression(self) -> Self::Expression {
        Bound::new(self)
    }
}

impl<ST, R, T> AsExpression<ST> for HasOne<R, T>
where
    R: AsExpression<ST>,
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

impl<R, T, DB, Ctx> WundergraphGraphqlMapper<DB, Ctx> for HasOne<R, T>
where
    T: WundergraphGraphqlMapper<DB, Ctx>,
{
    type GraphQLType = T::GraphQLType;

    fn type_info() -> <Self::GraphQLType as GraphQLType<WundergraphScalarValue>>::TypeInfo {
        T::type_info()
    }
}

impl<T, DB, Ctx> WundergraphGraphqlMapper<DB, Ctx> for Option<T>
where
    T: WundergraphGraphqlMapper<DB, Ctx>,
{
    type GraphQLType = Option<T::GraphQLType>;

    fn type_info() -> <Self::GraphQLType as GraphQLType<WundergraphScalarValue>>::TypeInfo {
        T::type_info()
    }
}

impl<R, T, C> PrimaryKeyInputObject<HasOne<R, T>> for C
where
    C: PrimaryKeyInputObject<R>,
    R: Eq + Hash,
    for<'a> &'a T: Identifiable<Id = &'a R>,
{
    fn register<'r>(
        registry: &mut Registry<'r, WundergraphScalarValue>,
    ) -> Vec<Argument<'r, WundergraphScalarValue>> {
        Self::register(registry)
    }

    fn from_input_value(value: &InputValue<WundergraphScalarValue>) -> Option<HasOne<R, T>> {
        Self::from_input_value(value).map(HasOne::Id)
    }
    fn from_look_ahead(
        look_ahead: &LookAheadValue<'_, WundergraphScalarValue>,
    ) -> Option<HasOne<R, T>> {
        Self::from_look_ahead(look_ahead).map(HasOne::Id)
    }
    fn to_input_value(values: &HasOne<R, T>) -> InputValue<WundergraphScalarValue> {
        match *values {
            HasOne::Id(ref id) => C::to_input_value(id),
            HasOne::Item(ref i) => C::to_input_value(i.id()),
        }
    }
}

impl<R, T> AsInputType for HasOne<R, T>
where
    R: AsInputType,
{
    type InputType = <R as AsInputType>::InputType;
}
