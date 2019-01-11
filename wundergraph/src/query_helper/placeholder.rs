use diesel::associations::HasTable;
use diesel::backend::Backend;
use diesel::connection::Connection;
use diesel::deserialize::{self, FromSql};
use diesel::dsl::SqlTypeOf;
use diesel::expression::nullable::Nullable as NullableExpression;
use diesel::expression::{AsExpression, NonAggregate};
use diesel::query_builder::{BoxedSelectStatement, QueryFragment};
use diesel::query_dsl::methods::BoxedDsl;
use diesel::sql_types::{BigInt, Bool, Float4, Float8, Integer, SmallInt, Text};
use diesel::sql_types::{HasSqlType, NotNull, Nullable};
use diesel::{
    AppearsOnTable, ExpressionMethods, Identifiable, NullableExpressionMethods, QueryDsl,
    QuerySource, Queryable, SelectableExpression, Table,
};
use failure::Error;
use filter::build_filter::BuildFilter;
use query_helper::tuple::{FamilyLt, TupleIndex};
use query_helper::{HasMany, HasOne};
use scalar::WundergraphScalarValue;
use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use LoadingHandler2;

use juniper::LookAheadMethods;

pub trait PlaceHolderMarker {
    type InnerType;

    fn into_inner(&self) -> Option<&Self::InnerType>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, FromSqlRow)]
pub struct PlaceHolder<T>(Option<T>);

impl<T> PlaceHolderMarker for PlaceHolder<T> {
    type InnerType = T;

    fn into_inner(&self) -> Option<&T> {
        self.0.as_ref()
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
    T: FromSql<ST, DB>,
    ST: NotNull,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        if bytes.is_some() {
            T::from_sql(bytes).map(Some).map(PlaceHolder)
        } else {
            Ok(PlaceHolder(None))
        }
    }
}

pub type SqlTypeOfPlaceholder<T, DB, K, Table> = <T as WundergraphFieldList<DB, K, Table>>::SqlType;

pub trait FieldValueResolver<T, DB>
where
    T: WundergraphValue,
    T::PlaceHolder: PlaceHolderMarker,
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
    T::PlaceHolder: PlaceHolderMarker,
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

