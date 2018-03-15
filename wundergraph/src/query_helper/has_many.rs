use juniper::{Arguments, ExecutionResult, Executor, FieldError, GraphQLType, Registry, Selection,
              Value};
use juniper::meta::MetaType;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum HasMany<T> {
    NotLoaded,
    Items(Vec<T>),
}

impl<T> Default for HasMany<T> {
    fn default() -> Self {
        HasMany::NotLoaded
    }
}

impl<T> HasMany<T> {
    pub fn expect_items(&self, msg: &str) -> &[T] {
        if let HasMany::Items(ref i) = *self {
            i
        } else {
            panic!("{}", msg)
        }
    }
}

impl<T> GraphQLType for HasMany<T>
where
    T: GraphQLType,
{
    type Context = T::Context;
    type TypeInfo = T::TypeInfo;

    fn name(info: &Self::TypeInfo) -> Option<&str> {
        Vec::<T>::name(info)
    }

    fn meta<'r>(info: &Self::TypeInfo, registry: &mut Registry<'r>) -> MetaType<'r> {
        Vec::<T>::meta(info, registry)
    }

    fn resolve_field(
        &self,
        info: &Self::TypeInfo,
        field_name: &str,
        arguments: &Arguments,
        executor: &Executor<Self::Context>,
    ) -> ExecutionResult {
        match *self {
            HasMany::NotLoaded => Err(FieldError::new("HasMany relation not loaded", Value::Null)),
            HasMany::Items(ref i) => i.resolve_field(info, field_name, arguments, executor),
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
            HasMany::NotLoaded => Err(FieldError::new("HasMany relation not loaded", Value::Null)),
            HasMany::Items(ref i) => i.resolve_into_type(info, type_name, selection_set, executor),
        }
    }

    fn concrete_type_name(&self, context: &Self::Context, info: &Self::TypeInfo) -> String {
        match *self {
            HasMany::NotLoaded => unreachable!(),
            HasMany::Items(ref i) => i.concrete_type_name(context, info),
        }
    }

    fn resolve(
        &self,
        info: &Self::TypeInfo,
        selection_set: Option<&[Selection]>,
        executor: &Executor<Self::Context>,
    ) -> Value {
        match *self {
            HasMany::NotLoaded => unreachable!(),
            HasMany::Items(ref i) => i.resolve(info, selection_set, executor),
        }
    }
}
