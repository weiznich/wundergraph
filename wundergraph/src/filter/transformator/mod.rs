use diesel::backend::Backend;
use diesel::sql_types::Bool;
use diesel_ext::BoxableFilter;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FilterType {
    Selective,
    Exclusive,
}

pub trait Transformator: Copy {
    fn transform<Tab, DB>(
        &self,
        f: Option<Box<BoxableFilter<Tab, DB, SqlType = Bool>>>,
        tpe: FilterType,
    ) -> Option<Box<BoxableFilter<Tab, DB, SqlType = Bool>>>
    where
        DB: Backend + 'static,
        Tab: 'static;
}

mod default;
mod exclusive;
mod selective;

pub use self::default::NoTransformator;
pub use self::exclusive::OnlyExclusive;
pub use self::selective::OnlySelective;