pub trait ResolveWundergraphFieldValue<DB: Backend>: WundergraphValue + Sized
where
    Self::PlaceHolder: PlaceHolderMarker,
{
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
    Self: WundergraphValue<PlaceHolder = PlaceHolder<T>>,
    T: ResolveWundergraphFieldValue<DB> + 'static,
    T::PlaceHolder: PlaceHolderMarker,
    juniper::Value<WundergraphScalarValue>: From<Self>,
{
    type Resolver = DirectResolver<Self>;
}

impl<T> IntoValue for Option<T>
where
    Self: WundergraphValue<PlaceHolder = PlaceHolder<T>>,
    juniper::Value<WundergraphScalarValue>: From<Self>,
    T: 'static,
{
    fn resolve(placeholder: Self::PlaceHolder) -> juniper::Value<WundergraphScalarValue> {
        placeholder.0.into()
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
    + HasSqlType<SqlTypeOf<<T::Table as Table>::PrimaryKey>>  + 'static,
    HasOne<R, T>: WundergraphValue,
    <HasOne<R, T> as WundergraphValue>::PlaceHolder: PlaceHolderMarker + Into<Option<R>>,
    R: WundergraphValue + Clone + Eq + Hash,
    <<HasOne<R, T> as WundergraphValue>::PlaceHolder as PlaceHolderMarker>::InnerType: Queryable<SqlTypeOf<<T::Table as Table>::PrimaryKey>, DB>,
    <R as WundergraphValue>::PlaceHolder: PlaceHolderMarker,
    for<'a> &'a T: Identifiable<Id = &'a R>,
    T: LoadingHandler2<DB>,
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
    Option<R>: AsExpression<SqlTypeOf<NullableExpression<<T::Table as Table>::PrimaryKey>>>,
    <Option<R> as AsExpression<SqlTypeOf<NullableExpression<<T::Table as Table>::PrimaryKey>>>>::Expression: AppearsOnTable<T::Table> + QueryFragment<DB>,
    <T::Table as Table>::PrimaryKey: QueryFragment<DB>,
    SqlTypeOf<<T::Table as Table>::PrimaryKey>: NotNull,
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

        let q = T::build_query(selection)?.filter(
            <T::Table as Table>::primary_key(&<T as HasTable>::table())
                .nullable()
                .eq_any(self.1),
        );

        Ok(Some(T::load(selection, conn, q)?))
    }
}

impl<R, T, DB> ResolveWundergraphFieldValue<DB> for HasOne<R, T>
where
    DB: Backend + HasSqlType<SqlTypeOfPlaceholder<T::FieldList, DB, T::PrimaryKeyIndex, T::Table>> + HasSqlType<SqlTypeOf<<T::Table as Table>::PrimaryKey>> + 'static,
    Self: WundergraphValue,
    Self::PlaceHolder: Into<Option<R>> + PlaceHolderMarker,
    R: WundergraphValue + Clone + Eq + Hash,
   <<HasOne<R, T> as WundergraphValue>::PlaceHolder as PlaceHolderMarker>::InnerType: Queryable<SqlTypeOf<<T::Table as Table>::PrimaryKey>, DB>,
    R::PlaceHolder: PlaceHolderMarker,
    for<'a> &'a T: Identifiable<Id = &'a R>,
    T: LoadingHandler2<DB>,
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
     SqlTypeOf<<T::Table as Table>::PrimaryKey>: NotNull,
      Option<R>: AsExpression<SqlTypeOf<NullableExpression<<T::Table as Table>::PrimaryKey>>>,
     <Option<R> as AsExpression<SqlTypeOf<NullableExpression<<T::Table as Table>::PrimaryKey>>>>::Expression:
         AppearsOnTable<T::Table> + NonAggregate + QueryFragment<DB>,
     <T::Table as Table>::PrimaryKey: QueryFragment<DB>,
    <T::Filter as BuildFilter<DB>>::Ret: AppearsOnTable<T::Table>,
<T::Table as QuerySource>::FromClause: QueryFragment<DB>,
Option<R>: AsExpression<SqlTypeOf<NullableExpression<<T::Table as Table>::PrimaryKey>>>,
<Option<R> as AsExpression<SqlTypeOf<NullableExpression<<T::Table as Table>::PrimaryKey>>>>::Expression: AppearsOnTable<T::Table> + QueryFragment<DB>,
{
    type Resolver = HasOneResolver<R, T>;
}

pub trait CollectTableFields<T> {
    type Fields;
    const SQL_NAME_INDICES: &'static [usize];
}

pub trait CollectNonTableFields<T> {
    type Fields;
    const FIELD_INDICES: &'static [usize];
}

impl<T> CollectTableFields<()> for T
where
    T: WundergraphValue,
{
    type Fields = (T,);
    const SQL_NAME_INDICES: &'static [usize] = &[0];
}

impl<T> CollectNonTableFields<()> for T
where
    T: WundergraphValue,
{
    type Fields = ();
    const FIELD_INDICES: &'static [usize] = &[];
}

impl<R> CollectTableFields<()> for HasMany<R> {
    type Fields = ();
    const SQL_NAME_INDICES: &'static [usize] = &[];
}

impl<R> CollectNonTableFields<()> for HasMany<R> {
    type Fields = (Self,);
    const FIELD_INDICES: &'static [usize] = &[0];
}

pub trait WundergraphResolvePlaceHolderList<R, DB: Backend> {
    fn resolve(
        self,
        name_list: &'static [&'static str],
        name_indices: &'static [usize],
        selection: &juniper::LookAheadSelection<WundergraphScalarValue>,
        conn: &impl Connection<Backend = DB>,
    ) -> Result<Vec<juniper::Object<WundergraphScalarValue>>, Error>;
}

pub trait WundergraphFieldList<DB: Backend, Key, Table> {
    type PlaceHolder: TupleIndex<Key> + 'static;
    type SqlType: 'static;

    const SQL_NAME_INDICES: &'static [usize];
    const NON_SQL_NAME_INDICES: &'static [usize];

    fn resolve(
        placeholder: Vec<Self::PlaceHolder>,
        select: &juniper::LookAheadSelection<WundergraphScalarValue>,
        name_list: &'static [&'static str],
        conn: &impl Connection<Backend = DB>,
    ) -> Result<Vec<juniper::Value<WundergraphScalarValue>>, Error>;
}

