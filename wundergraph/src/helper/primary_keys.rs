use super::FromLookAheadValue;
use diesel::associations::HasTable;
use diesel::query_builder::nodes::Identifier;
use diesel::{Column, Identifiable, QuerySource, Table};
use indexmap::IndexMap;
use juniper::meta::{Argument, MetaType};
use juniper::{FromInputValue, GraphQLType, InputValue, LookAheadValue, Registry, ToInputValue};
use scalar::WundergraphScalarValue;
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

macro_rules! unref_impl {
    ($(
        $Tuple:tt {
            $(($idx: tt) -> $T:ident, $ST: ident, $TT: ident,)+
        }
    )+) => {
        $(
            impl<'a, $($T,)+> UnRef<'a> for ($(&'a $T,)+) {
                type UnRefed = ($($T, )+);

                fn as_ref(v: &'a Self::UnRefed) -> Self {
                    ($(&v.$idx,)+)
                }
            }
        )+
    }
}

__diesel_for_each_tuple!(unref_impl);

pub trait PrimaryKeyInputObject<V, I> {
    fn register<'r>(
        registry: &mut Registry<'r, WundergraphScalarValue>,
        info: &I,
    ) -> Vec<Argument<'r, WundergraphScalarValue>>;

    fn from_input_value(value: &InputValue<WundergraphScalarValue>) -> Option<V>;
    fn from_look_ahead(look_ahead: &LookAheadValue<WundergraphScalarValue>) -> Option<V>;
    fn to_input_value(values: &V) -> InputValue<WundergraphScalarValue>;
}

impl<A, V1, I> PrimaryKeyInputObject<V1, I> for A
where
    A: Column,
    V1: GraphQLType<WundergraphScalarValue, TypeInfo = I>
        + FromInputValue<WundergraphScalarValue>
        + ToInputValue<WundergraphScalarValue>
        + FromLookAheadValue,
{
    fn register<'r>(
        registry: &mut Registry<'r, WundergraphScalarValue>,
        info: &I,
    ) -> Vec<Argument<'r, WundergraphScalarValue>> {
        vec![registry.arg::<V1>(Self::NAME, info)]
    }

    fn from_input_value(value: &InputValue<WundergraphScalarValue>) -> Option<V1> {
        V1::from_input_value(value)
    }

    fn from_look_ahead(value: &LookAheadValue<WundergraphScalarValue>) -> Option<V1> {
        if let LookAheadValue::Object(ref o) = *value {
            o.iter()
                .find(|&(n, _)| *n == Self::NAME)
                .and_then(|(_, v)| V1::from_look_ahead(v))
        } else {
            None
        }
    }

    fn to_input_value(values: &V1) -> InputValue<WundergraphScalarValue> {
        let mut map = IndexMap::with_capacity(1);
        map.insert(Self::NAME, values.to_input_value());
        InputValue::object(map)
    }
}

macro_rules! primary_key_input_object_impl {
    ($(
        $Tuple:tt {
            $(($idx: tt) -> $T:ident, $ST: ident, $TT: ident,)+
        }
    )+) => {
        $(
            impl<$($T,)+ $($ST,)+ __I> PrimaryKeyInputObject<($($ST,)+), __I> for ($($T,)+)
            where
                $($T: Column,)+
                $($ST: GraphQLType<WundergraphScalarValue, TypeInfo = __I>
                  + FromInputValue<WundergraphScalarValue>
                  + ToInputValue<WundergraphScalarValue>
                  + FromLookAheadValue,)+
            {
                fn register<'r>(
                    registry: &mut Registry<'r, WundergraphScalarValue>,
                    info: &__I
                ) -> Vec<Argument<'r, WundergraphScalarValue>> {
                    vec![$(
                        registry.arg::<$ST>($T::NAME, info),
                    )+]
                }

                fn from_input_value(value: &InputValue<WundergraphScalarValue>)
                    -> Option<($($ST,)+)>
                {
                    $(
                        let $ST = $ST::from_input_value(value)?;
                    ) +
                    Some(($($ST,)+))
                }

                fn from_look_ahead(value: &LookAheadValue<WundergraphScalarValue>)
                    -> Option<($($ST, )+)>
                {
                    if let LookAheadValue::Object(ref o) = *value {
                        $(
                            let $ST = o.iter().find(|&(ref n, _)| *n == $T::NAME)
                                .and_then(|(_, v)| $ST::from_look_ahead(v))?;
                        ) +
                        Some(($($ST,)+))
                    } else {
                        None
                    }
                }

                fn to_input_value(values: &($($ST, )+)) -> InputValue<WundergraphScalarValue> {
                    let mut map = IndexMap::with_capacity($Tuple);
                    $(
                        map.insert($T::NAME, values.$idx.to_input_value());
                    )+
                    InputValue::object(map)
                }
            }
        )+
    }
}

__diesel_for_each_tuple!(primary_key_input_object_impl);

#[derive(Debug)]
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

impl<'a, T, Ctx, V> GraphQLType<WundergraphScalarValue> for PrimaryKeyArgument<'a, T, Ctx, V>
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

    fn meta<'r>(
        info: &Self::TypeInfo,
        registry: &mut Registry<'r, WundergraphScalarValue>,
    ) -> MetaType<'r, WundergraphScalarValue>
    where
        WundergraphScalarValue: 'r,
    {
        let fields = T::PrimaryKey::register(registry, &());
        registry
            .build_input_object_type::<Self>(info, &fields)
            .into_meta()
    }
}

impl<'a, T, Ctx, V> ToInputValue<WundergraphScalarValue> for PrimaryKeyArgument<'a, T, Ctx, V>
where
    T: Table,
    T::PrimaryKey: PrimaryKeyInputObject<V::UnRefed, ()>,
    V: UnRef<'a>,
{
    fn to_input_value(&self) -> InputValue<WundergraphScalarValue> {
        T::PrimaryKey::to_input_value(&self.values)
    }
}

impl<'a, T, Ctx, V> FromInputValue<WundergraphScalarValue> for PrimaryKeyArgument<'a, T, Ctx, V>
where
    T: Table,
    T::PrimaryKey: PrimaryKeyInputObject<V::UnRefed, ()>,
    V: UnRef<'a>,
{
    fn from_input_value(value: &InputValue<WundergraphScalarValue>) -> Option<Self> {
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
    fn from_look_ahead(v: &LookAheadValue<WundergraphScalarValue>) -> Option<Self> {
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
