//! This module contains data structures to build a generic graphql interface to
//! filter entities. The main entry point is the [`Filter`](struct.Filter.html) struct

use crate::diesel_ext::BoxableFilter;
use crate::juniper_ext::{NameBuilder, Nameable, FromLookAheadValue};
use crate::scalar::WundergraphScalarValue;
use diesel::backend::Backend;
use diesel::query_builder::{BoxedSelectStatement, QueryFragment};
use diesel::sql_types::Bool;
use diesel::AppearsOnTable;
use diesel::QueryDsl;
use diesel::Table;
use indexmap::IndexMap;
use juniper::meta::{Argument, MetaType};
use juniper::FromInputValue;
use juniper::GraphQLType;
use juniper::InputValue;
use juniper::LookAheadValue;
use juniper::Registry;
use juniper::ToInputValue;
use std::marker::PhantomData;

pub mod build_filter;
pub mod collector;
mod common_filter;
pub mod filter_helper;
pub mod filter_value;
pub mod inner_filter;
mod not;
mod nullable_filter;
mod reference_filter;
mod string_filter;

use self::build_filter::BuildFilter;
use self::collector::{AndCollector, FilterCollector, OrCollector};
use self::inner_filter::InnerFilter;

pub use self::common_filter::FilterOption;
pub use self::not::Not;
pub use self::reference_filter::ReferenceFilter;

/// Main filter struct
///
/// This struct is the main entry point to wundergraphs filter api
/// The exact field specfic filters are given by a subtype (`inner`)
#[derive(Debug)]
pub struct Filter<F, T> {
    and: Option<Vec<Filter<F, T>>>,
    or: Option<Vec<Filter<F, T>>>,
    not: Option<Box<Not<Filter<F, T>>>>,
    inner: F,
    p: PhantomData<(T)>,
}

impl<F, T> Nameable for Filter<F, T>
where
    F: Nameable,
{
    fn name() -> String {
        F::name()
    }
}

impl<F, T> FromInputValue<WundergraphScalarValue> for Filter<F, T>
where
    F: InnerFilter,
{
    fn from_input_value(v: &InputValue<WundergraphScalarValue>) -> Option<Self> {
        if let Some(obj) = v.to_object_value() {
            let and = obj.get("and").map_or_else(
                || Option::from_input_value(&InputValue::Null),
                |v| Option::from_input_value(*v),
            )?;
            let or = obj.get("or").map_or_else(
                || Option::from_input_value(&InputValue::Null),
                |v| Option::from_input_value(*v),
            )?;
            let not = obj.get("not").map_or_else(
                || Option::from_input_value(&InputValue::Null),
                |v| Option::from_input_value(*v),
            )?;
            let inner = F::from_inner_input_value(obj)?;
            Some(Self {
                and,
                or,
                not,
                inner,
                p: PhantomData,
            })
        } else {
            None
        }
    }
}

impl<F, T> ToInputValue<WundergraphScalarValue> for Filter<F, T>
where
    F: InnerFilter,
{
    fn to_input_value(&self) -> InputValue<WundergraphScalarValue> {
        let mut map = IndexMap::with_capacity(Self::FIELD_COUNT);
        self.to_inner_input_value(&mut map);
        InputValue::object(map)
    }
}

impl<F, T> GraphQLType<WundergraphScalarValue> for Filter<F, T>
where
    F: InnerFilter,
{
    type Context = F::Context;
    type TypeInfo = NameBuilder<Self>;

    fn name(info: &Self::TypeInfo) -> Option<&str> {
        Some(info.name())
    }

    fn meta<'r>(
        info: &Self::TypeInfo,
        registry: &mut Registry<'r, WundergraphScalarValue>,
    ) -> MetaType<'r, WundergraphScalarValue>
    where
        WundergraphScalarValue: 'r,
    {
        let fields = Self::register_fields(info, registry);
        registry
            .build_input_object_type::<Self>(info, &fields)
            .into_meta()
    }
}

impl<F, T> FromLookAheadValue for Filter<F, T>
where
    F: InnerFilter,
{
    fn from_look_ahead(v: &LookAheadValue<'_, WundergraphScalarValue>) -> Option<Self> {
        if let LookAheadValue::Object(ref obj) = *v {
            Some(Self::from_inner_look_ahead(obj))
        } else {
            None
        }
    }
}

