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
use crate::filter::build_filter::BuildFilter;
use crate::query_helper::tuple::TupleIndex;
use crate::query_helper::{HasMany, HasOne};
use crate::scalar::WundergraphScalarValue;
use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use crate::LoadingHandler;
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
            let r = T::from_sql(bytes).map(Some).map(PlaceHolder);
            r
        } else {
            Ok(PlaceHolder(None))
        }
    }
}

pub type SqlTypeOfPlaceholder<T, DB, K, Table> = <T as WundergraphFieldList<DB, K, Table>>::SqlType;

pub trait FieldValueResolver<T, DB>
where
    T: WundergraphValue,
    DB: Backend,
{
    fn new(elements: usize) -> Self;

    fn resolve_value(
        &mut self,
        value: T::PlaceHolder,
        selection: &juniper::LookAheadSelection<WundergraphScalarValue>,
    ) -> Result<Option<juniper::Value<WundergraphScalarValue>>, Error>;

    fn finalize(
        self,
        conn: &impl Connection<Backend = DB>,
        selection: &juniper::LookAheadSelection<WundergraphScalarValue>,
    ) -> Result<Option<Vec<juniper::Value<WundergraphScalarValue>>>, Error>;
}

//#[derive(Debug)]
#[allow(missing_debug_implementations)]
pub struct DirectResolver<T>(PhantomData<T>);

impl<T, DB> FieldValueResolver<T, DB> for DirectResolver<T>
where
    T: IntoValue,
    DB: Backend,
{
    fn new(_elements: usize) -> Self {
        Self(PhantomData)
    }

    fn resolve_value(
        &mut self,
        value: T::PlaceHolder,
        _selection: &juniper::LookAheadSelection<WundergraphScalarValue>,
    ) -> Result<Option<juniper::Value<WundergraphScalarValue>>, Error> {
        Ok(Some(T::resolve(value)))
    }

    fn finalize(
        self,
        _conn: &impl Connection<Backend = DB>,
        _selection: &juniper::LookAheadSelection<WundergraphScalarValue>,
    ) -> Result<Option<Vec<juniper::Value<WundergraphScalarValue>>>, Error> {
        Ok(None)
    }
}

pub trait ResolveWundergraphFieldValue<DB: Backend>: WundergraphValue + Sized {
    type Resolver: FieldValueResolver<Self, DB>;
}

pub trait IntoValue: WundergraphValue {
    fn resolve(placeholder: Self::PlaceHolder) -> juniper::Value<WundergraphScalarValue>;
}

pub trait WundergraphValue {
    type PlaceHolder: 'static;
    type SqlType: 'static;
}

impl WundergraphValue for i16 {
    type PlaceHolder = PlaceHolder<Self>;
    type SqlType = Nullable<SmallInt>;
}

impl<DB: Backend> ResolveWundergraphFieldValue<DB> for i16 {
    type Resolver = DirectResolver<Self>;
}

impl IntoValue for i16 {
    fn resolve(placeholder: Self::PlaceHolder) -> juniper::Value<WundergraphScalarValue> {
        juniper::Value::scalar(placeholder.0.expect("Value is there"))
    }
}

impl WundergraphValue for i32 {
    type PlaceHolder = PlaceHolder<Self>;
    type SqlType = Nullable<Integer>;
}

impl<DB: Backend> ResolveWundergraphFieldValue<DB> for i32 {
    type Resolver = DirectResolver<Self>;
}

impl IntoValue for i32 {
    fn resolve(placeholder: Self::PlaceHolder) -> juniper::Value<WundergraphScalarValue> {
        juniper::Value::scalar(placeholder.0.expect("Value is there"))
    }
}

impl WundergraphValue for i64 {
    type PlaceHolder = PlaceHolder<Self>;
    type SqlType = Nullable<BigInt>;
}

impl<DB: Backend> ResolveWundergraphFieldValue<DB> for i64 {
    type Resolver = DirectResolver<Self>;
}

impl IntoValue for i64 {
    fn resolve(placeholder: Self::PlaceHolder) -> juniper::Value<WundergraphScalarValue> {
        juniper::Value::scalar(placeholder.0.expect("Value is there"))
    }
}

impl WundergraphValue for bool {
    type PlaceHolder = PlaceHolder<Self>;
    type SqlType = Nullable<Bool>;
}

