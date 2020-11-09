use super::{FieldValueResolver, ResolveWundergraphFieldValue};
use crate::context::WundergraphContext;
use crate::error::Result;
use crate::query_builder::selection::fields::WundergraphFieldList;
use crate::query_builder::selection::filter::build_filter::BuildFilter;
use crate::query_builder::selection::offset::ApplyOffset;
use crate::query_builder::selection::{LoadingHandler, SqlTypeOfPlaceholder};
use crate::query_builder::types::{HasOne, WundergraphSqlValue};
use crate::scalar::WundergraphScalarValue;
use diesel::backend::Backend;
use diesel::dsl::SqlTypeOf;
use diesel::expression::bound::Bound;
use diesel::expression::nullable::Nullable as NullableExpression;
use diesel::expression::AsExpression;
use diesel::query_builder::{BoxedSelectStatement, QueryFragment};
use diesel::query_dsl::methods::BoxedDsl;
use diesel::serialize::ToSql;
use diesel::sql_types::{HasSqlType, NotNull};
use diesel::{
    AppearsOnTable, Connection, ExpressionMethods, Identifiable, NullableExpressionMethods,
    QueryDsl, QuerySource, Queryable, Table,
};
use juniper::{Executor, Selection};
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;

#[allow(missing_debug_implementations)]
pub struct HasOneResolver<R, T, Ctx> {
    values: Vec<Option<R>>,
    p: PhantomData<(T, Ctx)>,
}

impl<'a, R, T, DB, Ctx> FieldValueResolver<HasOne<R, T>, DB, Ctx> for HasOneResolver<R, T, Ctx>
where
    DB: Backend
        + ApplyOffset
        + HasSqlType<SqlTypeOfPlaceholder<T::FieldList, DB, T::PrimaryKeyIndex, T::Table, Ctx>>
        + HasSqlType<SqlTypeOf<NullableExpression<<T::Table as Table>::PrimaryKey>>>
        + 'static,
    Option<R>: Queryable<SqlTypeOf<NullableExpression<<T::Table as Table>::PrimaryKey>>, DB>
        + ToSql<SqlTypeOf<NullableExpression<<T::Table as Table>::PrimaryKey>>, DB>,
    HasOne<R, T>: WundergraphSqlValue,
    <HasOne<R, T> as WundergraphSqlValue>::PlaceHolder: Into<Option<R>>,
    R: WundergraphSqlValue + Clone + Eq + Hash,
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
    <T::Table as Table>::PrimaryKey: QueryFragment<DB> + Default,
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
        value: <HasOne<R, T> as WundergraphSqlValue>::PlaceHolder,
        _look_ahead: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        _selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        _executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    ) -> Result<Option<juniper::Value<WundergraphScalarValue>>> {
        self.values.push(value.into());
        Ok(None)
    }

    fn finalize(
        self,
        global_args: &[juniper::LookAheadArgument<WundergraphScalarValue>],
        look_ahead: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    ) -> Result<Option<Vec<juniper::Value<WundergraphScalarValue>>>> {
        use diesel::RunQueryDsl;
        let conn = executor.context().get_connection();
        let q = T::build_query(global_args, look_ahead)?
            .filter(
                <T::Table as Table>::PrimaryKey::default()
                    .nullable()
                    .eq_any(&self.values),
            )
            .select((
                <T::Table as Table>::PrimaryKey::default().nullable(),
                T::get_select(look_ahead)?,
            ));

        #[cfg(feature = "debug")]
        {
            log::debug!("{:?}", diesel::debug_query(&q));
        }
        let items = q.load::<(
            Option<R>,
            <T::FieldList as WundergraphFieldList<_, _, _, Ctx>>::PlaceHolder,
        )>(conn)?;

        let (keys, placeholder): (Vec<_>, Vec<_>) = items.into_iter().unzip();

        let values = T::FieldList::resolve(
            placeholder,
            global_args,
            look_ahead,
            selection,
            T::FIELD_NAMES,
            executor,
        )?;

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
    R: WundergraphSqlValue + Clone + Hash + Eq,
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
        value: <Option<HasOne<R, T>> as WundergraphSqlValue>::PlaceHolder,
        _look_ahead: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        _selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        _executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    ) -> Result<Option<juniper::Value<WundergraphScalarValue>>> {
        self.values.push(value.into());
        Ok(None)
    }

    fn finalize(
        self,
        global_args: &[juniper::LookAheadArgument<WundergraphScalarValue>],
        look_ahead: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    ) -> Result<Option<Vec<juniper::Value<WundergraphScalarValue>>>> {
        <Self as FieldValueResolver<HasOne<R, T>, DB, Ctx>>::finalize(
            self,
            global_args,
            look_ahead,
            selection,
            executor,
        )
    }
}

impl<T, DB, Ctx, R> ResolveWundergraphFieldValue<DB, Ctx> for Option<HasOne<R, T>>
where
    DB: Backend,
    R: WundergraphSqlValue + Clone + Eq + Hash,
    HasOneResolver<R, T, Ctx>: FieldValueResolver<Option<HasOne<R, T>>, DB, Ctx>,
    Self::PlaceHolder: Into<Option<R>>,
    Self: WundergraphSqlValue,
{
    type Resolver = HasOneResolver<R, T, Ctx>;
}

impl<R, T, DB, Ctx> ResolveWundergraphFieldValue<DB, Ctx> for HasOne<R, T>
where
    HasOneResolver<R, T, Ctx>: FieldValueResolver<HasOne<R, T>, DB, Ctx>,
    R: WundergraphSqlValue + Clone + Eq + Hash,
    Self::PlaceHolder: Into<Option<R>>,
    Self: WundergraphSqlValue,
    DB: Backend,
{
    type Resolver = HasOneResolver<R, T, Ctx>;
}
