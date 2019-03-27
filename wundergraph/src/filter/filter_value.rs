use crate::filter::nullable_filter::NullableFilter;
use crate::filter::string_filter::StringFilter;
use crate::helper::FromLookAheadValue;
use crate::scalar::WundergraphScalarValue;
use juniper::{FromInputValue, ToInputValue};

/// A fundamental trait marking that a filter could be constructed for a given type
///
/// The generic parameter `C` represents the type of the column (from diesel)
/// For most implementations this should just be generic over all compatible columns
pub trait FilterValue<C> {
    /// The raw value type
    ///
    /// Normally this is the same as `Self` but there are cases like `Option<T>`
    /// where we want something other (`T`) as input for our filter operations
    type RawValue: Clone
        + FromInputValue<WundergraphScalarValue>
        + FromLookAheadValue
        + ToInputValue<WundergraphScalarValue>;
    /// A type describing possible additional filters
    ///
    /// For some cases a type supports more operations that just the default set
    /// (eq, neq, gt, lt, …). In such cases a type representing those additional
    /// operations could be specified here.
    /// If there are no additional operations just use `()`
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