impl<DB: Backend> ResolveWundergraphFieldValue<DB> for bool {
    type Resolver = DirectResolver<Self>;
}

impl IntoValue for bool {
    fn resolve(placeholder: Self::PlaceHolder) -> juniper::Value<WundergraphScalarValue> {
        juniper::Value::scalar(placeholder.0.expect("Value is there"))
    }
}

impl WundergraphValue for String {
    type PlaceHolder = PlaceHolder<Self>;
    type SqlType = Nullable<Text>;
}

impl<DB: Backend> ResolveWundergraphFieldValue<DB> for String {
    type Resolver = DirectResolver<Self>;
}

impl IntoValue for String {
    fn resolve(placeholder: Self::PlaceHolder) -> juniper::Value<WundergraphScalarValue> {
        juniper::Value::scalar(placeholder.0.expect("Value is there"))
    }
}

impl WundergraphValue for f32 {
    type PlaceHolder = PlaceHolder<Self>;
    type SqlType = Nullable<Float4>;
}

impl<DB: Backend> ResolveWundergraphFieldValue<DB> for f32 {
    type Resolver = DirectResolver<Self>;
}

impl IntoValue for f32 {
    fn resolve(placeholder: Self::PlaceHolder) -> juniper::Value<WundergraphScalarValue> {
        juniper::Value::scalar(placeholder.0.expect("Value is there"))
    }
}

impl WundergraphValue for f64 {
    type PlaceHolder = PlaceHolder<Self>;
    type SqlType = Nullable<Float8>;
}

impl<DB: Backend> ResolveWundergraphFieldValue<DB> for f64 {
    type Resolver = DirectResolver<Self>;
}

impl IntoValue for f64 {
    fn resolve(placeholder: Self::PlaceHolder) -> juniper::Value<WundergraphScalarValue> {
        juniper::Value::scalar(placeholder.0.expect("Value is there"))
    }
}

#[cfg(feature = "chrono")]
impl WundergraphValue for chrono::NaiveDateTime {
    type PlaceHolder = PlaceHolder<Self>;
    type SqlType = Nullable<::diesel::sql_types::Timestamp>;
}

#[cfg(feature = "chrono")]
impl<DB: Backend> ResolveWundergraphFieldValue<DB> for chrono::NaiveDateTime {
    type Resolver = DirectResolver<Self>;
}

#[cfg(feature = "chrono")]
impl IntoValue for chrono::NaiveDateTime {
    fn resolve(placeholder: Self::PlaceHolder) -> juniper::Value<WundergraphScalarValue> {
        juniper::Value::scalar(placeholder.0.expect("Value is there").timestamp() as f64)
    }
}

impl<T, Inner> WundergraphValue for Vec<T>
where
    T: WundergraphValue<SqlType = Nullable<Inner>> + 'static,
    Inner: NotNull + 'static,
{
    type PlaceHolder = PlaceHolder<Vec<T>>;
    type SqlType = Nullable<::diesel::sql_types::Array<Inner>>;
}

#[cfg(feature = "postgres")]
impl<T> ResolveWundergraphFieldValue<::diesel::pg::Pg> for Vec<T>
where
    T: 'static,
    Self: WundergraphValue<PlaceHolder = PlaceHolder<Vec<T>>> + IntoValue,
{
    type Resolver = DirectResolver<Self>;
}

impl<T> IntoValue for Vec<T>
where
    Self: WundergraphValue<PlaceHolder = PlaceHolder<Vec<T>>>,
    T: IntoValue + WundergraphValue<PlaceHolder = PlaceHolder<T>> + 'static,
{
    fn resolve(placeholder: Self::PlaceHolder) -> juniper::Value<WundergraphScalarValue> {
        let v = placeholder.0.expect("Value is there");
        let list = v
            .into_iter()
            .map(|v| T::resolve(PlaceHolder(Some(v))))
            .collect();
        juniper::Value::List(list)
    }
}

impl<T> WundergraphValue for Option<T>
where
    T: WundergraphValue,
{
    type PlaceHolder = T::PlaceHolder;
    type SqlType = T::SqlType;
}

impl<T, DB> ResolveWundergraphFieldValue<DB> for Option<T>
where
    DB: Backend,
    Self: WundergraphValue<PlaceHolder = PlaceHolder<T>> + IntoValue,
    T: 'static,
{
    type Resolver = DirectResolver<Self>;
}

