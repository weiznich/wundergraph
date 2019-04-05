use std::marker::PhantomData;

use juniper::ID;

/// A helper type allowing to construct dynamical named types
/// using the juniper api
#[derive(Debug)]
pub struct NameBuilder<T>(String, PhantomData<T>);

impl<T> Default for NameBuilder<T>
where
    T: Nameable,
{
    fn default() -> Self {
        Self(T::name(), PhantomData)
    }
}

impl<T> NameBuilder<T> {
    /// Create a new `NameBuilder` with a given naem.
    pub fn name(&self) -> &str {
        &self.0
    }
}

/// Mark a given type as nameable in a graphql context
pub trait Nameable {
    /// The name of the given type
    ///
    /// The returned name must be a valid graphq name.
    /// 1. The name must be unique for this type
    /// 2. The name should only contain alphanumerical
    /// characters and `_`
    // TODO: check this rules
    // TODO: Try to return `Cow<str>`?
    fn name() -> String;
}

impl Nameable for String {
    fn name() -> String {
        Self::from("String")
    }
}

impl Nameable for i16 {
    fn name() -> String {
        String::from("SmallInt")
    }
}

impl Nameable for i32 {
    fn name() -> String {
        String::from("Int")
    }
}

impl Nameable for i64 {
    fn name() -> String {
        String::from("BigInt")
    }
}

impl Nameable for f32 {
    fn name() -> String {
        String::from("Float")
    }
}

impl Nameable for f64 {
    fn name() -> String {
        String::from("Double")
    }
}

impl Nameable for bool {
    fn name() -> String {
        String::from("bool")
    }
}

impl Nameable for ID {
    fn name() -> String {
        String::from("ID")
    }
}

impl<T> Nameable for Option<T>
where
    T: Nameable,
{
    fn name() -> String {
        format!("Nullable_{}_", T::name())
    }
}

impl<T> Nameable for Vec<T>
where
    T: Nameable,
{
    fn name() -> String {
        format!("Vec_{}_", T::name())
    }
}

impl Nameable for () {
    fn name() -> String {
        String::new()
    }
}

#[cfg(feature = "chrono")]
impl Nameable for chrono_internal::NaiveDateTime {
    fn name() -> String {
        String::from("NaiveDateTime")
    }
}

#[cfg(feature = "chrono")]
impl<O> Nameable for chrono_internal::DateTime<O>
where
    O: chrono_internal::TimeZone,
{
    fn name() -> String {
        String::from("DateTime")
    }
}
#[cfg(feature = "chrono")]
impl Nameable for chrono_internal::NaiveDate {
    fn name() -> String {
        String::from("Date")
    }
}

#[cfg(feature = "uuid")]
impl Nameable for uuid_internal::Uuid {
    fn name() -> String {
        String::from("Uuid")
    }
}
