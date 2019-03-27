use crate::error::WundergraphError;
use crate::filter::build_filter::BuildFilter;
use crate::query_helper::tuple::TupleIndex;
use crate::query_helper::{HasMany, HasOne};
use crate::scalar::WundergraphScalarValue;
use crate::{LoadingHandler, WundergraphContext};
use diesel::associations::HasTable;
use diesel::backend::Backend;
use diesel::connection::Connection;
use diesel::deserialize::{self, FromSql};
use diesel::dsl::SqlTypeOf;
use diesel::expression::bound::Bound;
use diesel::expression::nullable::Nullable as NullableExpression;
use diesel::expression::{AsExpression, NonAggregate};
use diesel::query_builder::{BoxedSelectStatement, QueryFragment};
use diesel::query_dsl::methods::BoxedDsl;
use diesel::serialize::ToSql;
use diesel::sql_types::{BigInt, Bool, Float4, Float8, Integer, SmallInt, Text};
use diesel::sql_types::{HasSqlType, NotNull, Nullable};
use diesel::{
    AppearsOnTable, ExpressionMethods, Identifiable, NullableExpressionMethods, QueryDsl,
    QuerySource, Queryable, SelectableExpression, Table,
};
use failure::Error;
use juniper::{Executor, FromContext, GraphQLType};
use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
#[cfg(feature = "chrono")]
extern crate chrono;

use juniper::LookAheadMethods;

pub trait PlaceHolderMarker {
    type InnerType;

    fn into_inner(self) -> Option<Self::InnerType>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, FromSqlRow, Hash)]
pub struct PlaceHolder<T>(Option<T>);

impl<T> PlaceHolderMarker for PlaceHolder<T> {
    type InnerType = T;

    fn into_inner(self) -> Option<T> {
        self.0
    }
}

impl<T> Default for PlaceHolder<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T> Into<Option<T>> for PlaceHolder<T> {
    fn into(self) -> Option<T> {
        self.0
    }
}

impl<T> Into<Option<Option<T>>> for PlaceHolder<T> {
    fn into(self) -> Option<Option<T>> {
        Some(self.0)
    }
}

impl<'a, T> Into<Option<&'a T>> for &'a PlaceHolder<T> {
    fn into(self) -> Option<&'a T> {
        self.0.as_ref()
    }
}

impl<ST, T, DB> FromSql<Nullable<ST>, DB> for PlaceHolder<T>
where
    DB: Backend,
    T: FromSql<ST, DB> + ::std::fmt::Debug,
    ST: NotNull,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        if bytes.is_some() {
            T::from_sql(bytes).map(Some).map(Self)
        } else {
            Ok(Self(None))
        }
    }
}

pub type SqlTypeOfPlaceholder<T, DB, K, Table, Ctx> =
    <T as WundergraphFieldList<DB, K, Table, Ctx>>::SqlType;

pub trait FieldValueResolver<T, DB, Ctx>
where
    T: WundergraphValue,
    DB: Backend,
{
    fn new(elements: usize) -> Self;

    fn resolve_value(
        &mut self,
        value: T::PlaceHolder,
        selection: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    ) -> Result<Option<juniper::Value<WundergraphScalarValue>>, Error>;

    fn finalize(
        self,
        selection: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    ) -> Result<Option<Vec<juniper::Value<WundergraphScalarValue>>>, Error>;
}

#[derive(Debug, Clone, Copy)]
pub struct DirectResolver;

impl<T, DB, Ctx> FieldValueResolver<T, DB, Ctx> for DirectResolver
where
    DB: Backend,
    T: GraphQLType<WundergraphScalarValue, TypeInfo = ()> + WundergraphValue,
    T::PlaceHolder: Into<Option<T>>,
    <T as GraphQLType<WundergraphScalarValue>>::Context: FromContext<Ctx>,
{
    fn new(_elements: usize) -> Self {
        Self
    }

    fn resolve_value(
        &mut self,
        value: T::PlaceHolder,
        _selection: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    ) -> Result<Option<juniper::Value<WundergraphScalarValue>>, Error> {
        Ok(Some(
            executor
                .resolve_with_ctx(&(), &value.into().expect("Loading should not fail"))
                .map_err(|inner| WundergraphError::JuniperError { inner })?,
        ))
    }

    fn finalize(
        self,
        _selection: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        _executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    ) -> Result<Option<Vec<juniper::Value<WundergraphScalarValue>>>, Error> {
        Ok(None)
    }
}

pub trait ResolveWundergraphFieldValue<DB: Backend, Ctx>: WundergraphValue + Sized {
    type Resolver: FieldValueResolver<Self, DB, Ctx>;
}

