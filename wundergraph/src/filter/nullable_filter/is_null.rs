use std::marker::PhantomData;

use filter::build_filter::BuildFilter;
use scalar::WundergraphScalarValue;

use diesel::backend::Backend;
use diesel::expression::{operators, NonAggregate};
use diesel::query_builder::QueryFragment;
use diesel::sql_types::Bool;
use diesel::{AppearsOnTable, Column, ExpressionMethods};
use diesel_ext::BoxableFilter;

use juniper::{InputValue, ToInputValue};

#[derive(Debug)]
pub struct IsNull<C>(bool, PhantomData<C>);

impl<C> IsNull<C> {
    pub(crate) fn new(v: bool) -> Self {
        IsNull(v, PhantomData)
    }
}

impl<C> Clone for IsNull<C> {
    fn clone(&self) -> Self {
        IsNull(self.0, PhantomData)
    }
}

impl<C, DB> BuildFilter<DB> for IsNull<C>
where
    C: Column + ExpressionMethods + NonAggregate + QueryFragment<DB> + Default + 'static,
    DB: Backend + 'static,
    C::Table: 'static,
    operators::IsNull<C>: AppearsOnTable<C::Table, SqlType = Bool>,
    operators::IsNotNull<C>: AppearsOnTable<C::Table, SqlType = Bool>,
{
    type Ret = Box<BoxableFilter<C::Table, DB, SqlType = Bool>>;

    fn into_filter(self) -> Option<Self::Ret> {
        if self.0 {
            Some(Box::new(C::default().is_null()) as Box<_>)
        } else {
            Some(Box::new(C::default().is_not_null()) as Box<_>)
        }
    }
}

impl<C> ToInputValue<WundergraphScalarValue> for IsNull<C> {
    fn to_input_value(&self) -> InputValue<WundergraphScalarValue> {
        self.0.to_input_value()
    }
}
