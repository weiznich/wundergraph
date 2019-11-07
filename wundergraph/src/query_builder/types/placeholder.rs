use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::sql_types::{NotNull, Nullable};

pub trait PlaceHolderMarker {
    type InnerType;

    fn into_inner(self) -> Option<Self::InnerType>;
}

/// A wrapper type used inside of wundergraph to load values of the type T
/// from the database
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, FromSqlRow, Hash)]
pub struct PlaceHolder<T>(Option<T>);

impl<T> PlaceHolderMarker for PlaceHolder<T> {
    type InnerType = T;

    fn into_inner(self) -> Option<T> {
        self.0
    }
}

impl<T> Default for PlaceHolder<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T> Into<Option<T>> for PlaceHolder<T> {
    fn into(self) -> Option<T> {
        self.0
    }
}

impl<T> Into<Option<Option<T>>> for PlaceHolder<T> {
    fn into(self) -> Option<Option<T>> {
        Some(self.0)
    }
}

impl<'a, T> Into<Option<&'a T>> for &'a PlaceHolder<T> {
    fn into(self) -> Option<&'a T> {
        self.0.as_ref()
    }
}

impl<ST, T, DB> FromSql<Nullable<ST>, DB> for PlaceHolder<T>
where
    DB: Backend,
    T: FromSql<ST, DB>,
    ST: NotNull,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        if bytes.is_some() {
            T::from_sql(bytes).map(Some).map(Self)
        } else {
            Ok(Self(None))
        }
    }
}