impl<T, DB, Ctx> ResolveWundergraphFieldValue<DB, Ctx> for T
where
    DB: Backend,
    T: GraphQLType<WundergraphScalarValue> + WundergraphValue,
    DirectResolver: FieldValueResolver<T, DB, Ctx>,
{
    type Resolver = DirectResolver;
}

pub trait WundergraphValue {
    type PlaceHolder: 'static;
    type SqlType: 'static;
}

impl WundergraphValue for i16 {
    type PlaceHolder = PlaceHolder<Self>;
    type SqlType = Nullable<SmallInt>;
}

impl WundergraphValue for i32 {
    type PlaceHolder = PlaceHolder<Self>;
    type SqlType = Nullable<Integer>;
}

impl WundergraphValue for i64 {
    type PlaceHolder = PlaceHolder<Self>;
    type SqlType = Nullable<BigInt>;
}

impl WundergraphValue for bool {
    type PlaceHolder = PlaceHolder<Self>;
    type SqlType = Nullable<Bool>;
}

impl WundergraphValue for String {
    type PlaceHolder = PlaceHolder<Self>;
    type SqlType = Nullable<Text>;
}

impl WundergraphValue for f32 {
    type PlaceHolder = PlaceHolder<Self>;
    type SqlType = Nullable<Float4>;
}

impl WundergraphValue for f64 {
    type PlaceHolder = PlaceHolder<Self>;
    type SqlType = Nullable<Float8>;
}

#[cfg(feature = "chrono")]
impl WundergraphValue for chrono::NaiveDateTime {
    type PlaceHolder = PlaceHolder<Self>;
    type SqlType = Nullable<::diesel::sql_types::Timestamp>;
}

impl<T, Inner> WundergraphValue for Vec<T>
where
    T: WundergraphValue<SqlType = Nullable<Inner>> + 'static,
    Inner: NotNull + 'static,
{
    type PlaceHolder = PlaceHolder<Self>;
    type SqlType = Nullable<::diesel::sql_types::Array<Inner>>;
}

impl<T> WundergraphValue for Option<T>
where
    T: WundergraphValue,
{
    type PlaceHolder = T::PlaceHolder;
    type SqlType = T::SqlType;
}

impl<R, T> WundergraphValue for HasOne<R, T>
where
    R: WundergraphValue + Clone + Eq + Hash,
    for<'a> &'a T: Identifiable<Id = &'a R>,
{
    type PlaceHolder = R::PlaceHolder;
    type SqlType = R::SqlType;
}

#[allow(missing_debug_implementations)]
pub struct HasOneResolver<R, T, Ctx> {
    values: Vec<Option<R>>,
    p: PhantomData<(T, Ctx)>,
}

impl<'a, R, T, DB, Ctx> FieldValueResolver<HasOne<R, T>, DB, Ctx> for HasOneResolver<R, T, Ctx>
where
    DB: Backend
        + HasSqlType<SqlTypeOfPlaceholder<T::FieldList, DB, T::PrimaryKeyIndex, T::Table, Ctx>>
        + HasSqlType<SqlTypeOf<NullableExpression<<T::Table as Table>::PrimaryKey>>>
        + 'static,
    Option<R>: Queryable<SqlTypeOf<NullableExpression<<T::Table as Table>::PrimaryKey>>, DB>
        + ToSql<SqlTypeOf<NullableExpression<<T::Table as Table>::PrimaryKey>>, DB>,
    HasOne<R, T>: WundergraphValue,
    <HasOne<R, T> as WundergraphValue>::PlaceHolder: Into<Option<R>>,
    R: WundergraphValue + Clone + Eq + Hash,
    for<'b> &'b T: Identifiable<Id = &'b R>,
    T: LoadingHandler<DB, Ctx>,
    <T::Table as QuerySource>::FromClause: QueryFragment<DB>,
    T::Table: BoxedDsl<
            'static,
            DB,
            Output = BoxedSelectStatement<
                'static,
                SqlTypeOf<<T::Table as Table>::AllColumns>,
                T::Table,
                DB,
            >,
        > + 'static,
    NullableExpression<<T::Table as Table>::PrimaryKey>: ExpressionMethods,
    <T::Filter as BuildFilter<DB>>::Ret: AppearsOnTable<T::Table>,
    for<'b> &'b Option<R>: AsExpression<
        SqlTypeOf<NullableExpression<<T::Table as Table>::PrimaryKey>>,
        Expression = Bound<
            SqlTypeOf<NullableExpression<<T::Table as Table>::PrimaryKey>>,
            &'b Option<R>,
        >,
    >,
    <T::Table as Table>::PrimaryKey: QueryFragment<DB>,
    SqlTypeOf<<T::Table as Table>::PrimaryKey>: NotNull,
    DB::QueryBuilder: Default,
    Ctx: WundergraphContext,
    Ctx::Connection: Connection<Backend = DB>,
{
    fn new(elements: usize) -> Self {
        Self {
            values: Vec::with_capacity(elements),
            p: PhantomData,
        }
    }

    fn resolve_value(
        &mut self,
        value: <HasOne<R, T> as WundergraphValue>::PlaceHolder,
        _selection: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        _executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    ) -> Result<Option<juniper::Value<WundergraphScalarValue>>, Error> {
        self.values.push(value.into());
        Ok(None)
    }

    fn finalize(
        self,
        selection: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    ) -> Result<Option<Vec<juniper::Value<WundergraphScalarValue>>>, Error> {
        use diesel::RunQueryDsl;
        let conn = executor.context().get_connection();

        let q = T::build_query(selection)?
            .filter(
                <T::Table as Table>::primary_key(&<T as HasTable>::table())
                    .nullable()
                    .eq_any(&self.values),
            )
            .select((
                <T::Table as Table>::primary_key(&<T as HasTable>::table()).nullable(),
                T::get_select(selection)?,
            ));

        let items = q.load::<(
            Option<R>,
            <T::FieldList as WundergraphFieldList<_, _, _, Ctx>>::PlaceHolder,
        )>(conn)?;

        let (keys, placeholder): (Vec<_>, Vec<_>) = items.into_iter().unzip();

        let values = T::FieldList::resolve(placeholder, selection, T::FIELD_NAMES, executor)?;

        let map = keys
            .into_iter()
            .zip(values.into_iter())
            .collect::<HashMap<_, _>>();

        Ok(Some(
            self.values
                .iter()
                .map(|key| map.get(key).cloned().unwrap_or(juniper::Value::Null))
                .collect(),
        ))
    }
}

