use super::{FilterType, Transformator};
use diesel::sql_types::Bool;
use diesel_ext::BoxableFilter;

#[derive(Debug, Clone, Copy)]
pub struct OnlySelective;

impl Transformator for OnlySelective {
    fn transform<Tab, DB>(
        &self,
        f: Option<Box<BoxableFilter<Tab, DB, SqlType = Bool>>>,
        tpe: FilterType,
    ) -> Option<Box<BoxableFilter<Tab, DB, SqlType = Bool>>> {
        match tpe {
            FilterType::Selective => f,
            FilterType::Exclusive => None,
        }
    }
}
