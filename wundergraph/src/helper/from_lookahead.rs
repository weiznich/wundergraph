use juniper::{LookAheadValue, ID};

pub trait FromLookAheadValue: Sized {
    fn from_look_ahead(v: &LookAheadValue) -> Option<Self>;
}

impl FromLookAheadValue for i32 {
    fn from_look_ahead(v: &LookAheadValue) -> Option<Self> {
        if let LookAheadValue::Int(i) = *v {
            Some(i)
        } else {
            None
        }
    }
}

impl FromLookAheadValue for bool {
    fn from_look_ahead(v: &LookAheadValue) -> Option<Self> {
        if let LookAheadValue::Boolean(b) = *v {
            Some(b)
        } else {
            None
        }
    }
}

impl FromLookAheadValue for String {
    fn from_look_ahead(v: &LookAheadValue) -> Option<Self> {
        if let LookAheadValue::String(s) = *v {
            Some(s.to_owned())
        } else {
            None
        }
    }
}

impl FromLookAheadValue for f64 {
    fn from_look_ahead(v: &LookAheadValue) -> Option<Self> {
        if let LookAheadValue::Float(f) = *v {
            Some(f)
        } else {
            None
        }
    }
}

impl FromLookAheadValue for ID {
    fn from_look_ahead(v: &LookAheadValue) -> Option<Self> {
        match *v {
            LookAheadValue::Int(ref i) => Some(ID::from(i.to_string())),
            LookAheadValue::String(s) => Some(ID::from(s.to_string())),
            _ => None,
        }
    }
}

impl<T> FromLookAheadValue for Vec<T>
where
    T: FromLookAheadValue,
{
    fn from_look_ahead(v: &LookAheadValue) -> Option<Self> {
        if let LookAheadValue::List(ref l) = *v {
            l.iter().map(T::from_look_ahead).collect()
        } else {
            None
        }
    }
}

impl<T> FromLookAheadValue for Option<T>
where
    T: FromLookAheadValue,
{
    fn from_look_ahead(v: &LookAheadValue) -> Option<Self> {
        Some(T::from_look_ahead(v))
    }
}

#[cfg(feature = "chrono")]
static RFC3339_PARSE_FORMAT: &'static str = "%+";
#[cfg(feature = "chrono")]
static RFC3339_FORMAT: &'static str = "%Y-%m-%dT%H:%M:%S%.f%:z";

#[cfg(feature = "chrono")]
extern crate chrono;

#[cfg(feature = "chrono")]
impl FromLookAheadValue for self::chrono::NaiveDateTime {
    fn from_look_ahead(v: &LookAheadValue) -> Option<Self> {
        if let LookAheadValue::String(s) = *v {
            Self::parse_from_str(s, RFC3339_PARSE_FORMAT).ok()
        } else {
            None
        }
    }
}

#[cfg(feature = "chrono")]
impl FromLookAheadValue for self::chrono::DateTime<self::chrono::Utc> {
    fn from_look_ahead(v: &LookAheadValue) -> Option<Self> {
        if let LookAheadValue::String(s) = *v {
            s.parse().ok()
        } else {
            None
        }
    }
}

#[cfg(feature = "chrono")]
impl FromLookAheadValue for self::chrono::DateTime<self::chrono::FixedOffset> {
    fn from_look_ahead(v: &LookAheadValue) -> Option<Self> {
        if let LookAheadValue::String(s) = *v {
            self::chrono::DateTime::parse_from_rfc3339(s).ok()
        } else {
            None
        }
    }
}

#[cfg(feature = "chrono")]
impl FromLookAheadValue for self::chrono::NaiveDate {
    fn from_look_ahead(v: &LookAheadValue) -> Option<Self> {
        if let LookAheadValue::String(s) = *v {
            Self::parse_from_str(s, RFC3339_FORMAT).ok()
        } else {
            None
        }
    }
}

#[cfg(feature = "uuid")]
extern crate uuid;

#[cfg(feature = "uuid")]
impl FromLookAheadValue for self::uuid::Uuid {
    fn from_look_ahead(v: &LookAheadValue) -> Option<Self> {
        if let LookAheadValue::String(s) = *v {
            Self::parse_str(s).ok()
        } else {
            None
        }
    }
}