impl<R, T, DB, Ctx> FieldValueResolver<Option<HasOne<R, T>>, DB, Ctx> for HasOneResolver<R, T, Ctx>
where
    DB: Backend,
    R: WundergraphValue + Clone + Hash + Eq,
    Self: FieldValueResolver<HasOne<R, T>, DB, Ctx>,
    for<'b> &'b T: Identifiable<Id = &'b R>,
    R::PlaceHolder: Into<Option<R>>,
{
    fn new(elements: usize) -> Self {
        Self {
            values: Vec::with_capacity(elements),
            p: PhantomData,
        }
    }

    fn resolve_value(
        &mut self,
        value: <Option<HasOne<R, T>> as WundergraphValue>::PlaceHolder,
        _selection: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        _executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    ) -> Result<Option<juniper::Value<WundergraphScalarValue>>, Error> {
        self.values.push(value.into());
        Ok(None)
    }

    fn finalize(
        self,
        selection: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    ) -> Result<Option<Vec<juniper::Value<WundergraphScalarValue>>>, Error> {
        <Self as FieldValueResolver<HasOne<R, T>, DB, Ctx>>::finalize(self, selection, executor)
    }
}

impl<R, T, DB, Ctx> ResolveWundergraphFieldValue<DB, Ctx> for Option<HasOne<R, T>>
where
    HasOneResolver<R, T, Ctx>: FieldValueResolver<HasOne<R, T>, DB, Ctx>
        + FieldValueResolver<Option<HasOne<R, T>>, DB, Ctx>,
    R: WundergraphValue + Clone + Eq + Hash,
    <HasOne<R, T> as WundergraphValue>::PlaceHolder: Into<Option<R>>,
    HasOne<R, T>: WundergraphValue,
    DB: Backend,
{
    type Resolver = HasOneResolver<R, T, Ctx>;
}

impl<R, T, DB, Ctx> ResolveWundergraphFieldValue<DB, Ctx> for HasOne<R, T>
where
    HasOneResolver<R, T, Ctx>: FieldValueResolver<HasOne<R, T>, DB, Ctx>,
    R: WundergraphValue + Clone + Eq + Hash,
    Self::PlaceHolder: Into<Option<R>>,
    Self: WundergraphValue,
    DB: Backend,
{
    type Resolver = HasOneResolver<R, T, Ctx>;
}

pub trait AppendToTuple<T> {
    type Out;
    const LENGHT: usize;
}

impl<T> AppendToTuple<T> for () {
    type Out = (T,);

    const LENGHT: usize = 1;
}

pub trait TableFieldCollector<T> {
    type Out;

    const FIELD_COUNT: usize;

    fn map<F: Fn(usize) -> R, R>(local_index: usize, callback: F) -> Option<R>;
}

