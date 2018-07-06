use super::FromLookAheadValue;
use diesel::associations::HasTable;
use diesel::query_builder::nodes::Identifier;
use diesel::{Column, Identifiable, QuerySource, Table};
use indexmap::IndexMap;
use juniper::meta::{Argument, MetaType};
use juniper::{FromInputValue, GraphQLType, InputValue, LookAheadValue, Registry, ToInputValue};
use std::hash::Hash;
use std::marker::PhantomData;

pub trait UnRef<'a> {
    type UnRefed;

    fn as_ref(v: &'a Self::UnRefed) -> Self;
}

impl<'a, A> UnRef<'a> for &'a A {
    type UnRefed = A;

    fn as_ref(v: &'a Self::UnRefed) -> Self {
        v
    }
}

impl<'a, A> UnRef<'a> for (&'a A,) {
    type UnRefed = (A,);

    fn as_ref(v: &'a Self::UnRefed) -> Self {
        (&v.0,)
    }
}

impl<'a, A, B> UnRef<'a> for (&'a A, &'a B) {
    type UnRefed = (A, B);

    fn as_ref(v: &'a Self::UnRefed) -> Self {
        (&v.0, &v.1)
    }
}

impl<'a, A, B, C> UnRef<'a> for (&'a A, &'a B, &'a C) {
    type UnRefed = (A, B, C);

    fn as_ref(v: &'a Self::UnRefed) -> Self {
        (&v.0, &v.1, &v.2)
    }
}

impl<'a, A, B, C, D> UnRef<'a> for (&'a A, &'a B, &'a C, &'a D) {
    type UnRefed = (A, B, C, D);

    fn as_ref(v: &'a Self::UnRefed) -> Self {
        (&v.0, &v.1, &v.2, &v.3)
    }
}

impl<'a, A, B, C, D, E> UnRef<'a> for (&'a A, &'a B, &'a C, &'a D, &'a E) {
    type UnRefed = (A, B, C, D, E);

    fn as_ref(v: &'a Self::UnRefed) -> Self {
        (&v.0, &v.1, &v.2, &v.3, &v.4)
    }
}

pub trait PrimaryKeyInputObject<V, I> {
    fn register<'r>(registry: &mut Registry<'r>, info: &I) -> Vec<Argument<'r>>;

    fn from_input_value(value: &InputValue) -> Option<V>;
    fn from_look_ahead(look_ahead: &LookAheadValue) -> Option<V>;
    fn to_input_value(values: &V) -> InputValue;
}

impl<A, V1, I> PrimaryKeyInputObject<V1, I> for A
where
    A: Column,
    V1: GraphQLType<TypeInfo = I> + FromInputValue + ToInputValue + FromLookAheadValue,
{
    fn register<'r>(registry: &mut Registry<'r>, info: &I) -> Vec<Argument<'r>> {
        vec![registry.arg::<V1>(A::NAME, info)]
    }

    fn from_input_value(value: &InputValue) -> Option<V1> {
        V1::from_input_value(value)
    }

    fn from_look_ahead(value: &LookAheadValue) -> Option<V1> {
        if let LookAheadValue::Object(ref o) = *value {
            o.iter()
                .find(|&(ref n, _)| *n == A::NAME)
                .and_then(|(_, v)| V1::from_look_ahead(v))
        } else {
            None
        }
    }

    fn to_input_value(values: &V1) -> InputValue {
        let mut map = IndexMap::with_capacity(1);
        map.insert(A::NAME, values.to_input_value());
        InputValue::object(map)
    }
}

impl<A, V1, I> PrimaryKeyInputObject<(V1,), I> for (A,)
where
    A: Column,
    V1: GraphQLType<TypeInfo = I> + FromInputValue + ToInputValue + FromLookAheadValue,
{
    fn register<'r>(registry: &mut Registry<'r>, info: &I) -> Vec<Argument<'r>> {
        vec![registry.arg::<V1>(A::NAME, info)]
    }

    fn from_input_value(value: &InputValue) -> Option<(V1,)> {
        V1::from_input_value(value).map(|v| (v,))
    }

    fn from_look_ahead(value: &LookAheadValue) -> Option<(V1,)> {
        if let LookAheadValue::Object(ref o) = *value {
            o.iter()
                .find(|&(ref n, _)| *n == A::NAME)
                .and_then(|(_, v)| V1::from_look_ahead(v))
                .map(|v| (v,))
        } else {
            None
        }
    }

    fn to_input_value(values: &(V1,)) -> InputValue {
        let mut map = IndexMap::with_capacity(1);
        map.insert(A::NAME, values.0.to_input_value());
        InputValue::object(map)
    }
}

