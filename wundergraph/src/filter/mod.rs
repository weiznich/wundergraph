use juniper::meta::MetaType;
use juniper::FromInputValue;
use juniper::GraphQLType;
use juniper::InputValue;
use juniper::LookAheadValue;
use juniper::Registry;
use juniper::ToInputValue;

use diesel::backend::Backend;
use diesel::query_builder::{BoxedSelectStatement, QueryFragment};
use diesel::sql_types::Bool;
use diesel::AppearsOnTable;
use diesel::QueryDsl;
use diesel::Table;
use diesel_ext::BoxableFilter;
use indexmap::IndexMap;

use helper::{FromLookAheadValue, NameBuilder, Nameable};

mod common_filter;
mod nullable_filter;
mod reference_filter;
mod string_filter;

pub mod build_filter;
pub mod collector;
pub mod filter_value;
pub mod inner_filter;
pub mod transformator;

pub use self::common_filter::FilterOption;
pub use self::nullable_filter::{NullableReferenceFilter, ReverseNullableReferenceFilter};
pub use self::reference_filter::ReferenceFilter;

use self::build_filter::BuildFilter;
use self::collector::{AndCollector, FilterCollector, OrCollector};
use self::inner_filter::InnerFilter;
use self::transformator::{NoTransformator, Transformator};

#[derive(Debug)]
pub struct Filter<F, T> {
    and: Option<Vec<Filter<F, T>>>,
    or: Option<Vec<Filter<F, T>>>,
    inner: F,
    p: ::std::marker::PhantomData<(T)>,
}

impl<F, T> Nameable for Filter<F, T>
where
    F: Nameable,
{
    fn name() -> String {
        F::name()
    }
}

impl<F, T> Clone for Filter<F, T>
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

impl<F, T> FromInputValue for Filter<F, T>
where
    F: InnerFilter,
{
    fn from_input_value(v: &InputValue) -> Option<Self> {
        if let Some(obj) = v.to_object_value() {
            let and = obj
                .get("and")
                .map(|v| Option::from_input_value(*v))
                .unwrap_or_else(|| Option::from_input_value(&InputValue::Null));
            let and = match and {
                Some(and) => and,
                None => return None,
            };
            let or = obj
                .get("or")
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

impl<F, T> ToInputValue for Filter<F, T>
where
    F: InnerFilter,
{
    fn to_input_value(&self) -> InputValue {
        let mut map = IndexMap::with_capacity(2 + F::FIELD_COUNT);
        map.insert("and", self.and.to_input_value());
        map.insert("or", self.or.to_input_value());
        self.inner.to_inner_input_value(&mut map);
        InputValue::object(map)
    }
}

impl<F, T> GraphQLType for Filter<F, T>
where
    F: InnerFilter,
{
    type Context = F::Context;
    type TypeInfo = NameBuilder<Self>;

    fn name(info: &Self::TypeInfo) -> Option<&str> {
        Some(info.name())
    }

    fn meta<'r>(info: &Self::TypeInfo, registry: &mut Registry<'r>) -> MetaType<'r> {
        let and = registry.arg_with_default::<Option<Vec<Filter<F, T>>>>("and", &None, info);
        let or = registry.arg_with_default::<Option<Vec<Filter<F, T>>>>("or", &None, info);
        let mut fields = vec![and, or];
        fields.extend(F::register_fields(&Default::default(), registry));
        registry
            .build_input_object_type::<Self>(info, &fields)
            .into_meta()
    }
}

impl<F, T> FromLookAheadValue for Filter<F, T>
where
    F: InnerFilter,
{
    fn from_look_ahead(v: &LookAheadValue) -> Option<Self> {
        if let LookAheadValue::Object(ref obj) = *v {
            let and = obj
                .iter()
                .find(|o| o.0 == "and")
                .and_then(|o| Vec::from_look_ahead(&o.1));

            let or = obj
                .iter()
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

impl<F, DB, T> BuildFilter<DB> for Filter<F, T>
where
    DB: Backend + 'static,
    F: InnerFilter + BuildFilter<DB, Ret = Box<BoxableFilter<T, DB, SqlType = Bool>>> + 'static,
    T: 'static,
{
    type Ret = F::Ret;

    fn into_filter<C>(self, t: C) -> Option<Self::Ret>
    where
        C: Transformator,
    {
        let Filter { and, or, inner, .. } = self;
        let mut and =
            and.map(|a| {
                a.into_iter().fold(AndCollector::default(), |mut a, f| {
                    a.append_filter(f, t);
                    a
                })
            }).unwrap_or_default();
        let or =
            or.map(|a| {
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

impl<F, T> Filter<F, T>
where
    F: InnerFilter,
{
    pub fn apply_filter<'a, ST, DB>(
        self,
        mut q: BoxedSelectStatement<'a, ST, T, DB>,
    ) -> BoxedSelectStatement<'a, ST, T, DB>
    where
        T: Table + 'a,
        DB: Backend + 'a,
        Self: BuildFilter<DB>,
        <Self as BuildFilter<DB>>::Ret: AppearsOnTable<T> + QueryFragment<DB> + 'a,
    {
        if let Some(f) = self.into_filter(NoTransformator) {
            q = <BoxedSelectStatement<_, _, _> as QueryDsl>::filter(q, f);
        }
        q
    }
}
