use juniper::ID;

#[derive(Debug)]
pub struct NameBuilder<T>(String, ::std::marker::PhantomData<T>);

impl<T> Default for NameBuilder<T>
where
    T: Nameable,
{
    fn default() -> Self {
        NameBuilder(T::name(), Default::default())
    }
}

impl<T> NameBuilder<T> {
    pub fn name(&self) -> &str {
        &self.0
    }
}

pub trait Nameable {
    fn name() -> String;
}

impl Nameable for String {
    fn name() -> String {
        String::from("String")
    }
}

impl Nameable for i32 {
    fn name() -> String {
        String::from("Int")
    }
}

impl Nameable for f64 {
    fn name() -> String {
        String::from("Float")
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

impl Nameable for () {
    fn name() -> String {
        String::new()
    }
}

#[cfg(feature = "chrono")]
extern crate chrono;

#[cfg(feature = "chrono")]
impl Nameable for self::chrono::NaiveDateTime {
    fn name() -> String {
        String::from("NaiveDateTime")
    }
}

#[cfg(feature = "chrono")]
impl<O> Nameable for self::chrono::DateTime<O>
where
    O: self::chrono::TimeZone,
{
    fn name() -> String {
        String::from("DateTime")
    }
}
#[cfg(feature = "chrono")]
impl Nameable for self::chrono::NaiveDate {
    fn name() -> String {
        String::from("Date")
    }
}

#[cfg(feature = "uuid")]
extern crate uuid;

#[cfg(feature = "uuid")]
impl Nameable for self::uuid::Uuid {
    fn name() -> String {
        String::from("Uuid")
    }
}
