use std::marker::PhantomData;

use crate::query_builder::selection::filter::build_filter::BuildFilter;
use crate::scalar::WundergraphScalarValue;

use crate::diesel_ext::BoxableFilter;
use diesel::backend::Backend;
use diesel::expression::{operators, AsExpression, Expression, NonAggregate};
use diesel::query_builder::QueryFragment;
use diesel::serialize::ToSql;
use diesel::sql_types::{Bool, HasSqlType};
use diesel::{AppearsOnTable, Column, ExpressionMethods};

use juniper::{InputValue, ToInputValue};

#[derive(Debug)]
pub struct NotEq<T, C>(Option<T>, PhantomData<C>);

impl<T, C> NotEq<T, C> {
    pub(super) fn new(v: Option<T>) -> Self {
        Self(v, PhantomData)
    }
}

impl<T, C> Clone for NotEq<T, C>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<C, T, DB> BuildFilter<DB> for NotEq<T, C>
where
    C: ExpressionMethods + NonAggregate + Column + QueryFragment<DB> + Default + 'static,
    T: AsExpression<C::SqlType> + ToSql<<C as Expression>::SqlType, DB>,
    T::Expression: NonAggregate + AppearsOnTable<C::Table> + QueryFragment<DB> + 'static,
    DB: Backend + HasSqlType<<C as Expression>::SqlType> + 'static,
    C::Table: 'static,
    operators::NotEq<C, <T as AsExpression<C::SqlType>>::Expression>:
        AppearsOnTable<C::Table, SqlType = Bool>,
{
    type Ret = Box<dyn BoxableFilter<C::Table, DB, SqlType = Bool>>;

    fn into_filter(self) -> Option<Self::Ret> {
        let Self(filter, _) = self;
        filter.map(|v| Box::new(C::default().ne(v)) as Box<_>)
    }
}

impl<T, C> ToInputValue<WundergraphScalarValue> for NotEq<T, C>
where
    T: ToInputValue<WundergraphScalarValue>,
{
    fn to_input_value(&self) -> InputValue<WundergraphScalarValue> {
        self.0.to_input_value()
    }
}
