use super::FilterCollector;
use filter::build_filter::BuildFilter;
use filter::transformator::Transformator;

use diesel::{BoolExpressionMethods, BoxableExpression, SelectableExpression};
use diesel::backend::Backend;
use diesel::query_builder::QueryFragment;

use std::fmt::{self, Debug};

pub struct OrCollector<'a, T, DB>(
    Option<Box<BoxableExpression<T, DB, SqlType = ::diesel::sql_types::Bool> + 'a>>,
);

impl<'a, T, DB> Debug for OrCollector<'a, T, DB>
where
    DB: Backend,
    DB::QueryBuilder: Default,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
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
    fn append_filter<F, C>(&mut self, f: F, t: C)
    where
        C: Transformator,
        F: BuildFilter<DB> + 'a,
        F::Ret: SelectableExpression<T> + QueryFragment<DB> + 'a,
    {
        let f = f.into_filter(t);
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
    type Ret = Box<BoxableExpression<T, DB, SqlType = ::diesel::sql_types::Bool> + 'a>;

    fn into_filter<C>(self, _t: C) -> Option<Self::Ret>
    where
        C: Transformator,
    {
        self.0
    }
}
