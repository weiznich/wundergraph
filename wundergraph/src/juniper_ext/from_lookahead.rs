use crate::scalar::WundergraphScalarValue;
use juniper::{LookAheadValue, ID};

/// A helper trait marking how to convert a `LookAheadValue` into a specific type
pub trait FromLookAheadValue: Sized {
    /// Try to convert a `LookAheadValue` into a specific type
    ///
    /// For a successful conversion `Some(value)` is returned, otherwise `None`
    fn from_look_ahead(v: &LookAheadValue<'_, WundergraphScalarValue>) -> Option<Self>;
}

impl FromLookAheadValue for i16 {
    fn from_look_ahead(v: &LookAheadValue<'_, WundergraphScalarValue>) -> Option<Self> {
        if let LookAheadValue::Scalar(WundergraphScalarValue::SmallInt(ref i)) = *v {
            Some(*i)
        } else {
            None
        }
    }
}

impl FromLookAheadValue for i32 {
    fn from_look_ahead(v: &LookAheadValue<'_, WundergraphScalarValue>) -> Option<Self> {
        match *v {
            LookAheadValue::Scalar(WundergraphScalarValue::SmallInt(ref i)) => Some(Self::from(*i)),
            LookAheadValue::Scalar(WundergraphScalarValue::Int(ref i)) => Some(*i),
            _ => None,
        }
    }
}

impl FromLookAheadValue for i64 {
    fn from_look_ahead(v: &LookAheadValue<'_, WundergraphScalarValue>) -> Option<Self> {
        match *v {
            LookAheadValue::Scalar(WundergraphScalarValue::SmallInt(ref i)) => Some(Self::from(*i)),
            LookAheadValue::Scalar(WundergraphScalarValue::Int(ref i)) => Some(Self::from(*i)),
            LookAheadValue::Scalar(WundergraphScalarValue::BigInt(ref i)) => Some(*i),
            _ => None,
        }
    }
}

impl FromLookAheadValue for bool {
    fn from_look_ahead(v: &LookAheadValue<'_, WundergraphScalarValue>) -> Option<Self> {
        if let LookAheadValue::Scalar(WundergraphScalarValue::Boolean(ref b)) = *v {
            Some(*b)
        } else {
            None
        }
    }
}

impl FromLookAheadValue for String {
    fn from_look_ahead(v: &LookAheadValue<'_, WundergraphScalarValue>) -> Option<Self> {
        if let LookAheadValue::Scalar(WundergraphScalarValue::String(ref s)) = *v {
            Some(s.to_owned())
        } else {
            None
        }
    }
}

impl FromLookAheadValue for f32 {
    fn from_look_ahead(v: &LookAheadValue<'_, WundergraphScalarValue>) -> Option<Self> {
        if let LookAheadValue::Scalar(WundergraphScalarValue::Float(ref f)) = *v {
            Some(*f)
        } else {
            None
        }
    }
}

impl FromLookAheadValue for f64 {
    fn from_look_ahead(v: &LookAheadValue<'_, WundergraphScalarValue>) -> Option<Self> {
        match *v {
            LookAheadValue::Scalar(WundergraphScalarValue::Float(ref i)) => Some(Self::from(*i)),
            LookAheadValue::Scalar(WundergraphScalarValue::Double(ref i)) => Some(*i),
            _ => None,
        }
    }
}

impl FromLookAheadValue for ID {
    fn from_look_ahead(v: &LookAheadValue<'_, WundergraphScalarValue>) -> Option<Self> {
        match *v {
            LookAheadValue::Scalar(WundergraphScalarValue::Int(ref i)) => {
                Some(Self::from(i.to_string()))
            }
            LookAheadValue::Scalar(WundergraphScalarValue::String(ref s)) => {
                Some(Self::from(s.to_string()))
            }
            _ => None,
        }
    }
}

impl<T> FromLookAheadValue for Vec<T>
where
    T: FromLookAheadValue,
{
    fn from_look_ahead(v: &LookAheadValue<'_, WundergraphScalarValue>) -> Option<Self> {
        if let LookAheadValue::List(ref l) = *v {
            l.iter().map(T::from_look_ahead).collect()
        } else {
            None
        }
    }
}

impl<T> FromLookAheadValue for Box<T>
where
    T: FromLookAheadValue,
{
    fn from_look_ahead(v: &LookAheadValue<'_, WundergraphScalarValue>) -> Option<Self> {
        T::from_look_ahead(v).map(Box::new)
    }
}

impl<T> FromLookAheadValue for Option<T>
where
    T: FromLookAheadValue,
{
    fn from_look_ahead(v: &LookAheadValue<'_, WundergraphScalarValue>) -> Option<Self> {
        Some(T::from_look_ahead(v))
    }
}
