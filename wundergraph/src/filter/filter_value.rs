use juniper::{FromInputValue, ToInputValue};
use helper::FromLookAheadValue;
use filter::string_filter::StringFilter;
use filter::nullable_filter::NullableFilter;

pub trait FilterValue<C> {
    type RawValue: Clone + FromInputValue + FromLookAheadValue + ToInputValue;
    type AdditionalFilter;
}

impl<C> FilterValue<C> for i32 {
    type RawValue = i32;
    type AdditionalFilter = ();
}

impl<C> FilterValue<C> for String {
    type RawValue = String;
    type AdditionalFilter = StringFilter<C>;
}

impl<C> FilterValue<C> for bool {
    type RawValue = bool;
    type AdditionalFilter = ();
}

impl<C> FilterValue<C> for f64 {
    type RawValue = f64;
    type AdditionalFilter = ();
}

impl<V, C> FilterValue<C> for Option<V>
where
    V: Clone + FromInputValue + FromLookAheadValue + ToInputValue + FilterValue<C>,
{
    type RawValue = V;
    type AdditionalFilter = NullableFilter<V, C>;
}

#[cfg(feature = "chrono")]
extern crate chrono;

#[cfg(feature = "chrono")]
impl<C> FilterValue<C> for self::chrono::NaiveDateTime {
    type RawValue = Self;
    type AdditionalFilter = ();
}

#[cfg(feature = "chrono")]
impl<C> FilterValue<C> for self::chrono::NaiveDate {
    type RawValue = Self;
    type AdditionalFilter = ();
}

#[cfg(feature = "uuid")]
extern crate uuid;

#[cfg(feature = "uuid")]
impl<C> FilterValue<C> for self::uuid::Uuid {
    type RawValue = Self;
    type AdditionalFilter = ();
}
