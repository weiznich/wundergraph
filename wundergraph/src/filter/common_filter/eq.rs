use filter::build_filter::BuildFilter;
use filter::transformator::{FilterType, Transformator};

use diesel::backend::Backend;
use diesel::expression::{operators, AsExpression, Expression, NonAggregate};
use diesel::query_builder::QueryFragment;
use diesel::serialize::ToSql;
use diesel::sql_types::{Bool, HasSqlType};
use diesel::{AppearsOnTable, Column, ExpressionMethods};
use diesel_ext::BoxableFilter;

use juniper::{InputValue, ToInputValue};

#[derive(Debug)]
pub struct Eq<T, C>(Option<T>, ::std::marker::PhantomData<C>);

impl<T, C> Eq<T, C> {
    pub(super) fn new(v: Option<T>) -> Self {
        Eq(v, Default::default())
    }
}

impl<T, C> Clone for Eq<T, C>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Eq(self.0.clone(), Default::default())
    }
}

impl<C, T, DB> BuildFilter<DB> for Eq<T, C>
where
    C: ExpressionMethods + NonAggregate + Column + QueryFragment<DB> + Default + 'static,
    T: AsExpression<C::SqlType> + ToSql<<C as Expression>::SqlType, DB>,
    T::Expression: NonAggregate + AppearsOnTable<C::Table> + QueryFragment<DB> + 'static,
    DB: Backend + HasSqlType<<C as Expression>::SqlType> + 'static,
    C::Table: 'static,
    operators::Eq<C, <T as AsExpression<C::SqlType>>::Expression>:
        AppearsOnTable<C::Table, SqlType = Bool>,
{
    type Ret = Box<BoxableFilter<C::Table, DB, SqlType = Bool>>;

    fn into_filter<F>(self, t: F) -> Option<Self::Ret>
    where
        F: Transformator,
    {
        let Eq(filter, _) = self;
        t.transform(
            filter.map(|v| Box::new(C::default().eq(v)) as Box<_>),
            FilterType::Selective,
        )
    }
}

impl<T, C> ToInputValue for Eq<T, C>
where
    T: ToInputValue,
{
    fn to_input_value(&self) -> InputValue {
        self.0.to_input_value()
    }
}