impl<F, DB, T> BuildFilter<DB> for Filter<F, T>
where
    DB: Backend + 'static,
    F: InnerFilter + BuildFilter<DB> + 'static,
    T: 'static,
    F::Ret: AppearsOnTable<T>,
{
    type Ret = Box<dyn BoxableFilter<T, DB, SqlType = Bool>>;

    fn into_filter(self) -> Option<Self::Ret>
where {
        let Self { and, or, inner, .. } = self;
        let mut and = and
            .map(|a| {
                a.into_iter().fold(AndCollector::default(), |mut a, f| {
                    a.append_filter(f);
                    a
                })
            })
            .unwrap_or_default();
        let or = or
            .map(|a| {
                a.into_iter().fold(OrCollector::default(), |mut o, f| {
                    o.append_filter(f);
                    o
                })
            })
            .unwrap_or_default();
        and.append_filter(self.not.map(|not| *not));
        and.append_filter(or);
        and.append_filter(inner);
        and.into_filter()
    }
}

impl<F, T> Filter<F, T>
where
    F: InnerFilter,
{
    /// Apply the filter to a given select statement
    ///
    /// This function will extend the where clause with the given filter expression
    /// In case there is already an existing filter the new filter will by connected
    /// connected by and
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
        if let Some(f) = self.into_filter() {
            q = <BoxedSelectStatement<'_, _, _, _> as QueryDsl>::filter(q, f);
        }
        q
    }
}

impl<F, T> InnerFilter for Filter<F, T>
where
    F: InnerFilter,
{
    type Context = F::Context;

    const FIELD_COUNT: usize = F::FIELD_COUNT + 3;

    fn from_inner_input_value(
        obj: IndexMap<&str, &InputValue<WundergraphScalarValue>>,
    ) -> Option<Self> {
        let and = obj.get("and").map_or_else(
            || Option::from_input_value(&InputValue::Null),
            |v| Option::from_input_value(*v),
        )?;
        let or = obj.get("or").map_or_else(
            || Option::from_input_value(&InputValue::Null),
            |v| Option::from_input_value(*v),
        )?;
        let not = obj.get("not").map_or_else(
            || Option::from_input_value(&InputValue::Null),
            |v| Option::from_input_value(*v),
        )?;
        let inner = F::from_inner_input_value(obj)?;
        Some(Self {
            and,
            or,
            not,
            inner,
            p: PhantomData,
        })
    }

    fn from_inner_look_ahead(objs: &[(&str, LookAheadValue<'_, WundergraphScalarValue>)]) -> Self {
        let and = objs
            .iter()
            .find(|o| o.0 == "and")
            .and_then(|o| Vec::from_look_ahead(&o.1));

        let or = objs
            .iter()
            .find(|o| o.0 == "or")
            .and_then(|o| Vec::from_look_ahead(&o.1));

        let not = objs
            .iter()
            .find(|o| o.0 == "not")
            .and_then(|o| Box::from_look_ahead(&o.1));

        let inner = F::from_inner_look_ahead(objs);

        Self {
            and,
            or,
            not,
            inner,
            p: PhantomData,
        }
    }

    fn to_inner_input_value(&self, map: &mut IndexMap<&str, InputValue<WundergraphScalarValue>>) {
        map.insert("and", self.and.to_input_value());
        map.insert("or", self.or.to_input_value());
        map.insert("not", self.not.to_input_value());
        self.inner.to_inner_input_value(map);
    }

    fn register_fields<'r>(
        info: &NameBuilder<Self>,
        registry: &mut Registry<'r, WundergraphScalarValue>,
    ) -> Vec<Argument<'r, WundergraphScalarValue>> {
        let and = registry.arg_with_default::<Option<Vec<Self>>>("and", &None, info);
        let or = registry.arg_with_default::<Option<Vec<Self>>>("or", &None, info);
        let not = registry.arg_with_default::<Option<Box<Not<Self>>>>(
            "not",
            &None,
            &NameBuilder::default(),
        );
        let mut fields = vec![and, or, not];
        fields.extend(F::register_fields(&NameBuilder::default(), registry));
        fields
    }
}