#[derive(Debug)]
pub struct AssociationsReturn<K: Eq + Hash> {
    keys: Vec<K>,
    fields: Vec<&'static str>,
    values: HashMap<K, Vec<(usize, Vec<juniper::Value<WundergraphScalarValue>>)>>,
}

impl<K: Eq + Hash> AssociationsReturn<K> {
    fn empty() -> Self {
        Self {
            keys: Vec::new(),
            fields: Vec::new(),
            values: HashMap::new(),
        }
    }

    fn init(&mut self, get_keys: &impl Fn() -> Vec<K>) {
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
                                obj.add_field(field_name.to_owned(), juniper::Value::List(value));
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

pub trait WundergraphResolveAssociations<K, Other, DB>
where
    K: Eq + Hash,
    DB: Backend,
{
    fn resolve(
        selection: &juniper::LookAheadSelection<WundergraphScalarValue>,
        name_list: &'static [&'static str],
        name_indices: &'static [usize],
        get_keys: impl Fn() -> Vec<K>,
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
        _name_list: &'static [&'static str],
        _name_indices: &'static [usize],
        _get_keys: impl Fn() -> Vec<K>,
        _conn: &impl Connection<Backend = DB>,
    ) -> Result<AssociationsReturn<K>, Error> {
        Ok(AssociationsReturn::empty())
    }
}

pub trait WundergraphResolveAssociation<K, Other, DB: Backend> {
    fn resolve(
        selection: &juniper::LookAheadSelection<WundergraphScalarValue>,
        primary_keys: &[K],
        conn: &impl Connection<Backend = DB>,
    ) -> Result<HashMap<K, Vec<juniper::Value<WundergraphScalarValue>>>, Error>;
}

