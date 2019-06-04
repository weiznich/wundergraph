use super::{FilterType, Transformator};
use diesel::sql_types::Bool;
use diesel_ext::BoxableFilter;

#[derive(Debug, Clone, Copy)]
pub struct NoTransformator;

impl Transformator for NoTransformator {
    fn transform<Tab, DB>(
        &self,
        f: Option<Box<BoxableFilter<Tab, DB, SqlType = Bool>>>,
        _tpe: FilterType,
    ) -> Option<Box<BoxableFilter<Tab, DB, SqlType = Bool>>> {
        f
    }
}
