use juniper::{FromInputValue, ToInputValue};
use helper::FromLookAheadValue;
use filter::string_filter::StringFilter;
use filter::nullable_filter::NullableFilter;

pub trait FilterValue<C, DB> {
    type RawValue: Clone + FromInputValue + FromLookAheadValue + ToInputValue;
    type AdditionalFilter;
}

impl<C, DB> FilterValue<C, DB> for i32 {
    type RawValue = i32;
    type AdditionalFilter = ();
}

impl<C, DB> FilterValue<C, DB> for String {
    type RawValue = String;
    type AdditionalFilter = StringFilter<C, DB>;
}

impl<C, DB> FilterValue<C, DB> for bool {
    type RawValue = bool;
    type AdditionalFilter = ();
}

impl<C, DB> FilterValue<C, DB> for f64 {
    type RawValue = f64;
    type AdditionalFilter = ();
}

impl<V, C, DB> FilterValue<C, DB> for Option<V>
where
    V: Clone + FromInputValue + FromLookAheadValue + ToInputValue + FilterValue<C, DB>,
{
    type RawValue = V;
    type AdditionalFilter = NullableFilter<V, C, DB>;
}

#[cfg(feature = "chrono")]
extern crate chrono;

#[cfg(feature = "chrono")]
impl<C, DB> FilterValue<C, DB> for self::chrono::NaiveDateTime {
    type RawValue = Self;
    type AdditionalFilter = ();
}

#[cfg(feature = "chrono")]
impl<C, DB> FilterValue<C, DB> for self::chrono::NaiveDate {
    type RawValue = Self;
    type AdditionalFilter = ();
}

#[cfg(feature = "uuid")]
extern crate uuid;

#[cfg(feature = "uuid")]
impl<C, DB> FilterValue<C, DB> for self::uuid::Uuid {
    type RawValue = Self;
    type AdditionalFilter = ();
}
