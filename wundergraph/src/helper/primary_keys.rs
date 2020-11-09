use crate::juniper_ext::FromLookAheadValue;
use crate::query_builder::types::as_input_type::AsInputType;
use crate::scalar::WundergraphScalarValue;
use diesel::associations::HasTable;
use diesel::query_builder::nodes::Identifier;
use diesel::{Column, Identifiable, QuerySource, Table};
use indexmap::IndexMap;
use juniper::meta::{Argument, MetaType};
use juniper::{FromInputValue, GraphQLType, InputValue, LookAheadValue, Registry, ToInputValue};
use std::borrow::Cow;
use std::fmt::{self, Debug};
use std::hash::Hash;
use std::marker::PhantomData;

/// A helper trait to transform a tuple of references into a
/// tuple of owned values at type system level
pub trait UnRef<'a> {
    /// The owned tuple type
    type UnRefed;

    /// Do the inverse transformation, which means construct a tuple of references
    /// from a given reference to a owned tuple
    fn as_ref(v: &'a Self::UnRefed) -> Self;
}

/// A helper trait to transform a tuple of references into a tuple of
/// owned values at type system level
pub trait UnRefClone {
    /// The owned tuple type
    type UnRefed;

    /// Construct a owned tuple from a tuple of references
    fn make_owned(self) -> Self::UnRefed;
}

impl<'a, A> UnRef<'a> for &'a A {
    type UnRefed = A;

    fn as_ref(v: &'a Self::UnRefed) -> Self {
        v
    }
}

impl<'a, A> UnRefClone for &'a A
where
    A: Clone + 'static,
{
    type UnRefed = A;

    fn make_owned(self) -> A {
        self.clone()
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

            impl<'a, $($T,)+> UnRefClone for ($(&'a $T,)+)
                where $($T: Clone + 'static,)*
            {

                type UnRefed = ($($T, )+);

                fn make_owned(self) -> Self::UnRefed
                {
                    ($((*self.$idx).clone(),)*)
                }
            }
        )+
    }
}

__diesel_for_each_tuple!(unref_impl);

#[doc(hidden)]
pub trait PrimaryKeyInputObject<V> {
    fn register<'r>(
        registry: &mut Registry<'r, WundergraphScalarValue>,
    ) -> Vec<Argument<'r, WundergraphScalarValue>>;

    fn from_input_value(value: &InputValue<WundergraphScalarValue>) -> Option<V>;
    fn from_look_ahead(look_ahead: &LookAheadValue<'_, WundergraphScalarValue>) -> Option<V>;
    fn to_input_value(values: &V) -> InputValue<WundergraphScalarValue>;
}

impl<A, V1> PrimaryKeyInputObject<V1> for A
where
    A: Column,
    V1: GraphQLType<WundergraphScalarValue>
        + FromInputValue<WundergraphScalarValue>
        + ToInputValue<WundergraphScalarValue>
        + FromLookAheadValue,
    V1::TypeInfo: Default,
{
    fn register<'r>(
        registry: &mut Registry<'r, WundergraphScalarValue>,
    ) -> Vec<Argument<'r, WundergraphScalarValue>> {
        vec![registry.arg::<V1>(Self::NAME, &Default::default())]
    }

    fn from_input_value(value: &InputValue<WundergraphScalarValue>) -> Option<V1> {
        if let InputValue::Object(ref o) = *value {
            o.iter()
                .find(|&(n, _)| n.item == Self::NAME)
                .and_then(|(_, v)| V1::from_input_value(&v.item))
        } else {
            None
        }
    }

    fn from_look_ahead(value: &LookAheadValue<'_, WundergraphScalarValue>) -> Option<V1> {
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
            impl<$($T,)+ $($ST,)+ > PrimaryKeyInputObject<($($ST,)+)> for ($($T,)+)
            where
                $($T: Column,)+
                $($T: PrimaryKeyInputObject<$ST>,)+
            {
                fn register<'r>(
                    registry: &mut Registry<'r, WundergraphScalarValue>,
                ) -> Vec<Argument<'r, WundergraphScalarValue>> {
                    let mut ret = Vec::new();
                    $(
                        ret.extend($T::register(registry));
                    )*
                    ret
                }

                fn from_input_value(value: &InputValue<WundergraphScalarValue>)
                    -> Option<($($ST,)+)>
                {
                    if let InputValue::Object(ref _o) = *value {
                        Some(($(
                            $T::from_input_value(value)?,
                        )*))
                    } else {
                        None
                    }
                }

                fn from_look_ahead(value: &LookAheadValue<'_, WundergraphScalarValue>)
                    -> Option<($($ST, )+)>
                {
                    if let LookAheadValue::Object(ref _o) = *value {
                        Some(($(
                            $T::from_look_ahead(value)?,
                        )*))
                    } else {
                        None
                    }
                }

                fn to_input_value(values: &($($ST, )+)) -> InputValue<WundergraphScalarValue> {
                   let mut map = IndexMap::with_capacity($Tuple);
                   $(
                       map.insert($T::NAME, $T::to_input_value(&values.$idx));
                   )+
                   InputValue::object(map)
                }
            }
        )+
    }
}

