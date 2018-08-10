use diesel::backend::Backend;
use diesel::sql_types::{NotNull, Nullable};
use diesel::Queryable;
use filter::filter_value::FilterValue;
use helper::{FromLookAheadValue, Nameable};
use juniper::meta::MetaType;
use juniper::{
    Arguments, ExecutionResult, Executor, FieldError, FromInputValue, GraphQLType, InputValue,
    LookAheadValue, Registry, Selection, ToInputValue, Value,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum LazyLoad<T> {
    NotLoaded,
    Item(T),
}

impl<T> Default for LazyLoad<T> {
    fn default() -> Self {
        LazyLoad::NotLoaded
    }
}

impl<T> LazyLoad<T> {
    pub fn expect_loaded(&self, msg: &str) -> &T {
        if let LazyLoad::Item(ref i) = *self {
            i
        } else {
            panic!("{}", msg)
        }
    }
}

impl<DB, T, ST> Queryable<Nullable<ST>, DB> for LazyLoad<T>
where
    DB: Backend,
    T: Queryable<ST, DB>,
    ST: NotNull,
{
    type Row = <Option<T> as Queryable<Nullable<ST>, DB>>::Row;

    fn build(row: Self::Row) -> Self {
        match Queryable::build(row) {
            None => LazyLoad::NotLoaded,
            Some(i) => LazyLoad::Item(i),
        }
    }
}

impl<T, C> FilterValue<C> for LazyLoad<T>
where
    T: FilterValue<C>,
{
    type RawValue = <T as FilterValue<C>>::RawValue;
    type AdditionalFilter = <T as FilterValue<C>>::AdditionalFilter;
}

impl<T> GraphQLType for LazyLoad<T>
where
    T: GraphQLType,
{
    type Context = T::Context;
    type TypeInfo = T::TypeInfo;

    fn name(info: &Self::TypeInfo) -> Option<&str> {
        T::name(info)
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
            LazyLoad::NotLoaded => Err(FieldError::new("LazyLoad item not loaded", Value::Null)),
            LazyLoad::Item(ref i) => i.resolve_field(info, field_name, arguments, executor),
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
            LazyLoad::NotLoaded => Err(FieldError::new("LazyLoad item not loaded", Value::Null)),
            LazyLoad::Item(ref i) => i.resolve_into_type(info, type_name, selection_set, executor),
        }
    }

    fn concrete_type_name(&self, context: &Self::Context, info: &Self::TypeInfo) -> String {
        match *self {
            LazyLoad::NotLoaded => unreachable!(),
            LazyLoad::Item(ref i) => i.concrete_type_name(context, info),
        }
    }

    fn resolve(
        &self,
        info: &Self::TypeInfo,
        selection_set: Option<&[Selection]>,
        executor: &Executor<Self::Context>,
    ) -> Value {
        match *self {
            LazyLoad::NotLoaded => unreachable!(),
            LazyLoad::Item(ref i) => i.resolve(info, selection_set, executor),
        }
    }
}

impl<T> FromInputValue for LazyLoad<T>
where
    T: FromInputValue,
{
    fn from_input_value(v: &InputValue) -> Option<Self> {
        T::from_input_value(v).map(LazyLoad::Item)
    }
}

impl<T> ToInputValue for LazyLoad<T>
where
    T: ToInputValue,
{
    fn to_input_value(&self) -> InputValue {
        match *self {
            LazyLoad::NotLoaded => unreachable!(),
            LazyLoad::Item(ref i) => i.to_input_value(),
        }
    }
}

impl<T> FromLookAheadValue for LazyLoad<T>
where
    T: FromLookAheadValue,
{
    fn from_look_ahead(v: &LookAheadValue) -> Option<Self> {
        T::from_look_ahead(v).map(LazyLoad::Item)
    }
}

impl<T> Nameable for LazyLoad<T>
where
    T: Nameable,
{
    fn name() -> String {
        T::name()
    }
}