impl<A, B, V1, V2, I> PrimaryKeyInputObject<(V1, V2), I> for (A, B)
where
    A: Column,
    B: Column,
    V1: GraphQLType<TypeInfo = I> + FromInputValue + ToInputValue + FromLookAheadValue,
    V2: GraphQLType<TypeInfo = I> + FromInputValue + ToInputValue + FromLookAheadValue,
{
    fn register<'r>(registry: &mut Registry<'r>, info: &I) -> Vec<Argument<'r>> {
        vec![
            registry.arg::<V1>(A::NAME, info),
            registry.arg::<V2>(B::NAME, info),
        ]
    }

    fn from_input_value(value: &InputValue) -> Option<(V1, V2)> {
        V1::from_input_value(value).and_then(|v1| V2::from_input_value(value).map(|v2| (v1, v2)))
    }

    fn from_look_ahead(value: &LookAheadValue) -> Option<(V1, V2)> {
        if let LookAheadValue::Object(ref o) = *value {
            o.iter()
                .find(|&(ref n, _)| *n == A::NAME)
                .and_then(|(_, v)| V1::from_look_ahead(v))
                .and_then(|v1| {
                    o.iter()
                        .find(|&(ref n, _)| *n == B::NAME)
                        .and_then(|(_, v)| V2::from_look_ahead(v).map(|v2| (v1, v2)))
                })
        } else {
            None
        }
    }

    fn to_input_value(values: &(V1, V2)) -> InputValue {
        let mut map = IndexMap::with_capacity(2);
        map.insert(A::NAME, values.0.to_input_value());
        map.insert(B::NAME, values.1.to_input_value());
        InputValue::object(map)
    }
}

impl<A, B, C, V1, V2, V3, I> PrimaryKeyInputObject<(V1, V2, V3), I> for (A, B, C)
where
    A: Column,
    B: Column,
    C: Column,
    V1: GraphQLType<TypeInfo = I> + FromInputValue + ToInputValue + FromLookAheadValue,
    V2: GraphQLType<TypeInfo = I> + FromInputValue + ToInputValue + FromLookAheadValue,
    V3: GraphQLType<TypeInfo = I> + FromInputValue + ToInputValue + FromLookAheadValue,
{
    fn register<'r>(registry: &mut Registry<'r>, info: &I) -> Vec<Argument<'r>> {
        vec![
            registry.arg::<V1>(A::NAME, info),
            registry.arg::<V2>(B::NAME, info),
            registry.arg::<V3>(C::NAME, info),
        ]
    }

    fn from_input_value(value: &InputValue) -> Option<(V1, V2, V3)> {
        V1::from_input_value(value)
            .and_then(|v1| V2::from_input_value(value).map(|v2| (v1, v2)))
            .and_then(|(v1, v2)| V3::from_input_value(value).map(|v3| (v1, v2, v3)))
    }

    fn from_look_ahead(value: &LookAheadValue) -> Option<(V1, V2, V3)> {
        if let LookAheadValue::Object(ref o) = *value {
            o.iter()
                .find(|&(ref n, _)| *n == A::NAME)
                .and_then(|(_, v)| V1::from_look_ahead(v))
                .and_then(|v1| {
                    o.iter()
                        .find(|&(ref n, _)| *n == B::NAME)
                        .and_then(|(_, v)| V2::from_look_ahead(v).map(|v2| (v1, v2)))
                })
                .and_then(|(v1, v2)| {
                    o.iter()
                        .find(|&(ref n, _)| *n == C::NAME)
                        .and_then(|(_, v)| V3::from_look_ahead(v).map(|v3| (v1, v2, v3)))
                })
        } else {
            None
        }
    }

    fn to_input_value(values: &(V1, V2, V3)) -> InputValue {
        let mut map = IndexMap::with_capacity(3);
        map.insert(A::NAME, values.0.to_input_value());
        map.insert(B::NAME, values.1.to_input_value());
        map.insert(C::NAME, values.2.to_input_value());
        InputValue::object(map)
    }
}