pub trait NonTableFieldCollector<T> {
    type Out;

    const FIELD_COUNT: usize;

    fn map<F: Fn(usize) -> R, R>(local_index: usize, callback: F) -> Option<R>;
}

pub trait FieldListExtractor {
    type Out;

    const FIELD_COUNT: usize;

    fn map<F: Fn(usize) -> R, R>(local_index: usize, callback: F) -> Option<R>;
}

pub trait NonTableFieldExtractor {
    type Out;

    const FIELD_COUNT: usize;

    fn map<F: Fn(usize) -> R, R>(local_index: usize, callback: F) -> Option<R>;
}

impl FieldListExtractor for () {
    type Out = ();

    const FIELD_COUNT: usize = 0;

    fn map<F: Fn(usize) -> R, R>(_local_index: usize, _callback: F) -> Option<R> {
        None
    }
}

impl NonTableFieldExtractor for () {
    type Out = ();

    const FIELD_COUNT: usize = 0;

    fn map<F: Fn(usize) -> R, R>(_local_index: usize, _callback: F) -> Option<R> {
        None
    }
}

impl<T> TableFieldCollector<T> for ()
where
    T: WundergraphValue,
{
    type Out = (T,);

    const FIELD_COUNT: usize = 1;

    fn map<F: Fn(usize) -> R, R>(local_index: usize, callback: F) -> Option<R> {
        if local_index == 0 {
            Some(callback(0))
        } else {
            None
        }
    }
}

impl<T> TableFieldCollector<HasMany<T>> for () {
    type Out = ();

    const FIELD_COUNT: usize = 0;

    fn map<F: Fn(usize) -> R, R>(_local_index: usize, _callback: F) -> Option<R> {
        None
    }
}

impl<T> NonTableFieldCollector<T> for ()
where
    T: WundergraphValue,
{
    type Out = ();

    const FIELD_COUNT: usize = 0;

    fn map<F: Fn(usize) -> R, R>(_local_index: usize, _callback: F) -> Option<R> {
        None
    }
}

impl<T> NonTableFieldCollector<HasMany<T>> for () {
    type Out = (HasMany<T>,);

    const FIELD_COUNT: usize = 1;

    fn map<F: Fn(usize) -> R, R>(local_index: usize, callback: F) -> Option<R> {
        if local_index == 0 {
            Some(callback(0))
        } else {
            None
        }
    }
}

pub trait WundergraphResolvePlaceHolderList<R, DB: Backend, Ctx> {
    fn resolve(
        self,
        get_name: impl Fn(usize) -> &'static str,
        selection: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        executor: &Executor<Ctx, WundergraphScalarValue>,
    ) -> Result<Vec<juniper::Object<WundergraphScalarValue>>, Error>;
}

pub trait WundergraphFieldList<DB: Backend, Key, Table, Ctx> {
    type PlaceHolder: Queryable<Self::SqlType, DB> + 'static;
    type SqlType: 'static;

    const TABLE_FIELD_COUNT: usize;
    const NON_TABLE_FIELD_COUNT: usize;

    fn resolve(
        placeholder: Vec<Self::PlaceHolder>,
        select: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        name_list: &'static [&'static str],
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    ) -> Result<Vec<juniper::Value<WundergraphScalarValue>>, Error>;

    fn map_table_field<F: Fn(usize) -> R, R>(local_index: usize, callback: F) -> Option<R>;
    fn map_non_table_field<Func: Fn(usize) -> Ret, Ret>(
        local_index: usize,
        callback: Func,
    ) -> Option<Ret>;
}

#[derive(Debug)]
pub struct AssociationsReturn<K: Eq + Hash> {
    keys: Vec<Option<K>>,
    fields: Vec<&'static str>,
    values: HashMap<Option<K>, Vec<(usize, Vec<juniper::Value<WundergraphScalarValue>>)>>,
}

impl<K: Eq + Hash> AssociationsReturn<K> {
    fn empty() -> Self {
        Self {
            keys: Vec::new(),
            fields: Vec::new(),
            values: HashMap::new(),
        }
    }

    fn init(&mut self, get_keys: &impl Fn() -> Vec<Option<K>>) {
        if self.keys.is_empty() {
            self.keys = get_keys()
        }
    }

    fn push_field<T, O, DB, Ctx>(
        &mut self,
        field: &'static str,
        selection: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    ) -> Result<(), Error>
    where
        DB: Backend,
        T: WundergraphResolveAssociation<K, O, DB, Ctx>,
    {
        let values = T::resolve(selection, &self.keys, executor)?;

        let len = self.fields.len();
        self.fields.push(field);

        for (k, v) in values {
            self.values.entry(k).or_insert_with(Vec::new).push((len, v));
        }
        Ok(())
    }

