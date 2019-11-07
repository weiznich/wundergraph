use super::{HasOne, PlaceHolder};
use diesel::sql_types::{BigInt, Bool, Float4, Float8, Integer, Nullable, SmallInt, Text};
use diesel::Identifiable;
use std::hash::Hash;

pub use wundergraph_derive::WundergraphValue;

/// A marker trait indicating that a type could be used with wundergraph
///
/// # Deriving
/// This trait could be derived by using the [`#[derive(WundergraphValue)]`](derive.WundergraphValue.html)
/// custom derive
///
/// ## Example
/// The following example implements support for a enum type mapping to
/// a `SmallInt` at sql side
///
/// ```rust
/// #[macro_use] extern crate diesel;
/// use juniper::GraphQLEnum;
/// use wundergraph::query_builder::types::WundergraphValue;
/// use diesel::serialize::{self, ToSql};
/// use diesel::deserialize::{self, FromSql};
/// use diesel::backend::Backend;
/// use diesel::sql_types::SmallInt;
/// use std::io::Write;
///
/// #[derive(
///     Debug, Copy, Clone, AsExpression, FromSqlRow, GraphQLEnum, WundergraphValue
/// )]
/// #[sql_type = "SmallInt"]
/// pub enum Episode {
///     NEWHOPE = 1,
///     EMPIRE = 2,
///     JEDI = 3,
/// }
///
/// impl<DB> ToSql<SmallInt, DB> for Episode
/// where
///     DB: Backend,
///     i16: ToSql<SmallInt, DB>,
/// {
///     fn to_sql<W: Write>(&self, out: &mut serialize::Output<'_, W, DB>) -> serialize::Result {
///         (*self as i16).to_sql(out)
///     }
/// }
///
/// impl<DB> FromSql<SmallInt, DB> for Episode
/// where
///     DB: Backend,
///     i16: FromSql<SmallInt, DB>,
/// {
///     fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
///         let value = i16::from_sql(bytes)?;
///         Ok(match value {
///             1 => Episode::NEWHOPE,
///             2 => Episode::EMPIRE,
///             3 => Episode::JEDI,
///             _ => unreachable!(),
///         })
///     }
/// }
/// # fn main() {}
/// ```
///
/// # Manual implementation
///
/// Below a version with an expanded `#[derive(WundergraphValue)]` custom derive
/// is shown.
/// ```rust
/// #[macro_use] extern crate diesel;
/// use juniper::{GraphQLEnum, LookAheadValue};
/// use wundergraph::query_builder::types::{WundergraphValue, PlaceHolder};
/// use wundergraph::query_builder::selection::filter::{AsColumnFilter, FilterOption, FilterValue};
/// use wundergraph::juniper_ext::FromLookAheadValue;
/// use wundergraph::scalar::WundergraphScalarValue;
/// # use diesel::serialize::{self, ToSql};
/// # use diesel::deserialize::{self, FromSql};
/// # use diesel::backend::Backend;
/// use diesel::sql_types::{SmallInt, Nullable};
/// use std::io::Write;
///
///
/// #[derive(
///     Debug, Copy, Clone, AsExpression, FromSqlRow, GraphQLEnum
/// )]
/// #[sql_type = "SmallInt"]
/// pub enum Episode {
///     NEWHOPE = 1,
///     EMPIRE = 2,
///     JEDI = 3,
/// }
///
/// impl WundergraphValue for Episode {
///     type PlaceHolder = PlaceHolder<Self>;
///     type SqlType = Nullable<SmallInt>;
/// }
///
/// impl<C, DB, Ctx> AsColumnFilter<C, DB, Ctx> for Episode {
///     type Filter = FilterOption<Self, C>;
/// }
///
/// impl FromLookAheadValue for Episode {
///     fn from_look_ahead(v: &LookAheadValue<WundergraphScalarValue>) -> Option<Self> {
///         if let LookAheadValue::Enum(ref e) = *v {
///             match *e {
///                 "NEWHOPE" => Some(Episode::NEWHOPE),
///                 "EMIPRE" => Some(Episode::EMPIRE),
///                 "JEDI" => Some(Episode::JEDI),
///                 _ => None,
///             }
///         } else {
///             None
///         }
///     }
/// }
///
/// impl<C> FilterValue<C> for Episode {
///     type RawValue = Episode;
///     type AdditionalFilter = ();
/// }
/// #
/// # impl<DB> ToSql<SmallInt, DB> for Episode
/// # where
/// #    DB: Backend,
/// #    i16: ToSql<SmallInt, DB>,
/// # {
/// #    fn to_sql<W: Write>(&self, out: &mut serialize::Output<'_, W, DB>) -> serialize::Result {
/// #        (*self as i16).to_sql(out)
/// #    }
/// # }
/// #
/// # impl<DB> FromSql<SmallInt, DB> for Episode
/// # where
/// #     DB: Backend,
/// #     i16: FromSql<SmallInt, DB>,
/// # {
/// #     fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
/// #         let value = i16::from_sql(bytes)?;
/// #         Ok(match value {
/// #             1 => Episode::NEWHOPE,
/// #             2 => Episode::EMPIRE,
/// #             3 => Episode::JEDI,
/// #             _ => unreachable!(),
/// #         })
/// #     }
/// # }
/// # fn main() {}
/// ```
pub trait WundergraphValue {
    /// A type used to load values of the specified sql type into
    ///
    /// For common cases this should be `PlaceHolder<Self>`
    type PlaceHolder: 'static;
    /// The corresponding diesel sql type
    ///
    /// Normally this is some nullable type
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

#[cfg(feature = "postgres")]
impl<T, Inner> WundergraphValue for Vec<T>
where
    T: WundergraphValue<SqlType = Nullable<Inner>> + 'static,
    Inner: diesel::sql_types::NotNull + 'static,
{
    type PlaceHolder = PlaceHolder<Self>;
    type SqlType = Nullable<diesel::sql_types::Array<Inner>>;
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
