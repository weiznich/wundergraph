use diesel::backend::Backend;
use diesel::BoxableExpression;
use diesel::sql_types::Bool;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FilterType {
    Selective,
    Exclusive,
}

pub trait Transformator: Copy {
    fn transform<Tab, DB>(
        &self,
        f: Option<Box<BoxableExpression<Tab, DB, SqlType = Bool>>>,
        tpe: FilterType,
    ) -> Option<Box<BoxableExpression<Tab, DB, SqlType = Bool>>>
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
