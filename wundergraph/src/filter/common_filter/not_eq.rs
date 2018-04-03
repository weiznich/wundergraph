use filter::build_filter::BuildFilter;
use filter::transformator::{FilterType, Transformator};

use diesel::{BoxableExpression, Column, ExpressionMethods, SelectableExpression};
use diesel::expression::{operators, AsExpression, Expression, NonAggregate};
use diesel::query_builder::QueryFragment;
use diesel::backend::Backend;
use diesel::sql_types::{Bool, HasSqlType};
use diesel::serialize::ToSql;

use juniper::{InputValue, ToInputValue};

#[derive(Debug)]
pub struct NotEq<T, C>(Option<T>, ::std::marker::PhantomData<C>);

impl<T, C> NotEq<T, C> {
    pub(super) fn new(v: Option<T>) -> Self {
        NotEq(v, Default::default())
    }
}

impl<T, C> Clone for NotEq<T, C>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        NotEq(self.0.clone(), Default::default())
    }
}

impl<C, T, DB> BuildFilter<DB> for NotEq<T, C>
where
    C: ExpressionMethods + NonAggregate + Column + QueryFragment<DB> + Default + 'static,
    T: AsExpression<C::SqlType> + ToSql<<C as Expression>::SqlType, DB>,
    T::Expression: NonAggregate + SelectableExpression<C::Table> + QueryFragment<DB> + 'static,
    DB: Backend + HasSqlType<<C as Expression>::SqlType> + 'static,
    C::Table: 'static,
    operators::NotEq<C, <T as AsExpression<C::SqlType>>::Expression>: SelectableExpression<C::Table, SqlType = Bool>,
{
    type Ret = Box<BoxableExpression<C::Table, DB, SqlType = Bool>>;

    fn into_filter<F>(self, t: F) -> Option<Self::Ret>
    where
        F: Transformator,
    {
        let NotEq(filter, _) = self;
        t.transform(
            filter.map(|v| Box::new(C::default().ne(v)) as Box<_>),
            FilterType::Exclusive,
        )
    }
}

impl<T, C> ToInputValue for NotEq<T, C>
where
    T: ToInputValue,
{
    fn to_input_value(&self) -> InputValue {
        self.0.to_input_value()
    }
}
