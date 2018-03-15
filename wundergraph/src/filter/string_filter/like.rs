use filter::build_filter::BuildFilter;
use filter::transformator::{FilterType, Transformator};

use diesel::{BoxableExpression, Column, SelectableExpression, TextExpressionMethods};
use diesel::expression::{operators, AsExpression, NonAggregate};
use diesel::query_builder::QueryFragment;
use diesel::backend::Backend;
use diesel::sql_types::Bool;

use juniper::{InputValue, ToInputValue};

#[derive(Debug)]
pub struct Like<C, DB>(Option<String>, ::std::marker::PhantomData<(C, DB)>);

impl<C, DB> Like<C, DB> {
    pub(super) fn new(v: Option<String>) -> Self {
        Like(v, Default::default())
    }
}

impl<C, DB> Clone for Like<C, DB> where {
    fn clone(&self) -> Self {
        Like(self.0.clone(), Default::default())
    }
}

impl<C, DB> BuildFilter for Like<C, DB>
where
    C: TextExpressionMethods + NonAggregate + Column + QueryFragment<DB> + Default + 'static,
    String: AsExpression<C::SqlType>,
    <String as AsExpression<C::SqlType>>::Expression: NonAggregate
        + SelectableExpression<C::Table>
        + QueryFragment<DB>
        + 'static,
    DB: Backend + 'static,
    C::Table: 'static,
    operators::Like<C, <String as AsExpression<C::SqlType>>::Expression>: SelectableExpression<C::Table, SqlType = Bool>,
{
    type Ret = Box<BoxableExpression<C::Table, DB, SqlType = Bool>>;

    fn into_filter<F>(self, t: F) -> Option<Self::Ret>
    where
        F: Transformator,
    {
        let Like(filter, _) = self;
        t.transform(
            filter.map(|v| Box::new(C::default().like(v)) as Box<_>),
            FilterType::Selective,
        )
    }
}

impl<C, DB> ToInputValue for Like<C, DB> {
    fn to_input_value(&self) -> InputValue {
        self.0.to_input_value()
    }
}