    fn merge_with_object_list(
        self,
        objs: Vec<juniper::Object<WundergraphScalarValue>>,
    ) -> Vec<juniper::Value<WundergraphScalarValue>> {
        let Self {
            values,
            keys,
            fields,
        } = self;
        if keys.is_empty() {
            objs.into_iter().map(juniper::Value::object).collect()
        } else {
            objs.into_iter()
                .zip(keys.into_iter())
                .map(|(mut obj, key)| {
                    let values = values.get(&key);
                    if let Some(values) = values {
                        let mut value_iter = values.iter().peekable();
                        for (idx, field_name) in fields.iter().enumerate() {
                            match value_iter.peek() {
                                Some((field_idx, _)) if idx == *field_idx => {
                                    let value = value_iter
                                        .next()
                                        .expect("It's there because peekable")
                                        .1
                                        .clone();
                                    obj.add_field(
                                        field_name.to_owned(),
                                        juniper::Value::List(value),
                                    );
                                }
                                None | Some(_) => {
                                    obj.add_field(
                                        field_name.to_owned(),
                                        juniper::Value::List(Vec::new()),
                                    );
                                }
                            }
                        }
                    } else {
                        for f in &fields {
                            obj.add_field(f.to_owned(), juniper::Value::List(Vec::new()));
                        }
                    }
                    obj
                })
                .map(juniper::Value::object)
                .collect()
        }
    }
}

pub trait WundergraphResolveAssociations<K, Other, DB, Ctx>
where
    K: Eq + Hash,
    DB: Backend,
{
    fn resolve(
        selection: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        get_name: impl Fn(usize) -> &'static str,
        get_keys: impl Fn() -> Vec<Option<K>>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    ) -> Result<AssociationsReturn<K>, Error>;
}

impl<K, Other, DB, Ctx> WundergraphResolveAssociations<K, Other, DB, Ctx> for ()
where
    K: Eq + Hash,
    DB: Backend,
{
    fn resolve(
        _selection: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        _get_name: impl Fn(usize) -> &'static str,
        _get_keys: impl Fn() -> Vec<Option<K>>,
        _executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    ) -> Result<AssociationsReturn<K>, Error> {
        Ok(AssociationsReturn::empty())
    }
}

pub trait WundergraphResolveAssociation<K, Other, DB: Backend, Ctx> {
    fn resolve(
        selection: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        primary_keys: &[Option<K>],
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    ) -> Result<HashMap<Option<K>, Vec<juniper::Value<WundergraphScalarValue>>>, Error>;
}

pub trait WundergraphBelongsTo<Other, DB, Ctx>: LoadingHandler<DB, Ctx>
where
    DB: Backend + 'static,
    Self::Table: 'static,
    <Self::Table as QuerySource>::FromClause: QueryFragment<DB>,
    DB::QueryBuilder: Default,
{
    type ForeignKeyColumn: Default
        + NonAggregate
        + SelectableExpression<Self::Table>
        + QueryFragment<DB>;

    type Key: Eq + Hash;

    fn resolve(
        selection: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        keys: &[Option<Self::Key>],
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    ) -> Result<HashMap<Option<Self::Key>, Vec<juniper::Value<WundergraphScalarValue>>>, Error>;

    fn build_response(
        res: Vec<(
            Option<Self::Key>,
            <Self::FieldList as WundergraphFieldList<
                DB,
                Self::PrimaryKeyIndex,
                Self::Table,
                Ctx,
            >>::PlaceHolder,
        )>,
        selection: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    ) -> Result<HashMap<Option<Self::Key>, Vec<juniper::Value<WundergraphScalarValue>>>, Error>
    {
        let (keys, vals): (Vec<_>, Vec<_>) = res.into_iter().unzip();
        let vals = <<Self as LoadingHandler<DB, Ctx>>::FieldList as WundergraphFieldList<
            DB,
            <Self as LoadingHandler<DB, Ctx>>::PrimaryKeyIndex,
            <Self as HasTable>::Table,
            Ctx,
        >>::resolve(
            vals,
            selection,
            <Self as LoadingHandler<DB, Ctx>>::FIELD_NAMES,
            executor,
        )?;
        Ok(keys
            .into_iter()
            .zip(vals.into_iter())
            .fold(HashMap::new(), |mut m, (k, v)| {
                (*m.entry(k).or_insert_with(Vec::new)).push(v);
                m
            }))
    }
}

