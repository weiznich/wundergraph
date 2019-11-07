//! This module contains helper types to combine multiple filter expressions
//! into a final expression

use crate::query_builder::selection::filter::build_filter::BuildFilter;
use diesel::backend::Backend;
use diesel::query_builder::QueryFragment;
use diesel::AppearsOnTable;

mod and;
mod or;

#[doc(inline)]
pub use self::and::AndCollector;
#[doc(inline)]
pub use self::or::OrCollector;

/// A trait indicating that some type could collect multiple separate filter
/// expressions into one single expression
pub trait FilterCollector<'a, T, DB: Backend> {
    /// Append a new filter expression to the already collected expressions
    fn append_filter<F>(&mut self, f: F)
    where
        F: BuildFilter<DB> + 'a,
        F::Ret: AppearsOnTable<T> + QueryFragment<DB> + 'a;
}
