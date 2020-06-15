use super::{HandleBatchInsert, HandleInsert};
use crate::context::WundergraphContext;
use crate::helper::UnRef;
use crate::query_builder::selection::fields::WundergraphFieldList;
use crate::query_builder::selection::filter::build_filter::BuildFilter;
use crate::query_builder::selection::order::BuildOrder;
use crate::query_builder::selection::query_modifier::QueryModifier;
use crate::query_builder::selection::select::BuildSelect;
use crate::query_builder::selection::{LoadingHandler, SqlTypeOfPlaceholder};
use crate::scalar::WundergraphScalarValue;
use diesel::associations::HasTable;
use diesel::deserialize::FromSql;
use diesel::dsl::SqlTypeOf;
use diesel::expression::{Expression, NonAggregate, SelectableExpression};
use diesel::insertable::CanInsertInSingleQuery;
use diesel::mysql::Mysql;
use diesel::query_builder::{BoxedSelectStatement, QueryFragment};
use diesel::query_dsl::methods::{BoxedDsl, FilterDsl};
use diesel::sql_types::{Bigint, HasSqlType, Integer};
use diesel::{no_arg_sql_function, EqAll, Identifiable};
use diesel::{AppearsOnTable, Connection, Insertable, RunQueryDsl, Table};
use juniper::{ExecutionResult, Executor, Selection, Value};
use std::convert::TryFrom;

// https://dev.mysql.com/doc/refman/8.0/en/getting-unique-id.html
diesel::no_arg_sql_function!(LAST_INSERT_ID, Bigint);

impl<I, Ctx, L, T> HandleInsert<L, I, Mysql, Ctx> for T
where
    T: Table + HasTable<Table = T> + 'static,
    T::FromClause: QueryFragment<Mysql>,
    L: LoadingHandler<Mysql, Ctx, Table = T> + 'static,
    L::Columns: BuildOrder<T, Mysql>
        + BuildSelect<T, Mysql, SqlTypeOfPlaceholder<L::FieldList, Mysql, L::PrimaryKeyIndex, T, Ctx>>,
    Ctx: WundergraphContext + QueryModifier<L, Mysql>,
    Ctx::Connection: Connection<Backend = Mysql>,
    L::FieldList: WundergraphFieldList<Mysql, L::PrimaryKeyIndex, T, Ctx>,
    I: Insertable<T>,
    I::Values: QueryFragment<Mysql> + CanInsertInSingleQuery<Mysql>,
    T::PrimaryKey: QueryFragment<Mysql> + Default,
    T: BoxedDsl<
        'static,
        Mysql,
        Output = BoxedSelectStatement<'static, SqlTypeOf<<T as Table>::AllColumns>, T, Mysql>,
    >,
    <Ctx::Connection as Connection>::Backend: HasSqlType<SqlTypeOf<T::PrimaryKey>>
        + HasSqlType<SqlTypeOfPlaceholder<L::FieldList, Mysql, L::PrimaryKeyIndex, T, Ctx>>,
    <L::Filter as BuildFilter<Mysql>>::Ret: AppearsOnTable<T>,
    T::PrimaryKey: EqAll<i32>,
    T::PrimaryKey: Expression<SqlType = diesel::sql_types::Integer>,
    &'static L: Identifiable,
    <&'static L as Identifiable>::Id: UnRef<'static, UnRefed = i32>,
    <T::PrimaryKey as EqAll<i32>>::Output:
        SelectableExpression<T> + NonAggregate + QueryFragment<Mysql> + 'static,
{
    fn handle_insert(
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
        insertable: I,
    ) -> ExecutionResult<WundergraphScalarValue> {
        handle_single_insert(selection, executor, insertable)
    }
}