impl<T> IntoValue for Option<T>
where
    Self: WundergraphValue<PlaceHolder = PlaceHolder<T>>,
    T: Into<WundergraphScalarValue> + 'static,
{
    fn resolve(placeholder: Self::PlaceHolder) -> juniper::Value<WundergraphScalarValue> {
        placeholder
            .0
            .map(Into::into)
            .map(juniper::Value::Scalar)
            .unwrap_or(juniper::Value::Null)
    }
}

impl<T> IntoValue for Option<Vec<T>>
where
    T: 'static,
    Vec<T>: IntoValue + WundergraphValue<PlaceHolder = PlaceHolder<Vec<T>>>,
    Self: WundergraphValue<PlaceHolder = PlaceHolder<Vec<T>>>,
{
    fn resolve(placeholder: Self::PlaceHolder) -> juniper::Value<WundergraphScalarValue> {
        if placeholder.0.is_some() {
            <Vec<T> as IntoValue>::resolve(placeholder)
        } else {
            juniper::Value::Null
        }
    }
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
pub struct HasOneResolver<R, T>(PhantomData<T>, Vec<Option<R>>);

impl<R, T, DB> FieldValueResolver<HasOne<R, T>, DB> for HasOneResolver<R, T>
where
    DB: Backend
        + HasSqlType<SqlTypeOfPlaceholder<T::FieldList, DB, T::PrimaryKeyIndex, T::Table>>
        + HasSqlType<SqlTypeOf<NullableExpression<<T::Table as Table>::PrimaryKey>>>
        + 'static,
    Option<R>: Queryable<SqlTypeOf<NullableExpression<<T::Table as Table>::PrimaryKey>>, DB>
        + ToSql<SqlTypeOf<NullableExpression<<T::Table as Table>::PrimaryKey>>, DB>,
    HasOne<R, T>: WundergraphValue,
    <HasOne<R, T> as WundergraphValue>::PlaceHolder: Into<Option<R>>,
    R: WundergraphValue + Clone + Eq + Hash,
    for<'a> &'a T: Identifiable<Id = &'a R>,
    T: LoadingHandler<DB>,
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
    for<'a> &'a Option<R>: AsExpression<
        SqlTypeOf<NullableExpression<<T::Table as Table>::PrimaryKey>>,
        Expression = Bound<
            SqlTypeOf<NullableExpression<<T::Table as Table>::PrimaryKey>>,
            &'a Option<R>,
        >,
    >,
    <T::Table as Table>::PrimaryKey: QueryFragment<DB>,
    SqlTypeOf<<T::Table as Table>::PrimaryKey>: NotNull,
    DB::QueryBuilder: Default,
{
    fn new(elements: usize) -> Self {
        Self(PhantomData, Vec::with_capacity(elements))
    }

    fn resolve_value(
        &mut self,
        value: <HasOne<R, T> as WundergraphValue>::PlaceHolder,
        _selection: &juniper::LookAheadSelection<WundergraphScalarValue>,
    ) -> Result<Option<juniper::Value<WundergraphScalarValue>>, Error> {
        self.1.push(value.into());
        Ok(None)
    }

    fn finalize(
        self,
        conn: &impl Connection<Backend = DB>,
        selection: &juniper::LookAheadSelection<WundergraphScalarValue>,
    ) -> Result<Option<Vec<juniper::Value<WundergraphScalarValue>>>, Error> {
        use diesel::RunQueryDsl;

        let q = T::build_query(selection)?
            .filter(
                <T::Table as Table>::primary_key(&<T as HasTable>::table())
                    .nullable()
                    .eq_any(&self.1),
            )
            .select((
                <T::Table as Table>::primary_key(&<T as HasTable>::table()).nullable(),
                T::get_select(selection)?,
            ));

        let items = q.load::<(
            Option<R>,
            <T::FieldList as WundergraphFieldList<_, _, _>>::PlaceHolder,
        )>(conn)?;

        let (keys, placeholder): (Vec<_>, Vec<_>) = items.into_iter().unzip();

        let values = T::FieldList::resolve(placeholder, selection, T::FIELD_NAMES, conn)?;

        let map = keys
            .into_iter()
            .zip(values.into_iter())
            .collect::<HashMap<_, _>>();

        Ok(Some(
            self.1
                .iter()
                .map(|key| map.get(key).cloned().unwrap_or(juniper::Value::Null))
                .collect(),
        ))
    }
}

impl<R, T, DB> FieldValueResolver<Option<HasOne<R, T>>, DB> for HasOneResolver<R, T>
where
    DB: Backend,
    R: WundergraphValue + Clone + Hash + Eq,
    Self: FieldValueResolver<HasOne<R, T>, DB>,
    for<'a> &'a T: Identifiable<Id = &'a R>,
    R::PlaceHolder: Into<Option<R>>,
{
    fn new(elements: usize) -> Self {
        Self(PhantomData, Vec::with_capacity(elements))
    }

    fn resolve_value(
        &mut self,
        value: <Option<HasOne<R, T>> as WundergraphValue>::PlaceHolder,
        _selection: &juniper::LookAheadSelection<WundergraphScalarValue>,
    ) -> Result<Option<juniper::Value<WundergraphScalarValue>>, Error> {
        self.1.push(value.into());
        Ok(None)
    }

    fn finalize(
        self,
        conn: &impl Connection<Backend = DB>,
        selection: &juniper::LookAheadSelection<WundergraphScalarValue>,
    ) -> Result<Option<Vec<juniper::Value<WundergraphScalarValue>>>, Error> {
        <Self as FieldValueResolver<HasOne<R, T>, DB>>::finalize(self, conn, selection)
    }
}

impl<R, T, DB> ResolveWundergraphFieldValue<DB> for Option<HasOne<R, T>>
where
    HasOneResolver<R, T>:
        FieldValueResolver<HasOne<R, T>, DB> + FieldValueResolver<Option<HasOne<R, T>>, DB>,
    R: WundergraphValue + Clone + Eq + Hash,
    <HasOne<R, T> as WundergraphValue>::PlaceHolder: Into<Option<R>>,
    HasOne<R, T>: WundergraphValue,
    DB: Backend,
{
    type Resolver = HasOneResolver<R, T>;
}

impl<R, T, DB> ResolveWundergraphFieldValue<DB> for HasOne<R, T>
where
    HasOneResolver<R, T>: FieldValueResolver<HasOne<R, T>, DB>,
    R: WundergraphValue + Clone + Eq + Hash,
    Self::PlaceHolder: Into<Option<R>>,
    Self: WundergraphValue,
    DB: Backend,
{
    type Resolver = HasOneResolver<R, T>;
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

pub trait WundergraphResolvePlaceHolderList<R, DB: Backend> {
    fn resolve(
        self,
        get_name: impl Fn(usize) -> &'static str,
        selection: &juniper::LookAheadSelection<WundergraphScalarValue>,
        conn: &impl Connection<Backend = DB>,
    ) -> Result<Vec<juniper::Object<WundergraphScalarValue>>, Error>;
}

pub trait WundergraphFieldList<DB: Backend, Key, Table> {
    type PlaceHolder: Queryable<Self::SqlType, DB> + 'static;
    type SqlType: 'static;

    const TABLE_FIELD_COUNT: usize;
    const NON_TABLE_FIELD_COUNT: usize;

    fn resolve(
        placeholder: Vec<Self::PlaceHolder>,
        select: &juniper::LookAheadSelection<WundergraphScalarValue>,
        name_list: &'static [&'static str],
        conn: &impl Connection<Backend = DB>,
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

    fn push_field<T, O, DB, C>(
        &mut self,
        field: &'static str,
        selection: &juniper::LookAheadSelection<WundergraphScalarValue>,
        conn: &C,
    ) -> Result<(), Error>
    where
        DB: Backend,
        T: WundergraphResolveAssociation<K, O, DB>,
        C: Connection<Backend = DB>,
    {
        let values = T::resolve(selection, &self.keys, conn)?;

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
                                Some((field_idx, _value)) if idx == *field_idx => {
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

pub trait WundergraphResolveAssociations<K, Other, DB>
where
    K: Eq + Hash,
    DB: Backend,
{
    fn resolve(
        selection: &juniper::LookAheadSelection<WundergraphScalarValue>,
        get_name: impl Fn(usize) -> &'static str,
        get_keys: impl Fn() -> Vec<Option<K>>,
        conn: &impl Connection<Backend = DB>,
    ) -> Result<AssociationsReturn<K>, Error>;
}

impl<K, Other, DB> WundergraphResolveAssociations<K, Other, DB> for ()
where
    K: Eq + Hash,
    DB: Backend,
{
    fn resolve(
        _selection: &juniper::LookAheadSelection<WundergraphScalarValue>,
        _get_name: impl Fn(usize) -> &'static str,
        _get_keys: impl Fn() -> Vec<Option<K>>,
        _conn: &impl Connection<Backend = DB>,
    ) -> Result<AssociationsReturn<K>, Error> {
        Ok(AssociationsReturn::empty())
    }
}

pub trait WundergraphResolveAssociation<K, Other, DB: Backend> {
    fn resolve(
        selection: &juniper::LookAheadSelection<WundergraphScalarValue>,
        primary_keys: &[Option<K>],
        conn: &impl Connection<Backend = DB>,
    ) -> Result<HashMap<Option<K>, Vec<juniper::Value<WundergraphScalarValue>>>, Error>;
}

pub trait WundergraphBelongsTo<Other, DB>: LoadingHandler<DB>
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
        selection: &juniper::LookAheadSelection<WundergraphScalarValue>,
        keys: &[Option<Self::Key>],
        conn: &impl Connection<Backend = DB>,
    ) -> Result<HashMap<Option<Self::Key>, Vec<juniper::Value<WundergraphScalarValue>>>, Error>;

    fn build_response(
        res: Vec<(Option<Self::Key>, <Self::FieldList as WundergraphFieldList<DB, Self::PrimaryKeyIndex, Self::Table>>::PlaceHolder)>,
        selection: &juniper::LookAheadSelection<WundergraphScalarValue>,
        conn: &impl Connection<Backend = DB>,
    ) -> Result<HashMap<Option<Self::Key>, Vec<juniper::Value<WundergraphScalarValue>>>, Error>
    {
        let (keys, vals): (Vec<_>, Vec<_>) = res.into_iter().unzip();
        let vals = <<Self as LoadingHandler<DB>>::FieldList as WundergraphFieldList<
            DB,
            <Self as LoadingHandler<DB>>::PrimaryKeyIndex,
            <Self as HasTable>::Table,
        >>::resolve(
            vals,
            selection,
            <Self as LoadingHandler<DB>>::FIELD_NAMES,
            conn,
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

impl<T, K, Other, DB> WundergraphResolveAssociation<K, Other, DB> for HasMany<T>
where
    DB: Backend + 'static,
    T: WundergraphBelongsTo<Other, DB, Key = K>,
    K: Eq + Hash,
    T::Table: 'static,
    <T::Table as QuerySource>::FromClause: QueryFragment<DB>,
    DB::QueryBuilder: Default,
{
    fn resolve(
        selection: &juniper::LookAheadSelection<WundergraphScalarValue>,
        primary_keys: &[Option<K>],
        conn: &impl Connection<Backend = DB>,
    ) -> Result<HashMap<Option<K>, Vec<juniper::Value<WundergraphScalarValue>>>, Error> {
        T::resolve(selection, primary_keys, conn)
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


            impl<Back, $($T,)+ $($ST,)+> WundergraphResolvePlaceHolderList<($($ST,)*), Back> for Vec<($(PlaceHolder<$T>,)+)>
            where $($ST: WundergraphValue<PlaceHolder = PlaceHolder<$T>> +
                    ResolveWundergraphFieldValue<Back> ,)*
                  $($T: 'static,)*
                  Back: Backend,
            {
                fn resolve(
                    self,
                    get_name: impl Fn(usize) -> &'static str,
                    selection: &juniper::LookAheadSelection<WundergraphScalarValue>,
                    conn: &impl Connection<Backend = Back>,
                ) -> Result<Vec<juniper::Object<WundergraphScalarValue>>, Error>
                {
                    let mut resolver = (
                        $(<$ST as ResolveWundergraphFieldValue<Back>>::Resolver::new(self.len()),)*
                    );
                    let mut objs: Vec<juniper::Object<WundergraphScalarValue>>
                        = vec![juniper::Object::with_capacity(wundergraph_add_one_to_index!($($idx)*)-1); self.len()];

                    self.into_iter().zip(objs.iter_mut()).map(|(placeholder, obj)|{
                        $(
                            let name = get_name($idx);
                            if let Some(selection) = selection.select_child(name) {
                                if let Some(value) = resolver.$idx.resolve_value(
                                    placeholder.$idx,
                                    selection,
                                )? {
                                    obj.add_field(name, value);
                                }
                            }
                        )*
                        Ok(())
                    }).collect::<Result<Vec<_>, Error>>()?;
                    $(
                        let name = get_name($idx);
                        if let Some(selection) = selection.select_child(name) {
                            let vals = resolver.$idx.finalize(conn, selection)?;
                            if let Some(vals) = vals {
                                for (obj, val) in objs.iter_mut().zip(vals.into_iter()) {
                                    obj.add_field(name, val);
                                }
                            }
                        }
                    )*
                    Ok(objs)
                }

            }

            impl<Key, Back, Other, $($T,)*> WundergraphResolveAssociations<Key, Other, Back> for ($($T,)*)
            where Back: Backend,
                  Key: Eq + Hash,
                $($T: WundergraphResolveAssociation<Key, Other, Back>,)*

            {
                fn resolve(
                    selection: &juniper::LookAheadSelection<WundergraphScalarValue>,
                    get_name: impl Fn(usize) -> &'static str,
                    get_keys: impl Fn() -> Vec<Option<Key>>,
                    conn: &impl Connection<Backend = Back>
                ) -> Result<AssociationsReturn<Key>, Error>
                {
                    let mut ret = AssociationsReturn::empty();
                    $(
                        let name = get_name($idx);
                        if let Some(selection) = selection.select_child(name) {
                            ret.init(&get_keys);
                            ret.push_field::<$T, Other, Back, _>(name, selection, conn)?;
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

            impl<Back, Key, Table, $($T,)*> WundergraphFieldList<Back, Key, Table> for ($($T,)*)
            where Back: Backend,
                  ($($T,)*): FieldListExtractor + NonTableFieldExtractor,
                  <($($T,)*) as FieldListExtractor>::Out: WundergraphValue,
                  <<($($T,)*) as FieldListExtractor>::Out as WundergraphValue>::PlaceHolder: TupleIndex<Key> +
                      Queryable<<<($($T,)*) as FieldListExtractor>::Out as WundergraphValue>::SqlType, Back> + 'static,
            Vec<<<($($T,)*) as FieldListExtractor>::Out as WundergraphValue>::PlaceHolder>:
            WundergraphResolvePlaceHolderList<<($($T,)*) as FieldListExtractor>::Out, Back>,
            <<<($($T,)*) as FieldListExtractor>::Out as WundergraphValue>::PlaceHolder as TupleIndex<Key>>::Value: PlaceHolderMarker,
            <<<<($($T,)*) as FieldListExtractor>::Out as WundergraphValue>::PlaceHolder as TupleIndex<Key>>::Value as PlaceHolderMarker>::InnerType: Eq + Hash + Clone,
            <($($T,)*) as NonTableFieldExtractor>::Out: WundergraphResolveAssociations<<<<<($($T,)*) as FieldListExtractor>::Out as WundergraphValue>::PlaceHolder as TupleIndex<Key>>::Value as PlaceHolderMarker>::InnerType, Table, Back>,
            {
                type PlaceHolder = <<($($T,)*) as FieldListExtractor>::Out as WundergraphValue>::PlaceHolder;
                type SqlType = <<($($T,)*) as FieldListExtractor>::Out as WundergraphValue>::SqlType;

                const TABLE_FIELD_COUNT: usize = <($($T,)*) as FieldListExtractor>::FIELD_COUNT;
                const NON_TABLE_FIELD_COUNT: usize = <($($T,)*) as NonTableFieldExtractor>::FIELD_COUNT;

                fn resolve(
                    placeholder: Vec<Self::PlaceHolder>,
                    select: &juniper::LookAheadSelection<WundergraphScalarValue>,
                    name_list: &'static [&'static str],
                    conn: &impl Connection<Backend = Back>,
                ) -> Result<Vec<juniper::Value<WundergraphScalarValue>>, Error> {
                    let extern_values = {
                        let keys = ||{
                            placeholder.iter()
                                .map(TupleIndex::<Key>::get)
                                .map(|p| <_ as PlaceHolderMarker>::into_inner(p))
                                .collect::<Vec<_>>()
                        };

                        let name = |local_pos| {
                            <($($T,)*) as NonTableFieldExtractor>::map(
                                local_pos,
                                |pos| name_list[pos]
                            ).expect("Name is there")
                        };
                        <($($T,)*) as NonTableFieldExtractor>::Out::resolve(
                            select, name, keys, conn
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
                        conn
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
