use filter::build_filter::BuildFilter;
use filter::transformator::{FilterType, Transformator};
use scalar::WundergraphScalarValue;

use diesel::backend::Backend;
use diesel::expression::array_comparison::{In, Many};
use diesel::expression::{AsExpression, Expression, NonAggregate};
use diesel::query_builder::QueryFragment;
use diesel::serialize::ToSql;
use diesel::sql_types::{Bool, HasSqlType};
use diesel::{AppearsOnTable, Column, ExpressionMethods};
use diesel_ext::BoxableFilter;

use juniper::{InputValue, ToInputValue};

#[derive(Debug)]
pub struct EqAny<T, C>(Option<Vec<T>>, ::std::marker::PhantomData<C>);

impl<T, C> EqAny<T, C> {
    pub(super) fn new(v: Option<Vec<T>>) -> Self {
        EqAny(v, Default::default())
    }
}

impl<T, C> Clone for EqAny<T, C>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        EqAny(self.0.clone(), Default::default())
    }
}

impl<C, T, DB> BuildFilter<DB> for EqAny<T, C>
where
    DB: Backend + HasSqlType<<C as Expression>::SqlType> + 'static,
    C: ExpressionMethods + NonAggregate + Column + QueryFragment<DB> + Default + 'static,
    T: AsExpression<C::SqlType> + ToSql<<C as Expression>::SqlType, DB>,
    T::Expression: AppearsOnTable<C::Table> + QueryFragment<DB> + 'static,
    C::Table: 'static,
    In<C, Many<<T as AsExpression<C::SqlType>>::Expression>>:
        AppearsOnTable<C::Table, SqlType = Bool>,
{
    type Ret = Box<BoxableFilter<C::Table, DB, SqlType = Bool>>;

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

impl<T, C> ToInputValue<WundergraphScalarValue> for EqAny<T, C>
where
    T: ToInputValue<WundergraphScalarValue>,
{
    fn to_input_value(&self) -> InputValue<WundergraphScalarValue> {
        self.0.to_input_value()
    }
}
