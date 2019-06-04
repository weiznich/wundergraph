use crate::juniper_ext::{FromLookAheadValue, Nameable};
use crate::query_builder::selection::filter::filter_helper::AsColumnFilter;
use crate::query_builder::selection::filter::filter_value::FilterValue;
use crate::query_builder::selection::filter::FilterOption;
use crate::query_builder::types::{PlaceHolder, WundergraphValue};
use crate::scalar::WundergraphScalarValue;
use diesel::sql_types::Nullable;
use juniper::LookAheadValue;
use uuid_internal::Uuid;

impl Nameable for Uuid {
    fn name() -> String {
        String::from("Uuid")
    }
}

impl FromLookAheadValue for Uuid {
    fn from_look_ahead(v: &LookAheadValue<'_, WundergraphScalarValue>) -> Option<Self> {
        if let LookAheadValue::Scalar(WundergraphScalarValue::String(ref s)) = *v {
            Self::parse_str(s).ok()
        } else {
            None
        }
    }
}

impl WundergraphValue for Uuid {
    type PlaceHolder = PlaceHolder<Self>;
    type SqlType = Nullable<::diesel::sql_types::Uuid>;
}

impl<C, DB, Ctx> AsColumnFilter<C, DB, Ctx> for Uuid {
    type Filter = FilterOption<Self, C>;
}

impl<C> FilterValue<C> for Uuid {
    type RawValue = Self;
    type AdditionalFilter = ();
}