__diesel_for_each_tuple!(primary_key_input_object_impl);

/// A marker trait for query sources with a name
pub trait NamedTable {
    /// The name of query source
    fn name(&self) -> Cow<'static, str>;
}

impl NamedTable for Identifier<'static> {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed(self.0)
    }
}

#[cfg(feature = "postgres")]
impl NamedTable
    for diesel::query_builder::nodes::InfixNode<'static, Identifier<'static>, Identifier<'static>>
{
    fn name(&self) -> Cow<'static, str> {
        // Find a better way …
        Cow::Owned(
            diesel::debug_query::<diesel::pg::Pg, _>(self)
                .to_string()
                .split("--")
                .next()
                .expect("It's there otherwise diesel changed the debug output")
                .replace("\"", "")
                .replace(".", "_"),
        )
    }
}

#[derive(Debug)]
#[doc(hidden)]
pub struct PrimaryKeyInfo<T>(String, PhantomData<T>);

impl<T> Default for PrimaryKeyInfo<T>
where
    T: HasTable<Table = T> + Table + QuerySource,
    T::FromClause: NamedTable,
{
    fn default() -> Self {
        Self(
            format!(
                "{}PrimaryKey",
                uppercase_first_letter(T::table().from_clause().name())
            ),
            PhantomData,
        )
    }
}

#[allow(clippy::needless_pass_by_value)]
fn uppercase_first_letter(s: Cow<'_, str>) -> Cow<'_, str> {
    if s.chars().next().map_or(true, |c| c.is_uppercase()) {
        Cow::Owned(s.trim().to_string())
    } else {
        Cow::Owned(
            s.chars()
                .take(1)
                .flat_map(|c| c.to_uppercase())
                .chain(s.chars().skip(1))
                .collect::<String>()
                .trim()
                .to_string(),
        )
    }
}

#[doc(hidden)]
pub struct PrimaryKeyArgument<'a, T, Ctx, V>
where
    V: UnRef<'a>,
    V::UnRefed: AsInputType,
{
    pub(crate) values: <V::UnRefed as AsInputType>::InputType,
    _marker: PhantomData<(&'a T, Ctx)>,
}

impl<'a, T, Ctx, V> Debug for PrimaryKeyArgument<'a, T, Ctx, V>
where
    V: UnRef<'a>,
    V::UnRefed: AsInputType,
    <V::UnRefed as AsInputType>::InputType: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PrimaryKeyArgument")
            .field("primary_key", &self.values)
            .finish()
    }
}

impl<'a, T, Ctx, V> GraphQLType<WundergraphScalarValue> for PrimaryKeyArgument<'a, T, Ctx, V>
where
    T: Table + 'a,
    T::PrimaryKey: PrimaryKeyInputObject<<V::UnRefed as AsInputType>::InputType>,
    V: UnRef<'a>,
    V::UnRefed: AsInputType,
{
    type Context = Ctx;
    type TypeInfo = PrimaryKeyInfo<T>;

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
        let fields = T::PrimaryKey::register(registry);
        registry
            .build_input_object_type::<Self>(info, &fields)
            .into_meta()
    }
}

impl<'a, T, Ctx, V> ToInputValue<WundergraphScalarValue> for PrimaryKeyArgument<'a, T, Ctx, V>
where
    T: Table,
    T::PrimaryKey: PrimaryKeyInputObject<<V::UnRefed as AsInputType>::InputType>,
    V: UnRef<'a>,
    V::UnRefed: AsInputType,
{
    fn to_input_value(&self) -> InputValue<WundergraphScalarValue> {
        T::PrimaryKey::to_input_value(&self.values)
    }
}

impl<'a, T, Ctx, V> FromInputValue<WundergraphScalarValue> for PrimaryKeyArgument<'a, T, Ctx, V>
where
    T: Table,
    T::PrimaryKey: PrimaryKeyInputObject<<V::UnRefed as AsInputType>::InputType>,
    V: UnRef<'a>,
    V::UnRefed: AsInputType,
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
    T::PrimaryKey: PrimaryKeyInputObject<<V::UnRefed as AsInputType>::InputType>,
    V: UnRef<'a>,
    V::UnRefed: AsInputType,
{
    fn from_look_ahead(v: &LookAheadValue<'_, WundergraphScalarValue>) -> Option<Self> {
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
    V::UnRefed: AsInputType,
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
    V::UnRefed: AsInputType,
{
    type Id = V;

    fn id(self) -> Self::Id {
        todo!()
    }
}