impl<A, B, C, D, V1, V2, V3, V4, I> PrimaryKeyInputObject<(V1, V2, V3, V4), I> for (A, B, C, D)
where
    A: Column,
    B: Column,
    C: Column,
    D: Column,
    V1: GraphQLType<TypeInfo = I> + FromInputValue + ToInputValue + FromLookAheadValue,
    V2: GraphQLType<TypeInfo = I> + FromInputValue + ToInputValue + FromLookAheadValue,
    V3: GraphQLType<TypeInfo = I> + FromInputValue + ToInputValue + FromLookAheadValue,
    V4: GraphQLType<TypeInfo = I> + FromInputValue + ToInputValue + FromLookAheadValue,
{
    fn register<'r>(registry: &mut Registry<'r>, info: &I) -> Vec<Argument<'r>> {
        vec![
            registry.arg::<V1>(A::NAME, info),
            registry.arg::<V2>(B::NAME, info),
            registry.arg::<V3>(C::NAME, info),
            registry.arg::<V4>(D::NAME, info),
        ]
    }

    fn from_input_value(value: &InputValue) -> Option<(V1, V2, V3, V4)> {
        V1::from_input_value(value)
            .and_then(|v1| V2::from_input_value(value).map(|v2| (v1, v2)))
            .and_then(|(v1, v2)| V3::from_input_value(value).map(|v3| (v1, v2, v3)))
            .and_then(|(v1, v2, v3)| V4::from_input_value(value).map(|v4| (v1, v2, v3, v4)))
    }

    fn from_look_ahead(value: &LookAheadValue) -> Option<(V1, V2, V3, V4)> {
        if let LookAheadValue::Object(ref o) = *value {
            o.iter()
                .find(|&(ref n, _)| *n == A::NAME)
                .and_then(|(_, v)| V1::from_look_ahead(v))
                .and_then(|v1| {
                    o.iter()
                        .find(|&(ref n, _)| *n == B::NAME)
                        .and_then(|(_, v)| V2::from_look_ahead(v).map(|v2| (v1, v2)))
                })
                .and_then(|(v1, v2)| {
                    o.iter()
                        .find(|&(ref n, _)| *n == C::NAME)
                        .and_then(|(_, v)| V3::from_look_ahead(v).map(|v3| (v1, v2, v3)))
                })
                .and_then(|(v1, v2, v3)| {
                    o.iter()
                        .find(|&(ref n, _)| *n == D::NAME)
                        .and_then(|(_, v)| V4::from_look_ahead(v).map(|v4| (v1, v2, v3, v4)))
                })
        } else {
            None
        }
    }

    fn to_input_value(values: &(V1, V2, V3, V4)) -> InputValue {
        let mut map = IndexMap::with_capacity(5);
        map.insert(A::NAME, values.0.to_input_value());
        map.insert(B::NAME, values.1.to_input_value());
        map.insert(C::NAME, values.2.to_input_value());
        map.insert(D::NAME, values.3.to_input_value());
        InputValue::object(map)
    }
}

impl<A, B, C, D, E, V1, V2, V3, V4, V5, I> PrimaryKeyInputObject<(V1, V2, V3, V4, V5), I>
    for (A, B, C, D, E)
where
    A: Column,
    B: Column,
    C: Column,
    D: Column,
    E: Column,
    V1: GraphQLType<TypeInfo = I> + FromInputValue + ToInputValue + FromLookAheadValue,
    V2: GraphQLType<TypeInfo = I> + FromInputValue + ToInputValue + FromLookAheadValue,
    V3: GraphQLType<TypeInfo = I> + FromInputValue + ToInputValue + FromLookAheadValue,
    V4: GraphQLType<TypeInfo = I> + FromInputValue + ToInputValue + FromLookAheadValue,
    V5: GraphQLType<TypeInfo = I> + FromInputValue + ToInputValue + FromLookAheadValue,
{
    fn register<'r>(registry: &mut Registry<'r>, info: &I) -> Vec<Argument<'r>> {
        vec![
            registry.arg::<V1>(A::NAME, info),
            registry.arg::<V2>(B::NAME, info),
            registry.arg::<V3>(C::NAME, info),
            registry.arg::<V4>(D::NAME, info),
            registry.arg::<V5>(E::NAME, info),
        ]
    }

    fn from_input_value(value: &InputValue) -> Option<(V1, V2, V3, V4, V5)> {
        V1::from_input_value(value)
            .and_then(|v1| V2::from_input_value(value).map(|v2| (v1, v2)))
            .and_then(|(v1, v2)| V3::from_input_value(value).map(|v3| (v1, v2, v3)))
            .and_then(|(v1, v2, v3)| V4::from_input_value(value).map(|v4| (v1, v2, v3, v4)))
            .and_then(|(v1, v2, v3, v4)| V5::from_input_value(value).map(|v5| (v1, v2, v3, v4, v5)))
    }

    fn from_look_ahead(value: &LookAheadValue) -> Option<(V1, V2, V3, V4, V5)> {
        if let LookAheadValue::Object(ref o) = *value {
            o.iter()
                .find(|&(ref n, _)| *n == A::NAME)
                .and_then(|(_, v)| V1::from_look_ahead(v))
                .and_then(|v1| {
                    o.iter()
                        .find(|&(ref n, _)| *n == B::NAME)
                        .and_then(|(_, v)| V2::from_look_ahead(v).map(|v2| (v1, v2)))
                })
                .and_then(|(v1, v2)| {
                    o.iter()
                        .find(|&(ref n, _)| *n == C::NAME)
                        .and_then(|(_, v)| V3::from_look_ahead(v).map(|v3| (v1, v2, v3)))
                })
                .and_then(|(v1, v2, v3)| {
                    o.iter()
                        .find(|&(ref n, _)| *n == D::NAME)
                        .and_then(|(_, v)| V4::from_look_ahead(v).map(|v4| (v1, v2, v3, v4)))
                })
                .and_then(|(v1, v2, v3, v4)| {
                    o.iter()
                        .find(|&(ref n, _)| *n == E::NAME)
                        .and_then(|(_, v)| V5::from_look_ahead(v).map(|v5| (v1, v2, v3, v4, v5)))
                })
        } else {
            None
        }
    }

    fn to_input_value(values: &(V1, V2, V3, V4, V5)) -> InputValue {
        let mut map = IndexMap::with_capacity(5);
        map.insert(A::NAME, values.0.to_input_value());
        map.insert(B::NAME, values.1.to_input_value());
        map.insert(C::NAME, values.2.to_input_value());
        map.insert(D::NAME, values.3.to_input_value());
        map.insert(E::NAME, values.4.to_input_value());
        InputValue::object(map)
    }
}

