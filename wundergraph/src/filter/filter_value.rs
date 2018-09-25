use filter::nullable_filter::NullableFilter;
use filter::string_filter::StringFilter;
use helper::FromLookAheadValue;
use juniper::{FromInputValue, ToInputValue};
use scalar::WundergraphScalarValue;

pub trait FilterValue<C> {
    type RawValue: Clone
        + FromInputValue<WundergraphScalarValue>
        + FromLookAheadValue
        + ToInputValue<WundergraphScalarValue>;
    type AdditionalFilter;
}

impl<C> FilterValue<C> for i16 {
    type RawValue = Self;
    type AdditionalFilter = ();
}

impl<C> FilterValue<C> for i32 {
    type RawValue = Self;
    type AdditionalFilter = ();
}

impl<C> FilterValue<C> for i64 {
    type RawValue = Self;
    type AdditionalFilter = ();
}

impl<C> FilterValue<C> for String {
    type RawValue = Self;
    type AdditionalFilter = StringFilter<C>;
}

impl<C> FilterValue<C> for bool {
    type RawValue = Self;
    type AdditionalFilter = ();
}

impl<C> FilterValue<C> for f32 {
    type RawValue = Self;
    type AdditionalFilter = ();
}

impl<C> FilterValue<C> for f64 {
    type RawValue = Self;
    type AdditionalFilter = ();
}

impl<C, V> FilterValue<C> for Vec<V>
where
    V: FromLookAheadValue
       + FromInputValue<WundergraphScalarValue>
       + ToInputValue<WundergraphScalarValue>
       + FilterValue<C>
       + Clone,
{
    type RawValue = Self;
    type AdditionalFilter = ();
}

impl<V, C> FilterValue<C> for Option<V>
where
    V: Clone
    + FromInputValue<WundergraphScalarValue>
    + FromLookAheadValue
    + ToInputValue<WundergraphScalarValue>
    + FilterValue<C>,
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
impl<O, C> FilterValue<C> for self::chrono::DateTime<O>
where
    O: self::chrono::TimeZone,
    Self: ToInputValue<WundergraphScalarValue>
    + FromInputValue<WundergraphScalarValue>
    + FromLookAheadValue,
{
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
