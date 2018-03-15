use juniper::FromInputValue;
use juniper::InputValue;
use juniper::GraphQLType;
use juniper::ToInputValue;
use juniper::Registry;
use juniper::meta::MetaType;
use juniper::LookAheadValue;

use ordermap::OrderMap;
use diesel::AppearsOnTable;
use diesel::expression::BoxableExpression;
use diesel::backend::Backend;
use diesel::query_builder::{BoxedSelectStatement, QueryFragment};
use diesel::Table;
use diesel::sql_types::Bool;
use diesel::QueryDsl;

use helper::{FromLookAheadValue, NameBuilder, Nameable};

mod nullable_filter;
mod string_filter;
mod common_filter;
mod reference_filter;

pub mod collector;
pub mod transformator;
pub mod build_filter;
pub mod filter_value;
pub mod inner_filter;

pub use self::common_filter::FilterOption;
pub use self::nullable_filter::{NullableReferenceFilter, ReverseNullableReferenceFilter};
pub use self::reference_filter::ReferenceFilter;

use self::collector::{AndCollector, FilterCollector, OrCollector};
use self::transformator::{NoTransformator, Transformator};
use self::build_filter::BuildFilter;
use self::inner_filter::InnerFilter;

#[derive(Debug)]
pub struct Filter<F, DB, T> {
    and: Option<Vec<Filter<F, DB, T>>>,
    or: Option<Vec<Filter<F, DB, T>>>,
    inner: F,
    p: ::std::marker::PhantomData<(DB, T)>,
}

impl<F, DB, T> Nameable for Filter<F, DB, T>
where
    F: Nameable,
{
    fn name() -> String {
        F::name()
    }
}

impl<F, DB, T> Clone for Filter<F, DB, T>
where
    F: Clone,
{
    fn clone(&self) -> Self {
        Filter {
            and: self.and.clone(),
            or: self.or.clone(),
            inner: self.inner.clone(),
            p: Default::default(),
        }
    }
}

impl<F, DB, T> FromInputValue for Filter<F, DB, T>
where
    F: InnerFilter,
{
    fn from_input_value(v: &InputValue) -> Option<Self> {
        if let Some(obj) = v.to_object_value() {
            let and = obj.get("and")
                .map(|v| Option::from_input_value(*v))
                .unwrap_or_else(|| Option::from_input_value(&InputValue::Null));
            let and = match and {
                Some(and) => and,
                None => return None,
            };
            let or = obj.get("or")
                .map(|v| Option::from_input_value(*v))
                .unwrap_or_else(|| Option::from_input_value(&InputValue::Null));
            let or = match or {
                Some(or) => or,
                None => return None,
            };
            let inner = match F::from_inner_input_value(obj) {
                Some(inner) => inner,
                None => return None,
            };
            Some(Self {
                and,
                or,
                inner,
                p: Default::default(),
            })
        } else {
            None
        }
    }
}

impl<F, DB, T> ToInputValue for Filter<F, DB, T>
where
    F: InnerFilter,
{
    fn to_input_value(&self) -> InputValue {
        let mut map = OrderMap::with_capacity(2 + F::FIELD_COUNT);
        map.insert("and", self.and.to_input_value());
        map.insert("or", self.or.to_input_value());
        self.inner.to_inner_input_value(&mut map);
        InputValue::object(map)
    }
}

impl<F, DB, T> GraphQLType for Filter<F, DB, T>
where
    F: InnerFilter,
{
    type Context = F::Context;
    type TypeInfo = NameBuilder<Self>;

    fn name(info: &Self::TypeInfo) -> Option<&str> {
        Some(info.name())
    }

    fn meta<'r>(info: &Self::TypeInfo, registry: &mut Registry<'r>) -> MetaType<'r> {
        let and = registry.arg_with_default::<Option<Vec<Filter<F, DB, T>>>>("and", &None, info);
        let or = registry.arg_with_default::<Option<Vec<Filter<F, DB, T>>>>("or", &None, info);
        let mut fields = vec![and, or];
        fields.extend(F::register_fields(&Default::default(), registry));
        registry
            .build_input_object_type::<Self>(info, &fields)
            .into_meta()
    }
}

impl<F, DB, T> FromLookAheadValue for Filter<F, DB, T>
where
    F: InnerFilter,
{
    fn from_look_ahead(v: &LookAheadValue) -> Option<Self> {
        if let LookAheadValue::Object(ref obj) = *v {
            let and = obj.iter()
                .find(|o| o.0 == "and")
                .and_then(|o| Vec::from_look_ahead(&o.1));

            let or = obj.iter()
                .find(|o| o.0 == "or")
                .and_then(|o| Vec::from_look_ahead(&o.1));

            let inner = F::from_inner_look_ahead(obj);

            Some(Self {
                and,
                or,
                inner,
                p: Default::default(),
            })
        } else {
            None
        }
    }
}

impl<F, DB, T> BuildFilter for Filter<F, DB, T>
where
    DB: Backend + 'static,
    F: InnerFilter + BuildFilter<Ret = Box<BoxableExpression<T, DB, SqlType = Bool>>> + 'static,
    T: 'static,
{
    type Ret = F::Ret;

    fn into_filter<C>(self, t: C) -> Option<Self::Ret>
    where
        C: Transformator,
    {
        let Filter { and, or, inner, .. } = self;
        let mut and = and.map(|a| {
            a.into_iter().fold(AndCollector::default(), |mut a, f| {
                a.append_filter(f, t);
                a
            })
        }).unwrap_or_default();
        let or = or.map(|a| {
            a.into_iter().fold(OrCollector::default(), |mut o, f| {
                o.append_filter(f, t);
                o
            })
        }).unwrap_or_default();
        and.append_filter(or, t);
        and.append_filter(inner, t);
        and.into_filter(t)
    }
}

impl<F, D, T> Filter<F, D, T>
where
    F: InnerFilter,
    Self: BuildFilter,
{
    pub fn apply_filter<'a, ST, B>(
        self,
        mut q: BoxedSelectStatement<'a, ST, T, B>,
    ) -> BoxedSelectStatement<'a, ST, T, B>
    where
        T: Table + 'a,
        B: Backend + 'a,
        <Self as BuildFilter>::Ret: AppearsOnTable<T> + QueryFragment<B> + 'a,
    {
        if let Some(f) = self.into_filter(NoTransformator) {
            q = <BoxedSelectStatement<_, _, _> as QueryDsl>::filter(q, f);
        }
        q
    }
}
