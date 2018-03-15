use filter::build_filter::BuildFilter;
use filter::transformator::{FilterType, Transformator};

use diesel::{BoxableExpression, Column, ExpressionMethods, SelectableExpression};
use diesel::expression::{operators, NonAggregate};
use diesel::query_builder::QueryFragment;
use diesel::backend::Backend;
use diesel::sql_types::Bool;

use juniper::{InputValue, ToInputValue};

#[derive(Debug)]
pub struct IsNull<C, DB>(bool, ::std::marker::PhantomData<(C, DB)>);

impl<C, DB> IsNull<C, DB> {
    pub(crate) fn new(v: bool) -> Self {
        IsNull(v, Default::default())
    }
}

impl<C, DB> Clone for IsNull<C, DB> {
    fn clone(&self) -> Self {
        IsNull(self.0, Default::default())
    }
}

impl<C, DB> BuildFilter for IsNull<C, DB>
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

impl<C, DB> ToInputValue for IsNull<C, DB> {
    fn to_input_value(&self) -> InputValue {
        self.0.to_input_value()
    }
}
