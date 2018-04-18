use super::{FilterType, Transformator};
use diesel::sql_types::Bool;
use diesel::BoxableExpression;

#[derive(Debug, Clone, Copy)]
pub struct NoTransformator;

impl Transformator for NoTransformator {
    fn transform<Tab, DB>(
        &self,
        f: Option<Box<BoxableExpression<Tab, DB, SqlType = Bool>>>,
        _tpe: FilterType,
    ) -> Option<Box<BoxableExpression<Tab, DB, SqlType = Bool>>> {
        f
    }
}
