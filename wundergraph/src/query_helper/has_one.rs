use diesel::Queryable;
use diesel::backend::Backend;
use diesel::expression::AsExpression;
use diesel::expression::bound::Bound;
use juniper::{Arguments, ExecutionResult, Executor, FieldError, GraphQLType, Registry, Selection,
              Value};
use juniper::meta::MetaType;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum HasOne<R, T> {
    Id(R),
    Item(T),
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

impl<R, T> GraphQLType for HasOne<R, T>
where
    T: GraphQLType,
{
    type Context = T::Context;
    type TypeInfo = T::TypeInfo;

    fn name(info: &Self::TypeInfo) -> Option<&str> {
        T::name(info)
    }

    fn meta<'r>(info: &Self::TypeInfo, registry: &mut Registry<'r>) -> MetaType<'r> {
        T::meta(info, registry)
    }

    fn resolve_field(
        &self,
        info: &Self::TypeInfo,
        field_name: &str,
        arguments: &Arguments,
        executor: &Executor<Self::Context>,
    ) -> ExecutionResult {
        match *self {
            HasOne::Id(_) => Err(FieldError::new("HasOne relation not loaded", Value::Null)),
            HasOne::Item(ref i) => i.resolve_field(info, field_name, arguments, executor),
        }
    }

    fn resolve_into_type(
        &self,
        info: &Self::TypeInfo,
        type_name: &str,
        selection_set: Option<&[Selection]>,
        executor: &Executor<Self::Context>,
    ) -> ExecutionResult {
        match *self {
            HasOne::Id(_) => Err(FieldError::new("HasOne relation not loaded", Value::Null)),
            HasOne::Item(ref i) => i.resolve_into_type(info, type_name, selection_set, executor),
        }
    }

    fn concrete_type_name(&self, context: &Self::Context, info: &Self::TypeInfo) -> String {
        match *self {
            HasOne::Id(_) => unreachable!(),
            HasOne::Item(ref i) => i.concrete_type_name(context, info),
        }
    }

    fn resolve(
        &self,
        info: &Self::TypeInfo,
        selection_set: Option<&[Selection]>,
        executor: &Executor<Self::Context>,
    ) -> Value {
        match *self {
            HasOne::Id(_) => unreachable!(),
            HasOne::Item(ref i) => i.resolve(info, selection_set, executor),
        }
    }
}
