use super::{HasOne, PlaceHolder};
use diesel::sql_types::{
    Array, BigInt, Bool, Float4, Float8, Integer, NotNull, Nullable, SmallInt, Text,
};
use diesel::Identifiable;
use std::hash::Hash;

pub trait WundergraphValue {
    type PlaceHolder: 'static;
    type SqlType: 'static;
}

impl WundergraphValue for i16 {
    type PlaceHolder = PlaceHolder<Self>;
    type SqlType = Nullable<SmallInt>;
}

impl WundergraphValue for i32 {
    type PlaceHolder = PlaceHolder<Self>;
    type SqlType = Nullable<Integer>;
}

impl WundergraphValue for i64 {
    type PlaceHolder = PlaceHolder<Self>;
    type SqlType = Nullable<BigInt>;
}

impl WundergraphValue for bool {
    type PlaceHolder = PlaceHolder<Self>;
    type SqlType = Nullable<Bool>;
}

impl WundergraphValue for String {
    type PlaceHolder = PlaceHolder<Self>;
    type SqlType = Nullable<Text>;
}

impl WundergraphValue for f32 {
    type PlaceHolder = PlaceHolder<Self>;
    type SqlType = Nullable<Float4>;
}

impl WundergraphValue for f64 {
    type PlaceHolder = PlaceHolder<Self>;
    type SqlType = Nullable<Float8>;
}

impl<T, Inner> WundergraphValue for Vec<T>
where
    T: WundergraphValue<SqlType = Nullable<Inner>> + 'static,
    Inner: NotNull + 'static,
{
    type PlaceHolder = PlaceHolder<Self>;
    type SqlType = Nullable<Array<Inner>>;
}

impl<T> WundergraphValue for Option<T>
where
    T: WundergraphValue,
{
    type PlaceHolder = T::PlaceHolder;
    type SqlType = T::SqlType;
}

impl<R, T> WundergraphValue for HasOne<R, T>
where
    R: WundergraphValue + Clone + Eq + Hash,
    for<'a> &'a T: Identifiable<Id = &'a R>,
{
    type PlaceHolder = R::PlaceHolder;
    type SqlType = R::SqlType;
}


macro_rules! wundergraph_value_impl {
    ($(
        $Tuple:tt {
            $(($idx:tt) -> $T:ident, $ST: ident, $TT: ident,) +
        }
    )+) => {
        $(
            impl<$($T,)+> WundergraphValue for ($($T,)+)
                where $($T: WundergraphValue,)+
            {
                type PlaceHolder = ($($T::PlaceHolder,)+);
                type SqlType = ($($T::SqlType,)+);
            }
        )*
    }
}

__diesel_for_each_tuple!(wundergraph_value_impl);
