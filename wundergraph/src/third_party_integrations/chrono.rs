use crate::juniper_ext::{FromLookAheadValue, Nameable};
use crate::query_builder::selection::filter::filter_helper::AsColumnFilter;
use crate::query_builder::selection::filter::filter_value::FilterValue;
use crate::query_builder::selection::filter::FilterOption;
use crate::query_builder::types::{PlaceHolder, WundergraphValue};
use crate::scalar::WundergraphScalarValue;
use chrono_internal::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, TimeZone, Utc};
use diesel::sql_types::{Date, Nullable, Timestamp};
use juniper::{FromInputValue, LookAheadValue, ToInputValue};

impl From<NaiveDateTime> for WundergraphScalarValue {
    fn from(n: NaiveDateTime) -> Self {
        WundergraphScalarValue::Double(n.timestamp() as _)
    }
}

impl Nameable for NaiveDateTime {
    fn name() -> String {
        String::from("NaiveDateTime")
    }
}

impl<O> Nameable for DateTime<O>
where
    O: TimeZone,
{
    fn name() -> String {
        String::from("DateTime")
    }
}
impl Nameable for NaiveDate {
    fn name() -> String {
        String::from("Date")
    }
}

static RFC3339_PARSE_FORMAT: &str = "%+";
static RFC3339_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.f%:z";

impl FromLookAheadValue for NaiveDateTime {
    fn from_look_ahead(v: &LookAheadValue<'_, WundergraphScalarValue>) -> Option<Self> {
        if let LookAheadValue::Scalar(WundergraphScalarValue::String(ref s)) = *v {
            Self::parse_from_str(s, RFC3339_PARSE_FORMAT).ok()
        } else {
            None
        }
    }
}

impl FromLookAheadValue for DateTime<Utc> {
    fn from_look_ahead(v: &LookAheadValue<'_, WundergraphScalarValue>) -> Option<Self> {
        if let LookAheadValue::Scalar(WundergraphScalarValue::String(ref s)) = *v {
            s.parse().ok()
        } else {
            None
        }
    }
}

impl FromLookAheadValue for DateTime<FixedOffset> {
    fn from_look_ahead(v: &LookAheadValue<'_, WundergraphScalarValue>) -> Option<Self> {
        if let LookAheadValue::Scalar(WundergraphScalarValue::String(ref s)) = *v {
            Self::parse_from_rfc3339(s).ok()
        } else {
            None
        }
    }
}

impl FromLookAheadValue for NaiveDate {
    fn from_look_ahead(v: &LookAheadValue<'_, WundergraphScalarValue>) -> Option<Self> {
        if let LookAheadValue::Scalar(WundergraphScalarValue::String(ref s)) = *v {
            Self::parse_from_str(s, RFC3339_FORMAT).ok()
        } else {
            None
        }
    }
}

impl WundergraphValue for NaiveDateTime {
    type PlaceHolder = PlaceHolder<Self>;
    type SqlType = Nullable<Timestamp>;
}

#[cfg(feature = "postgres")]
impl WundergraphValue for DateTime<Utc> {
    type PlaceHolder = PlaceHolder<Self>;
    type SqlType = Nullable<diesel::sql_types::Timestamptz>;
}

impl WundergraphValue for NaiveDate {
    type PlaceHolder = PlaceHolder<Self>;
    type SqlType = Nullable<Date>;
}

impl<C> FilterValue<C> for NaiveDateTime {
    type RawValue = Self;
    type AdditionalFilter = ();
}

impl<O, C> FilterValue<C> for DateTime<O>
where
    O: TimeZone,
    Self: ToInputValue<WundergraphScalarValue>
        + FromInputValue<WundergraphScalarValue>
        + FromLookAheadValue,
{
    type RawValue = Self;
    type AdditionalFilter = ();
}

impl<C> FilterValue<C> for NaiveDate {
    type RawValue = Self;
    type AdditionalFilter = ();
}

impl<C, DB, Ctx> AsColumnFilter<C, DB, Ctx> for NaiveDateTime {
    type Filter = FilterOption<Self, C>;
}

impl<C, DB, Ctx> AsColumnFilter<C, DB, Ctx> for DateTime<Utc> {
    type Filter = FilterOption<Self, C>;
}

impl<C, DB, Ctx> AsColumnFilter<C, DB, Ctx> for NaiveDate {
    type Filter = FilterOption<Self, C>;
}
