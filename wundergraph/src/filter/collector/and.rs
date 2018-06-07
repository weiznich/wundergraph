use super::FilterCollector;
use filter::build_filter::BuildFilter;
use filter::transformator::Transformator;

use diesel::backend::Backend;
use diesel::query_builder::QueryFragment;
use diesel::{AppearsOnTable, BoolExpressionMethods};
use diesel_ext::BoxableFilter;

use std::fmt::{self, Debug};

pub struct AndCollector<'a, T, DB>(
    Option<Box<BoxableFilter<T, DB, SqlType = ::diesel::sql_types::Bool> + 'a>>,
);

impl<'a, T, DB> Debug for AndCollector<'a, T, DB>
where
    DB: Backend,
    DB::QueryBuilder: Default,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_tuple("AndCollector")
            .field(&self.0.as_ref().map(|q| ::diesel::debug_query(q)))
            .finish()
    }
}

impl<'a, T, DB> Default for AndCollector<'a, T, DB> {
    fn default() -> Self {
        AndCollector(None)
    }
}

impl<'a, T, DB> FilterCollector<'a, T, DB> for AndCollector<'a, T, DB>
where
    DB: Backend + 'a,
    T: 'a,
{
    fn append_filter<F, C>(&mut self, f: F, t: C)
    where
        C: Transformator,
        F: BuildFilter<DB> + 'a,
        F::Ret: AppearsOnTable<T> + QueryFragment<DB> + 'a,
    {
        let f = f.into_filter(t);
        let c = ::std::mem::replace(&mut self.0, None);
        self.0 = match (c, f) {
            (Some(c), Some(f)) => Some(Box::new(c.and(f)) as Box<_>),
            (Some(c), None) => Some(c),
            (None, Some(f)) => Some(Box::new(f) as Box<_>),
            (None, None) => None,
        };
    }
}

impl<'a, T, DB> BuildFilter<DB> for AndCollector<'a, T, DB>
where
    DB: Backend,
{
    type Ret = Box<BoxableFilter<T, DB, SqlType = ::diesel::sql_types::Bool> + 'a>;

    fn into_filter<C>(self, _t: C) -> Option<Self::Ret>
    where
        C: Transformator,
    {
        self.0
    }
}
