use filter::collector::{AndCollector, FilterCollector};
use filter::inner_filter::InnerFilter;
use filter::build_filter::BuildFilter;
use filter::transformator::{OnlyExclusive, OnlySelective, Transformator};

use diesel::{BoxableExpression, Column, ExpressionMethods, QueryDsl, SelectableExpression};
use diesel::query_dsl::methods::{BoxedDsl, FilterDsl, SelectDsl};
use diesel::query_builder::AsQuery;
use diesel::sql_types::{Bool, SingleValue};
use diesel::backend::Backend;
use diesel::expression::NonAggregate;
use diesel::associations::HasTable;
use diesel::query_builder::QueryFragment;
use diesel::expression::array_comparison::AsInExpression;
use diesel::dsl::{EqAny, Filter, IntoBoxed, NeAny, Select};

use juniper::{FromInputValue, GraphQLType, InputValue, LookAheadValue, Registry, ToInputValue};
use juniper::meta::{Argument, MetaType};

use ordermap::OrderMap;

use helper::{FromLookAheadValue, NameBuilder, Nameable};

#[derive(Debug)]
pub struct ReferenceFilter<C, I, C2> {
    inner: Box<I>,
    p: ::std::marker::PhantomData<(C, I, C2)>,
}

impl<C, I, C2> Clone for ReferenceFilter<C, I, C2>
where
    I: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            p: Default::default(),
        }
    }
}

impl<C, DB, I, C2> BuildFilter<DB> for ReferenceFilter<C, I, C2>
where
    C: Column + NonAggregate + QueryFragment<DB> + Default + 'static,
    C::SqlType: SingleValue,
    C::Table: 'static,
    DB: Backend + 'static,
    I: BuildFilter<DB> + Clone + InnerFilter,
    C2: Column + NonAggregate + QueryFragment<DB> + Default + 'static,
    C2::Table: HasTable<Table = C2::Table>,
    <C2::Table as AsQuery>::Query: FilterDsl<I::Ret>,
    Filter<<C2::Table as AsQuery>::Query, I::Ret>: QueryDsl + SelectDsl<C2>,
    Select<Filter<<C2::Table as AsQuery>::Query, I::Ret>, C2>: QueryDsl + BoxedDsl<'static, DB> + 'static,
    IntoBoxed<'static, Select<Filter<<C2::Table as AsQuery>::Query, I::Ret>, C2>, DB>: AsInExpression<C::SqlType>,
    <IntoBoxed<'static, Select<Filter<<C2::Table as AsQuery>::Query, I::Ret>, C2>, DB> as AsInExpression<C::SqlType>>::InExpression: SelectableExpression<C::Table> + QueryFragment<DB>,
    EqAny<C, IntoBoxed<'static, Select<Filter<<C2::Table as AsQuery>::Query, I::Ret>, C2>, DB>>: BoxableExpression<C::Table, DB, SqlType = Bool>,
    NeAny<C, IntoBoxed<'static, Select<Filter<<C2::Table as AsQuery>::Query, I::Ret>, C2>, DB>>: BoxableExpression<C::Table, DB, SqlType = Bool>
{
    type Ret = Box<BoxableExpression<C::Table, DB, SqlType = Bool>>;

    fn into_filter<F>(self, t: F) -> Option<Self::Ret>
    where
        F: Transformator,
    {
        let mut and = AndCollector::default();

        let selective_inner = self.inner
            .clone()
            .into_filter(OnlySelective)
            .map(|f| <_ as QueryDsl>::filter(C2::Table::table(), f))
            .map(|f| <_ as QueryDsl>::select(f, C2::default()))
            .map(|f| f.into_boxed())
            .map(|q| Box::new(C::default().eq_any(q)) as Box<_>);
        and.append_filter(selective_inner, t);

        let exclusive_inner = self.inner.clone().into_filter(OnlyExclusive)
            .map(|f| <_ as QueryDsl>::filter(C2::Table::table(), f))
            .map(|f| <_ as QueryDsl>::select(f, C2::default()))
            .map(|f| f.into_boxed())
            .map(|q| Box::new(C::default().ne_all(q)) as Box<_>);
        and.append_filter(exclusive_inner, t);

        and.into_filter(t)
    }
}

impl<C, I, C2> Nameable for ReferenceFilter<C, I, C2>
where
    I: Nameable,
{
    fn name() -> String {
        I::name()
    }
}

impl<C, I, C2> FromInputValue for ReferenceFilter<C, I, C2>
where
    I: InnerFilter,
{
    fn from_input_value(v: &InputValue) -> Option<Self> {
        if let Some(obj) = v.to_object_value() {
            I::from_inner_input_value(obj).map(|inner| Self {
                inner: Box::new(inner),
                p: Default::default(),
            })
        } else {
            None
        }
    }
}

impl<C, I, C2> ToInputValue for ReferenceFilter<C, I, C2>
where
    I: InnerFilter,
{
    fn to_input_value(&self) -> InputValue {
        let mut map = OrderMap::with_capacity(I::FIELD_COUNT);
        self.inner.to_inner_input_value(&mut map);
        InputValue::object(map)
    }
}

impl<C, I, C2> FromLookAheadValue for ReferenceFilter<C, I, C2>
where
    I: InnerFilter,
{
    fn from_look_ahead(v: &LookAheadValue) -> Option<Self> {
        if let LookAheadValue::Object(ref obj) = *v {
            let inner = I::from_inner_look_ahead(obj);
            Some(Self {
                inner: Box::new(inner),
                p: Default::default(),
            })
        } else {
            None
        }
    }
}

impl<C, I, C2> GraphQLType for ReferenceFilter<C, I, C2>
where
    I: InnerFilter,
{
    type Context = I::Context;
    type TypeInfo = NameBuilder<Self>;

    fn name(info: &Self::TypeInfo) -> Option<&str> {
        Some(info.name())
    }

    fn meta<'r>(info: &Self::TypeInfo, registry: &mut Registry<'r>) -> MetaType<'r> {
        let fields = I::register_fields(&Default::default(), registry);
        registry
            .build_input_object_type::<Self>(info, &fields)
            .into_meta()
    }
}

impl<C, I, C2> InnerFilter for ReferenceFilter<C, I, C2>
where
    I: InnerFilter,
{
    type Context = I::Context;

    const FIELD_COUNT: usize = I::FIELD_COUNT;

    fn from_inner_input_value(obj: OrderMap<&str, &InputValue>) -> Option<Self> {
        let inner = I::from_inner_input_value(obj);
        let inner = match inner {
            Some(inner) => Box::new(inner),
            None => return None,
        };
        Some(Self {
            inner,
            p: Default::default(),
        })
    }
    fn from_inner_look_ahead(obj: &[(&str, LookAheadValue)]) -> Self {
        let inner = I::from_inner_look_ahead(obj);
        Self {
            inner: Box::new(inner),
            p: Default::default(),
        }
    }
    fn to_inner_input_value(&self, map: &mut OrderMap<&str, InputValue>) {
        self.inner.to_inner_input_value(map);
    }
    fn register_fields<'r>(
        _info: &NameBuilder<Self>,
        registry: &mut Registry<'r>,
    ) -> Vec<Argument<'r>> {
        I::register_fields(&Default::default(), registry)
    }
}
