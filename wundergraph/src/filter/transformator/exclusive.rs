use super::{FilterType, Transformator};
use diesel::backend::Backend;
use diesel::BoxableExpression;
use diesel::sql_types::Bool;

#[derive(Debug, Clone, Copy)]
pub struct OnlyExclusive;

impl Transformator for OnlyExclusive {
    fn transform<Tab, DB>(
        &self,
        f: Option<Box<BoxableExpression<Tab, DB, SqlType = Bool>>>,
        tpe: FilterType,
    ) -> Option<Box<BoxableExpression<Tab, DB, SqlType = Bool>>>
    where
        DB: Backend + 'static,
        Tab: 'static,
    {
        match tpe {
            FilterType::Exclusive => {
                use diesel::expression::dsl::not;
                f.map(|f| Box::new(not(f)) as Box<_>)
            }
            FilterType::Selective => None,
        }
    }
}
