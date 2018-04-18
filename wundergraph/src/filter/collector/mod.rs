use filter::build_filter::BuildFilter;
use filter::transformator::Transformator;

use diesel::backend::Backend;
use diesel::query_builder::QueryFragment;
use diesel::SelectableExpression;

mod and;
mod or;

pub use self::and::AndCollector;
pub use self::or::OrCollector;

pub trait FilterCollector<'a, T, DB: Backend> {
    fn append_filter<F, C>(&mut self, f: F, t: C)
    where
        C: Transformator,
        F: BuildFilter<DB> + 'a,
        F::Ret: SelectableExpression<T> + QueryFragment<DB> + 'a;
}
