use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql, FromSqlRow, Queryable};
use diesel::row::Row;
use diesel::sql_types::{NotNull, Nullable};

pub trait PlaceHolderMarker {
    type InnerType;

    fn into_inner(self) -> Option<Self::InnerType>;
}

/// A wrapper type used inside of wundergraph to load values of the type T
/// from the database
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
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

impl<ST, T, DB> FromSqlRow<Nullable<ST>, DB> for PlaceHolder<T>
where
    Option<T>: FromSqlRow<Nullable<ST>, DB>,
    DB: Backend,
    ST: NotNull,
{
    const FIELDS_NEEDED: usize = <Option<T> as FromSqlRow<Nullable<ST>, DB>>::FIELDS_NEEDED;

    fn build_from_row<R: Row<DB>>(row: &mut R) -> deserialize::Result<Self> {
        Option::build_from_row(row).map(PlaceHolder)
    }
}

impl<ST, T, DB> Queryable<Nullable<ST>, DB> for PlaceHolder<T>
where
    Option<T>: Queryable<Nullable<ST>, DB>,
    DB: Backend,
    ST: NotNull,
{
    type Row = <Option<T> as Queryable<Nullable<ST>, DB>>::Row;

    fn build(row: Self::Row) -> Self {
        PlaceHolder(Option::build(row))
    }
}