pub trait WundergraphBelongsTo<Other, K, DB>: LoadingHandler2<DB>
where
    DB: Backend + 'static,
    Self::Table: 'static,
    K: Eq + Hash,
    <Self::Table as QuerySource>::FromClause: QueryFragment<DB>,
{
    type ForeignKeyColumn: Default
        + NonAggregate
        + SelectableExpression<Self::Table>
        + QueryFragment<DB>;

    fn resolve(
        selection: &juniper::LookAheadSelection<WundergraphScalarValue>,
        keys: &[K],
        conn: &impl Connection<Backend = DB>,
    ) -> Result<HashMap<K, Vec<juniper::Value<WundergraphScalarValue>>>, Error>;

    fn build_response(
        res: Vec<(K, <Self as LoadingHandler2<DB>>::PlaceHolder)>,
        selection: &juniper::LookAheadSelection<WundergraphScalarValue>,
        conn: &impl Connection<Backend = DB>,
    ) -> Result<HashMap<K, Vec<juniper::Value<WundergraphScalarValue>>>, Error> {
        let (keys, vals): (Vec<_>, Vec<_>) = res.into_iter().unzip();
        let vals = <<Self as LoadingHandler2<DB>>::FieldList as WundergraphFieldList<
            DB,
            <Self as LoadingHandler2<DB>>::PrimaryKeyIndex,
            <Self as HasTable>::Table,
        >>::resolve(
            vals,
            selection,
            <Self as LoadingHandler2<DB>>::FIELD_NAMES,
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
    T: WundergraphBelongsTo<Other, K, DB>,
    K: Eq + Hash,
    T::Table: 'static,
    <T::Table as QuerySource>::FromClause: QueryFragment<DB>,
{
    fn resolve(
        selection: &juniper::LookAheadSelection<WundergraphScalarValue>,
        primary_keys: &[K],
        conn: &impl Connection<Backend = DB>,
    ) -> Result<HashMap<K, Vec<juniper::Value<WundergraphScalarValue>>>, Error> {
        T::resolve(selection, primary_keys, conn)
    }
}

macro_rules! expand_field_list {
    (
        params = {$($T: ident,)*},
        indices = {$($idx:tt)*}
    ) => {
        expand_field_list!{
            params = {$($T,)*},
            indices = {$($idx)*},
            where_clause = {},
            table_pusher = {()},
            non_table_pusher = {()},
            old_params = {}
        }
    };
    (
        params = {$H: ident, $($T: ident,)+},
        indices = {$($idx:tt)*},
        where_clause = {$($where:tt)*},
        table_pusher = {$($table_pusher:tt)*},
        non_table_pusher = {$($non_table_pusher: tt)*},
        old_params = {$($OldT: ident,)*}
    ) => {
        expand_field_list!{
            params = {$($T,)*},
            indices = {$($idx)*},
            where_clause = {$($where)* $H:
                            CollectTableFields<$($table_pusher)*> +
                            CollectNonTableFields<$($non_table_pusher)*>,},
            table_pusher = {<$H as CollectTableFields<$($table_pusher)*>>::Fields},
            non_table_pusher = {<$H as CollectNonTableFields<$($non_table_pusher)*>>::Fields},
            old_params = {$($OldT,)* $H,}
        }
    };
    (
        params = {$H: ident,},
        indices = {$($idx:tt)*},
        where_clause = {$($where: tt)*},
        table_pusher = {$($table_pusher:tt)*},
        non_table_pusher = {$($non_table_pusher:tt)*},
        old_params = {$($OldT: ident,)*}
    ) => {
        impl<Back, Key, Table, $($OldT,)* $H> WundergraphFieldList<Back, Key, Table> for ($($OldT,)* $H,)
        where Back: Backend,
              $($where)*
              $H: CollectTableFields<$($table_pusher)*> +
                  CollectNonTableFields<$($non_table_pusher)*>,
              <$H as CollectTableFields<$($table_pusher)*>>::Fields: WundergraphValue,
              Vec<<<$H as CollectTableFields<$($table_pusher)*>>::Fields as WundergraphValue>::PlaceHolder>:
        WundergraphResolvePlaceHolderList<<$H as CollectTableFields<$($table_pusher)*>>::Fields, Back>,
        <<$H as CollectTableFields<$($table_pusher)*>>::Fields as WundergraphValue>::PlaceHolder: TupleIndex<Key>,
        for<'a> <<<$H as CollectTableFields<$($table_pusher)*>>::Fields as WundergraphValue>::PlaceHolder as TupleIndex<Key>>::RetValue: FamilyLt<'a, Out = &'a <<<$H as CollectTableFields<$($table_pusher)*>>::Fields as WundergraphValue>::PlaceHolder as TupleIndex<Key>>::Value>,
        <<<$H as CollectTableFields<$($table_pusher)*>>::Fields as WundergraphValue>::PlaceHolder as TupleIndex<Key>>::Value: PlaceHolderMarker,
        <<<<$H as CollectTableFields<$($table_pusher)*>>::Fields as WundergraphValue>::PlaceHolder as TupleIndex<Key>>::Value as PlaceHolderMarker>::InnerType: Eq + Hash + Clone,
       <$H as CollectNonTableFields<$($non_table_pusher)*>>::Fields:
       WundergraphResolveAssociations<<<<<$H as CollectTableFields<$($table_pusher)*>>::Fields as WundergraphValue>::PlaceHolder as TupleIndex<Key>>::Value as PlaceHolderMarker>::InnerType, Table, Back>
        {
            type PlaceHolder = <
                <$H as CollectTableFields<$($table_pusher)*>>::Fields as WundergraphValue
                >::PlaceHolder;
            type SqlType = <
                <$H as CollectTableFields<$($table_pusher)*>>::Fields as WundergraphValue>::SqlType;

            const SQL_NAME_INDICES: &'static [usize] =
                <$H as CollectTableFields<$($table_pusher)*>>::SQL_NAME_INDICES;
            const NON_SQL_NAME_INDICES: &'static [usize] =
                <$H as CollectNonTableFields<$($non_table_pusher)*>>::FIELD_INDICES;

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
                            .map(|p| <_ as PlaceHolderMarker>::into_inner(p).unwrap())
                            .cloned()
                            .collect::<Vec<_>>()
                    };
                    <$H as CollectNonTableFields<$($non_table_pusher)*>>::Fields::resolve(
                        select, name_list, Self::NON_SQL_NAME_INDICES, keys, conn
                    )?
                };

                let objs = placeholder.resolve(
                    name_list,
                    <Self as WundergraphFieldList<Back, Key, Table>>::SQL_NAME_INDICES,
                    select,
                    conn
                )?;


                Ok(extern_values.merge_with_object_list(objs))
            }
        }
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

macro_rules! wundergraph_value_impl {
    ($(
        $Tuple:tt {
            $(($idx:tt) -> $T:ident, $ST: ident, $TT: ident,) +
        }
    )+) => {
        $(
            impl<Value, $($T,)+> CollectTableFields<($($T,)+)> for Value
            where
                Value: WundergraphValue
            {
                type Fields = ($($T,)+ Value,);

                const SQL_NAME_INDICES: &'static [usize] = &[
                    $($idx,)* wundergraph_add_one_to_index!($($idx)*)
                ];
            }

            impl<Value, $($T,)+> CollectNonTableFields<($($T,)+)> for Value
            where
                Value: WundergraphValue
            {
                type Fields = ($($T,)+);
                const FIELD_INDICES: &'static [usize] = &[$($idx,)*];
            }


            impl<Remote, $($T,)+> CollectTableFields<($($T,)+)> for HasMany<Remote> {
                type Fields = ($($T,)+);
                const SQL_NAME_INDICES: &'static [usize] = & [$($idx,)*];
            }

            impl<Remote, $($T,)+> CollectNonTableFields<($($T,)+)> for HasMany<Remote>
            {
                type Fields = ($($T,)+ Remote,);

                const FIELD_INDICES: &'static [usize] = &[
                    $($idx,)* wundergraph_add_one_to_index!($($idx)*)
                ];
            }

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
                    name_list: &'static [&'static str],
                    name_indices: &'static [usize],
                    selection: &juniper::LookAheadSelection<WundergraphScalarValue>,
                    conn: &impl Connection<Backend = Back>,
                ) -> Result<Vec<juniper::Object<WundergraphScalarValue>>, Error>
                {
                    let mut resolver = (
                        $(<$ST as ResolveWundergraphFieldValue<Back>>::Resolver::new(self.len()),)*
                    );
                    let mut objs: Vec<juniper::Object<WundergraphScalarValue>>
                        = vec![juniper::Object::with_capacity(name_list.len()); self.len()];

                    self.into_iter().zip(objs.iter_mut()).map(|(placeholder, obj)|{
                        $(
                            if let Some(selection) = selection.select_child(
                                name_list[name_indices[$idx]]
                            ) {

                                if let Some(value) = resolver.$idx.resolve_value(
                                    placeholder.$idx,
                                    selection,
                                )? {
                                    obj.add_field(name_list[name_indices[$idx]], value);
                                }
                            }
                        )*
                        Ok(())
                    }).collect::<Result<Vec<_>, Error>>()?;
                    $(
                        if let Some(selection) = selection.select_child(
                            name_list[name_indices[$idx]]
                        ) {
                            let vals = resolver.$idx.finalize(conn, selection)?;
                            if let Some(vals) = vals {
                                for (obj, val) in objs.iter_mut().zip(vals.into_iter()) {
                                    obj.add_field(name_list[name_indices[$idx]], val);
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
                    name_list: &'static [&'static str],
                    name_indices: &'static [usize],
                    get_keys: impl Fn() -> Vec<Key>,
                    conn: &impl Connection<Backend = Back>
                ) -> Result<AssociationsReturn<Key>, Error>
                {
                    let mut ret = AssociationsReturn::empty();
                    $(
                        if let Some(selection) = selection.select_child(
                            name_list[name_indices[$idx]]
                        ) {
                            ret.init(&get_keys);
                            ret.push_field::<$T, Other, Back, _>(name_list[name_indices[$idx]], selection, conn)?;
                        }
                    )*
                    Ok(ret)
                }
            }

            // impl<$($T,)*> PlaceHolderMarker for ($(PlaceHolder<$T>,)*)
            // where $(PlaceHolder<$T>: PlaceHolderMarker,)*
            // {
            //     type InnerType = ($(<PlaceHolder<$T> as PlaceHolderMarker>::InnerType,)*);

            //     fn into_inner(self) -> &
            // }

            expand_field_list! {
                params = {$($T,)*},
                indices = {$($idx)*}
            }

        )+
    }
}

__diesel_for_each_tuple!(wundergraph_value_impl);
