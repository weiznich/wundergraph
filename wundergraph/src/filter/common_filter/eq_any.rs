use filter::build_filter::BuildFilter;
use filter::transformator::{FilterType, Transformator};

use diesel::{BoxableExpression, Column, ExpressionMethods, SelectableExpression};
use diesel::expression::{AsExpression, NonAggregate};
use diesel::query_builder::QueryFragment;
use diesel::expression::array_comparison::{In, Many};
use diesel::backend::Backend;
use diesel::sql_types::Bool;

use juniper::{InputValue, ToInputValue};

#[derive(Debug)]
pub(super) struct EqAny<T, C, DB>(Option<Vec<T>>, ::std::marker::PhantomData<(DB, C)>);

impl<T, C, DB> EqAny<T, C, DB> {
    pub(super) fn new(v: Option<Vec<T>>) -> Self {
        EqAny(v, Default::default())
    }
}

impl<T, C, DB> Clone for EqAny<T, C, DB>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        EqAny(self.0.clone(), Default::default())
    }
}

impl<C, T, DB> BuildFilter for EqAny<T, C, DB>
where
    DB: Backend + 'static,
    C: ExpressionMethods + NonAggregate + Column + QueryFragment<DB> + Default + 'static,
    T: AsExpression<C::SqlType>,
    T::Expression: SelectableExpression<C::Table> + QueryFragment<DB> + 'static,
    C::Table: 'static,
    In<C, Many<<T as AsExpression<C::SqlType>>::Expression>>: SelectableExpression<C::Table, SqlType = Bool>,
{
    type Ret = Box<BoxableExpression<C::Table, DB, SqlType = Bool>>;

    fn into_filter<F>(self, t: F) -> Option<Self::Ret>
    where
        F: Transformator,
    {
        let EqAny(filter, _) = self;
        t.transform(
            filter.map(|v| Box::new(C::default().eq_any(v)) as Box<_>),
            FilterType::Selective,
        )
    }
}

impl<T, C, DB> ToInputValue for EqAny<T, C, DB>
where
    T: ToInputValue,
{
    fn to_input_value(&self) -> InputValue {
        self.0.to_input_value()
    }
}