pub struct PrimaryKeyInfo(String);

impl PrimaryKeyInfo {
    pub fn new<T>(table: &T) -> Self
    where
        T: QuerySource<FromClause = Identifier<'static>>,
    {
        PrimaryKeyInfo(format!("{}Key", table.from_clause().0))
    }
}

#[derive(Debug)]
pub struct PrimaryKeyArgument<'a, T: 'a, Ctx, V>
where
    V: UnRef<'a>,
{
    pub values: V::UnRefed,
    _marker: PhantomData<(&'a T, Ctx)>,
}

impl<'a, T, Ctx, V> GraphQLType for PrimaryKeyArgument<'a, T, Ctx, V>
where
    T: Table + 'a,
    T::PrimaryKey: PrimaryKeyInputObject<V::UnRefed, ()>,
    V: UnRef<'a>,
{
    type Context = Ctx;
    type TypeInfo = PrimaryKeyInfo;

    fn name(info: &Self::TypeInfo) -> Option<&str> {
        Some(&info.0)
    }

    fn meta<'r>(info: &Self::TypeInfo, registry: &mut Registry<'r>) -> MetaType<'r> {
        let fields = T::PrimaryKey::register(registry, &());
        registry
            .build_input_object_type::<Self>(info, &fields)
            .into_meta()
    }
}

impl<'a, T, Ctx, V> ToInputValue for PrimaryKeyArgument<'a, T, Ctx, V>
where
    T: Table,
    T::PrimaryKey: PrimaryKeyInputObject<V::UnRefed, ()>,
    V: UnRef<'a>,
{
    fn to_input_value(&self) -> InputValue {
        T::PrimaryKey::to_input_value(&self.values)
    }
}

impl<'a, T, Ctx, V> FromInputValue for PrimaryKeyArgument<'a, T, Ctx, V>
where
    T: Table,
    T::PrimaryKey: PrimaryKeyInputObject<V::UnRefed, ()>,
    V: UnRef<'a>,
{
    fn from_input_value(value: &InputValue) -> Option<Self> {
        T::PrimaryKey::from_input_value(value).map(|values| Self {
            values,
            _marker: PhantomData,
        })
    }
}

impl<'a, T, Ctx, V> FromLookAheadValue for PrimaryKeyArgument<'a, T, Ctx, V>
where
    T: Table,
    T::PrimaryKey: PrimaryKeyInputObject<V::UnRefed, ()>,
    V: UnRef<'a>,
{
    fn from_look_ahead(v: &LookAheadValue) -> Option<Self> {
        T::PrimaryKey::from_look_ahead(v).map(|values| Self {
            values,
            _marker: PhantomData,
        })
    }
}

impl<'a, T, Ctx, V> HasTable for PrimaryKeyArgument<'a, T, Ctx, V>
where
    T: Table + HasTable<Table = T>,
    V: UnRef<'a>,
{
    type Table = T;

    fn table() -> Self::Table {
        T::table()
    }
}

impl<'a, T, Ctx, V> Identifiable for &'a PrimaryKeyArgument<'a, T, Ctx, V>
where
    Self: HasTable,
    V: UnRef<'a> + Hash + Eq,
{
    type Id = V;

    fn id(self) -> Self::Id {
        V::as_ref(&self.values)
    }
}