impl<T, K, Other, DB, Ctx> WundergraphResolveAssociation<K, Other, DB, Ctx> for HasMany<T>
where
    DB: Backend + 'static,
    T: WundergraphBelongsTo<Other, DB, Ctx, Key = K>,
    K: Eq + Hash,
    T::Table: 'static,
    <T::Table as QuerySource>::FromClause: QueryFragment<DB>,
    DB::QueryBuilder: Default,
{
    fn resolve(
        selection: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        primary_keys: &[Option<K>],
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    ) -> Result<HashMap<Option<K>, Vec<juniper::Value<WundergraphScalarValue>>>, Error> {
        T::resolve(selection, primary_keys, executor)
    }
}

macro_rules! wundergraph_add_one_to_index {
    ($idx_head: tt $($idx: tt)+) => {
        wundergraph_add_one_to_index!{$($idx)*}
    };
    ($idx: tt) => {
        $idx + 1
    }
}

macro_rules! wundergraph_impl_field_extractor {
    ($($T: ident,)*) => {
        wundergraph_impl_field_extractor!{
            t = [$($T,)*],
            rest = [],
        }
    };
    (
        t = [$T:ident, $($Ts:ident,)+],
        rest = [$($Other:ident,)*],
    ) => {
        wundergraph_impl_field_extractor!{
            t = [$($Ts,)*],
            rest = [$($Other,)* $T,],
        }
    };
    (
        t = [$T:ident,],
        rest = [$($Other:ident,)*],
    ) => {
        impl<$($Other,)* $T> FieldListExtractor for ($($Other,)* $T,)
        where ($($Other,)*): TableFieldCollector<$T>
        {
            type Out = <($($Other,)*) as TableFieldCollector<$T>>::Out;

            const FIELD_COUNT: usize = <($($Other,)*) as TableFieldCollector<$T>>::FIELD_COUNT;

            fn map<Func: Fn(usize) -> Ret, Ret>(local_index: usize, callback: Func) -> Option<Ret> {
                <($($Other,)*) as TableFieldCollector<$T>>::map(local_index, callback)
            }
        }

        impl<$($Other,)* $T> NonTableFieldExtractor for ($($Other,)* $T,)
        where ($($Other,)*): NonTableFieldCollector<$T>
        {
            type Out = <($($Other,)*) as NonTableFieldCollector<$T>>::Out;

            const FIELD_COUNT: usize = <($($Other,)*) as NonTableFieldCollector<$T>>::FIELD_COUNT;

            fn map<Func: Fn(usize) -> Ret, Ret>(local_index: usize, callback: Func) -> Option<Ret> {
                <($($Other,)*) as NonTableFieldCollector<$T>>::map(local_index, callback)
            }
        }
    };
}