impl<I, Ctx, L, T> HandleBatchInsert<L, I, Mysql, Ctx> for T
where
    T: Table + HasTable<Table = T> + 'static,
    T::FromClause: QueryFragment<Mysql>,
    L: LoadingHandler<Mysql, Ctx, Table = T> + 'static,
    L::Columns: BuildOrder<T, Mysql>
        + BuildSelect<T, Mysql, SqlTypeOfPlaceholder<L::FieldList, Mysql, L::PrimaryKeyIndex, T, Ctx>>,
    Ctx: WundergraphContext + QueryModifier<L, Mysql>,
    Ctx::Connection: Connection<Backend = Mysql>,
    L::FieldList: WundergraphFieldList<Mysql, L::PrimaryKeyIndex, T, Ctx>,
    I: Insertable<T>,
    I::Values: QueryFragment<Mysql> + CanInsertInSingleQuery<Mysql>,
    T::PrimaryKey: QueryFragment<Mysql> + Default,
    T: BoxedDsl<
        'static,
        Mysql,
        Output = BoxedSelectStatement<'static, SqlTypeOf<<T as Table>::AllColumns>, T, Mysql>,
    >,
    <Ctx::Connection as Connection>::Backend: HasSqlType<SqlTypeOf<T::PrimaryKey>>
        + HasSqlType<SqlTypeOfPlaceholder<L::FieldList, Mysql, L::PrimaryKeyIndex, T, Ctx>>,
    <L::Filter as BuildFilter<Mysql>>::Ret: AppearsOnTable<T>,
    T::PrimaryKey: EqAll<i32>,
    T::PrimaryKey: Expression<SqlType = diesel::sql_types::Integer>,
    &'static L: Identifiable,
    <&'static L as Identifiable>::Id: UnRef<'static, UnRefed = i32>,
    <T::PrimaryKey as EqAll<i32>>::Output:
        SelectableExpression<T> + NonAggregate + QueryFragment<Mysql> + 'static,
{
    fn handle_batch_insert(
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
        batch: Vec<I>,
    ) -> ExecutionResult<WundergraphScalarValue> {
        let r = batch
            .into_iter()
            .map(|i| handle_single_insert(selection, executor, i))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Value::List(r))
    }
}

fn handle_single_insert<L, I, T, Ctx>(
    selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
    executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    insertable: I,
) -> ExecutionResult<WundergraphScalarValue>
where
    L: LoadingHandler<Mysql, Ctx, Table = T> + 'static,
    L::FieldList: WundergraphFieldList<Mysql, L::PrimaryKeyIndex, T, Ctx>,
    <L::Filter as BuildFilter<Mysql>>::Ret: AppearsOnTable<T>,
    L::Columns: BuildOrder<T, Mysql>
        + BuildSelect<T, Mysql, SqlTypeOfPlaceholder<L::FieldList, Mysql, L::PrimaryKeyIndex, T, Ctx>>,
    &'static L: Identifiable,
    I: Insertable<T>,
    I::Values: QueryFragment<Mysql> + CanInsertInSingleQuery<Mysql>,
    Ctx: WundergraphContext + QueryModifier<L, Mysql>,
    Ctx::Connection: Connection<Backend = Mysql>,
    <Ctx::Connection as Connection>::Backend: HasSqlType<SqlTypeOf<T::PrimaryKey>>
        + HasSqlType<SqlTypeOfPlaceholder<L::FieldList, Mysql, L::PrimaryKeyIndex, T, Ctx>>,
    T: Table + HasTable<Table = T> + 'static,
    T::FromClause: QueryFragment<Mysql>,
    T: BoxedDsl<
        'static,
        Mysql,
        Output = BoxedSelectStatement<'static, SqlTypeOf<<T as Table>::AllColumns>, T, Mysql>,
    >,
    T::PrimaryKey: EqAll<i32>,
    T::PrimaryKey: Expression<SqlType = Integer>,
    T::PrimaryKey: QueryFragment<Mysql> + Default,
    <T::PrimaryKey as EqAll<i32>>::Output:
        SelectableExpression<T> + NonAggregate + QueryFragment<Mysql> + 'static,
    i32: FromSql<Integer, Mysql>,
{
    let ctx = executor.context();
    let conn = ctx.get_connection();
    let look_ahead = executor.look_ahead();
    insertable.insert_into(T::table()).execute(conn).unwrap();
    let last_insert_id: i64 = diesel::select(LAST_INSERT_ID).first(conn)?;
    let last_insert_id = i32::try_from(last_insert_id)?;
    let q = L::build_query(&[], &look_ahead)?;
    let q = FilterDsl::filter(q, T::PrimaryKey::default().eq_all(last_insert_id));
    let items = L::load(&look_ahead, selection, executor, q)?;
    Ok(items.into_iter().next().unwrap_or(Value::Null))
}
