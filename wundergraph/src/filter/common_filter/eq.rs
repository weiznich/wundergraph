use filter::build_filter::BuildFilter;
use filter::transformator::{FilterType, Transformator};

use diesel::{BoxableExpression, Column, ExpressionMethods, SelectableExpression};
use diesel::expression::{operators, AsExpression, NonAggregate};
use diesel::query_builder::QueryFragment;
use diesel::backend::Backend;
use diesel::sql_types::Bool;

use juniper::{InputValue, ToInputValue};

#[derive(Debug)]
pub struct Eq<T, C, DB>(Option<T>, ::std::marker::PhantomData<(C, DB)>);

impl<T, C, DB> Eq<T, C, DB> {
    pub(super) fn new(v: Option<T>) -> Self {
        Eq(v, Default::default())
    }
}

impl<T, C, DB> Clone for Eq<T, C, DB>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Eq(self.0.clone(), Default::default())
    }
}

impl<C, T, DB> BuildFilter for Eq<T, C, DB>
where
    C: ExpressionMethods + NonAggregate + Column + QueryFragment<DB> + Default + 'static,
    T: AsExpression<C::SqlType>,
    T::Expression: NonAggregate + SelectableExpression<C::Table> + QueryFragment<DB> + 'static,
    DB: Backend + 'static,
    C::Table: 'static,
    operators::Eq<C, <T as AsExpression<C::SqlType>>::Expression>: SelectableExpression<C::Table, SqlType = Bool>,
{
    type Ret = Box<BoxableExpression<C::Table, DB, SqlType = Bool>>;

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

impl<T, C, DB> ToInputValue for Eq<T, C, DB>
where
    T: ToInputValue,
{
    fn to_input_value(&self) -> InputValue {
        self.0.to_input_value()
    }
}
