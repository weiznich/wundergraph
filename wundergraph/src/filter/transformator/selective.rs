use super::{FilterType, Transformator};
use diesel::sql_types::Bool;
use diesel::BoxableExpression;

#[derive(Debug, Clone, Copy)]
pub struct OnlySelective;

impl Transformator for OnlySelective {
    fn transform<Tab, DB>(
        &self,
        f: Option<Box<BoxableExpression<Tab, DB, SqlType = Bool>>>,
        tpe: FilterType,
    ) -> Option<Box<BoxableExpression<Tab, DB, SqlType = Bool>>> {
        match tpe {
            FilterType::Selective => f,
            FilterType::Exclusive => None,
        }
    }
}
