use super::FilterCollector;
use crate::filter::build_filter::BuildFilter;

use crate::diesel_ext::BoxableFilter;
use diesel::backend::Backend;
use diesel::query_builder::QueryFragment;
use diesel::{AppearsOnTable, BoolExpressionMethods};

use std::fmt::{self, Debug};

/// A filter collected that combines all given filters using `or`
pub struct OrCollector<'a, T, DB>(
    Option<Box<dyn BoxableFilter<T, DB, SqlType = ::diesel::sql_types::Bool> + 'a>>,
);

impl<'a, T, DB> Debug for OrCollector<'a, T, DB>
where
    DB: Backend,
    DB::QueryBuilder: Default,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_tuple("OrCollector")
            .field(&self.0.as_ref().map(|q| ::diesel::debug_query(q)))
            .finish()
    }
}

impl<'a, T, DB> Default for OrCollector<'a, T, DB> {
    fn default() -> Self {
        OrCollector(None)
    }
}

impl<'a, T, DB> FilterCollector<'a, T, DB> for OrCollector<'a, T, DB>
where
    DB: Backend + 'a,
    T: 'a,
{
    fn append_filter<F>(&mut self, f: F)
    where
        F: BuildFilter<DB> + 'a,
        F::Ret: AppearsOnTable<T> + QueryFragment<DB> + 'a,
    {
        let f = f.into_filter();
        let c = ::std::mem::replace(&mut self.0, None);
        self.0 = match (c, f) {
            (Some(c), Some(f)) => Some(Box::new(c.or(f)) as Box<_>),
            (Some(c), None) => Some(c),
            (None, Some(f)) => Some(Box::new(f) as Box<_>),
            (None, None) => None,
        };
    }
}

impl<'a, T, DB> BuildFilter<DB> for OrCollector<'a, T, DB>
where
    DB: Backend,
{
    type Ret = Box<dyn BoxableFilter<T, DB, SqlType = ::diesel::sql_types::Bool> + 'a>;

    fn into_filter(self) -> Option<Self::Ret> {
        self.0
    }
}