macro_rules! wundergraph_value_impl {
    ($(
        $Tuple:tt {
            $(($idx:tt) -> $T:ident, $ST: ident, $TT: ident,) +
        }
    )+) => {
        $(
            impl<$($T,)+> WundergraphValue for ($($T,)+)
                where $($T: WundergraphValue,)+
            {
                type PlaceHolder = ($($T::PlaceHolder,)+);
                type SqlType = ($($T::SqlType,)+);
            }


            #[allow(clippy::use_self)]
            impl<Back, $($T,)+ $($ST,)+ Ctx> WundergraphResolvePlaceHolderList<($($ST,)*), Back, Ctx> for Vec<($(PlaceHolder<$T>,)+)>
            where $($ST: WundergraphValue<PlaceHolder = PlaceHolder<$T>> +
                    ResolveWundergraphFieldValue<Back, Ctx> ,)*
                  $($T: 'static,)*
                  Back: Backend,
            {
                fn resolve(
                    self,
                    get_name: impl Fn(usize) -> &'static str,
                    selection: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
                    executor: &Executor<Ctx, WundergraphScalarValue>,
                ) -> Result<Vec<juniper::Object<WundergraphScalarValue>>, Error>
                {
                    let mut resolver = (
                        $(<$ST as ResolveWundergraphFieldValue<Back, Ctx>>::Resolver::new(self.len()),)*
                    );
                    let mut objs: Vec<juniper::Object<WundergraphScalarValue>>
                        = vec![juniper::Object::with_capacity(wundergraph_add_one_to_index!($($idx)*)-1); self.len()];

                    self.into_iter().zip(objs.iter_mut()).map(|(placeholder, obj)|{
                        $(
                            if let Some(selection) = selection.select_child(get_name($idx)) {
                                if let Some(value) = resolver.$idx.resolve_value(
                                    placeholder.$idx,
                                    selection,
                                    executor
                                )? {
                                    obj.add_field(get_name($idx), value);
                                }
                            }
                        )*
                        Ok(())
                    }).collect::<Result<Vec<_>, Error>>()?;
                    $(
                        if let Some(selection) = selection.select_child(get_name($idx)) {
                            let vals = resolver.$idx.finalize(selection, executor)?;
                            if let Some(vals) = vals {
                                for (obj, val) in objs.iter_mut().zip(vals.into_iter()) {
                                    obj.add_field(get_name($idx), val);
                                }
                            }
                        }
                    )*
                    Ok(objs)
                }

            }

            impl<Key, Back, Other, Ctx, $($T,)*> WundergraphResolveAssociations<Key, Other, Back, Ctx> for ($($T,)*)
            where Back: Backend,
                  Key: Eq + Hash,
                $($T: WundergraphResolveAssociation<Key, Other, Back, Ctx>,)*

            {
                fn resolve(
                    selection: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
                    get_name: impl Fn(usize) -> &'static str,
                    get_keys: impl Fn() -> Vec<Option<Key>>,
                    executor: &Executor<'_, Ctx, WundergraphScalarValue>,
                ) -> Result<AssociationsReturn<Key>, Error>
                {
                    let mut ret = AssociationsReturn::empty();
                    $(
                        if let Some(selection) = selection.select_child(get_name($idx)) {
                            ret.init(&get_keys);
                            ret.push_field::<$T, Other, Back, Ctx>(get_name($idx), selection, executor)?;
                        }
                    )*
                    Ok(ret)
                }
            }

            impl<$($T,)* New> AppendToTuple<New> for ($($T,)*) {
                type Out = ($($T,)* New);
                const LENGHT: usize = wundergraph_add_one_to_index!($($idx)*) + 1;
            }

            wundergraph_impl_field_extractor!($($T,)*);

            impl<$($T,)* Next> TableFieldCollector<Next> for ($($T,)*)
            where Next: WundergraphValue,
                  ($($T,)*): FieldListExtractor,
                  <($($T,)*) as FieldListExtractor>::Out: AppendToTuple<Next>,
            {
                type Out = <<($($T,)*) as FieldListExtractor>::Out as AppendToTuple<Next>>::Out;

                const FIELD_COUNT: usize = <<($($T,)*) as FieldListExtractor>::Out as AppendToTuple<Next>>::LENGHT;

                fn map<Func: Fn(usize) -> Ret, Ret>(local_index: usize, callback: Func) -> Option<Ret> {
                    if local_index == <<($($T,)*) as FieldListExtractor>::Out as AppendToTuple<Next>>::LENGHT - 1 {
                        Some(callback(wundergraph_add_one_to_index!($($idx)*)))
                    } else {
                        <($($T,)*) as FieldListExtractor>::map(local_index, callback)
                    }
                }
            }

            impl<$($T,)* Next> TableFieldCollector<HasMany<Next>> for ($($T,)*)
                where ($($T,)*): FieldListExtractor,
            {
                type Out = <($($T,)*) as FieldListExtractor>::Out;

                const FIELD_COUNT: usize = <($($T,)*) as FieldListExtractor>::FIELD_COUNT;

                fn map<Func: Fn(usize) -> Ret, Ret>(local_index: usize, callback: Func) -> Option<Ret> {
                    <($($T,)*) as FieldListExtractor>::map(local_index, callback)
                }
            }

            impl<$($T,)* Next> NonTableFieldCollector<Next> for ($($T,)*)
            where Next: WundergraphValue,
                  ($($T,)*): NonTableFieldExtractor,
            {
                type Out = <($($T,)*) as NonTableFieldExtractor>::Out;

                const FIELD_COUNT: usize = <($($T,)*) as NonTableFieldExtractor>::FIELD_COUNT;

                fn map<Func: Fn(usize) -> Ret, Ret>(local_index: usize, callback: Func) -> Option<Ret> {
                    <($($T,)*) as NonTableFieldExtractor>::map(local_index, callback)
                }
            }

            impl<$($T,)* Next> NonTableFieldCollector<HasMany<Next>> for ($($T,)*)
            where ($($T,)*): NonTableFieldExtractor,
                  <($($T,)*) as NonTableFieldExtractor>::Out: AppendToTuple<HasMany<Next>>,
            {
                type Out = <<($($T,)*) as NonTableFieldExtractor>::Out as AppendToTuple<HasMany<Next>>>::Out;

                const FIELD_COUNT: usize = <<($($T,)*) as NonTableFieldExtractor>::Out as AppendToTuple<HasMany<Next>>>::LENGHT;

                fn map<Func: Fn(usize) -> Ret, Ret>(local_index: usize, callback: Func) -> Option<Ret> {
                    if local_index == <<($($T,)*) as NonTableFieldExtractor>::Out as AppendToTuple<HasMany<Next>>>::LENGHT - 1 {
                        Some(callback(wundergraph_add_one_to_index!($($idx)*)))
                    } else {
                        <($($T,)*) as NonTableFieldExtractor>::map(local_index, callback)
                    }
                }
            }

            impl<Back, Key, Table, Ctx, $($T,)*> WundergraphFieldList<Back, Key, Table, Ctx> for ($($T,)*)
            where Back: Backend,
                  ($($T,)*): FieldListExtractor + NonTableFieldExtractor,
                  <($($T,)*) as FieldListExtractor>::Out: WundergraphValue,
                  <<($($T,)*) as FieldListExtractor>::Out as WundergraphValue>::PlaceHolder: TupleIndex<Key> +
                      Queryable<<<($($T,)*) as FieldListExtractor>::Out as WundergraphValue>::SqlType, Back> + 'static,
            Vec<<<($($T,)*) as FieldListExtractor>::Out as WundergraphValue>::PlaceHolder>:
            WundergraphResolvePlaceHolderList<<($($T,)*) as FieldListExtractor>::Out, Back, Ctx>,
            <<<($($T,)*) as FieldListExtractor>::Out as WundergraphValue>::PlaceHolder as TupleIndex<Key>>::Value: PlaceHolderMarker,
            <<<<($($T,)*) as FieldListExtractor>::Out as WundergraphValue>::PlaceHolder as TupleIndex<Key>>::Value as PlaceHolderMarker>::InnerType: Eq + Hash + Clone,
            <($($T,)*) as NonTableFieldExtractor>::Out: WundergraphResolveAssociations<<<<<($($T,)*) as FieldListExtractor>::Out as WundergraphValue>::PlaceHolder as TupleIndex<Key>>::Value as PlaceHolderMarker>::InnerType, Table, Back, Ctx>,
            Ctx: WundergraphContext,
            Ctx::Connection: Connection<Backend = Back>,
            {
                type PlaceHolder = <<($($T,)*) as FieldListExtractor>::Out as WundergraphValue>::PlaceHolder;
                type SqlType = <<($($T,)*) as FieldListExtractor>::Out as WundergraphValue>::SqlType;

                const TABLE_FIELD_COUNT: usize = <($($T,)*) as FieldListExtractor>::FIELD_COUNT;
                const NON_TABLE_FIELD_COUNT: usize = <($($T,)*) as NonTableFieldExtractor>::FIELD_COUNT;

                fn resolve(
                    placeholder: Vec<Self::PlaceHolder>,
                    select: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
                    name_list: &'static [&'static str],
                    executor: &Executor<'_, Ctx, WundergraphScalarValue>,
                ) -> Result<Vec<juniper::Value<WundergraphScalarValue>>, Error> {
                    let extern_values = {
                        let keys = ||{
                            placeholder.iter()
                                .map(TupleIndex::<Key>::get)
                                .map(<_ as PlaceHolderMarker>::into_inner)
                                .collect::<Vec<_>>()
                        };

                        let name = |local_pos| {
                            <($($T,)*) as NonTableFieldExtractor>::map(
                                local_pos,
                                |pos| name_list[pos]
                            ).expect("Name is there")
                        };
                        <($($T,)*) as NonTableFieldExtractor>::Out::resolve(
                            select, name, keys, executor
                        )?
                    };
                    let name = |local_pos| {
                        <($($T,)*) as FieldListExtractor>::map(local_pos, |pos| {
                            name_list[pos]
                        }).expect("Name is there")
                    };
                    let objs = placeholder.resolve(
                        name,
                        select,
                        executor,
                    )?;

                     Ok(extern_values.merge_with_object_list(objs))
                }

                fn map_table_field<Func: Fn(usize) -> Ret, Ret>(local_index: usize, callback: Func) -> Option<Ret> {
                    <($($T,)*) as FieldListExtractor>::map(local_index, callback)
                }

                fn map_non_table_field<Func: Fn(usize) -> Ret, Ret>(local_index: usize, callback: Func) -> Option<Ret> {
                    <($($T,)*) as NonTableFieldExtractor>::map(local_index, callback)
                }
            }

            impl<$($T,)*> PlaceHolderMarker for ($($T,)*)
            where $($T: PlaceHolderMarker,)*
            {
                type InnerType = ($(<$T as PlaceHolderMarker>::InnerType,)*);

                fn into_inner(self) -> Option<Self::InnerType> {
                    Some((
                        $(
                            <$T as PlaceHolderMarker>::into_inner(self.$idx)?,
                        )*
                    ))
                }
            }

        )+
    }
}

__diesel_for_each_tuple!(wundergraph_value_impl);
