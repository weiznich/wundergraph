use filter::build_filter::BuildFilter;
use filter::transformator::{FilterType, Transformator};

use diesel::backend::Backend;
use diesel::expression::{operators, NonAggregate};
use diesel::query_builder::QueryFragment;
use diesel::sql_types::Bool;
use diesel::{BoxableExpression, Column, ExpressionMethods, SelectableExpression};

use juniper::{InputValue, ToInputValue};

#[derive(Debug)]
pub struct IsNull<C>(bool, ::std::marker::PhantomData<C>);

impl<C> IsNull<C> {
    pub(crate) fn new(v: bool) -> Self {
        IsNull(v, Default::default())
    }
}

impl<C> Clone for IsNull<C> {
    fn clone(&self) -> Self {
        IsNull(self.0, Default::default())
    }
}

impl<C, DB> BuildFilter<DB> for IsNull<C>
where
    C: Column + ExpressionMethods + NonAggregate + QueryFragment<DB> + Default + 'static,
    DB: Backend + 'static,
    C::Table: 'static,
    operators::IsNull<C>: SelectableExpression<C::Table, SqlType = Bool>,
    operators::IsNotNull<C>: SelectableExpression<C::Table, SqlType = Bool>,
{
    type Ret = Box<BoxableExpression<C::Table, DB, SqlType = Bool>>;

    fn into_filter<F>(self, t: F) -> Option<Self::Ret>
    where
        F: Transformator,
    {
        if self.0 {
            t.transform(
                Some(Box::new(C::default().is_null()) as Box<_>),
                FilterType::Selective,
            )
        } else {
            t.transform(
                Some(Box::new(C::default().is_not_null()) as Box<_>),
                FilterType::Exclusive,
            )
        }
    }
}

impl<C> ToInputValue for IsNull<C> {
    fn to_input_value(&self) -> InputValue {
        self.0.to_input_value()
    }
}
