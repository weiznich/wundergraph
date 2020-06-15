use crate::mutations::{NewAppearsIn, NewFriend, NewHero, NewHomeWorld, NewSpecies};
use diesel::associations::HasTable;
use diesel::deserialize::FromSql;
use diesel::dsl::SqlTypeOf;
use diesel::expression::{Expression, NonAggregate, SelectableExpression};
use diesel::insertable::CanInsertInSingleQuery;
use diesel::mysql::Mysql;
use diesel::query_builder::{BoxedSelectStatement, QueryFragment};
use diesel::query_dsl::methods::{BoxedDsl, FilterDsl};
use diesel::sql_types::{Bigint, HasSqlType, Integer};
use diesel::{
    no_arg_sql_function, AppearsOnTable, Connection, EqAll, Identifiable, Insertable, RunQueryDsl,
    Table,
};
use juniper::{ExecutionResult, Executor, Selection, Value};
use std::convert::TryFrom;
use wundergraph::query_builder::mutations::{HandleBatchInsert, HandleInsert};
use wundergraph::query_builder::selection::fields::WundergraphFieldList;
use wundergraph::query_builder::selection::filter::BuildFilter;
use wundergraph::query_builder::selection::order::BuildOrder;
use wundergraph::query_builder::selection::select::BuildSelect;
use wundergraph::query_builder::selection::{LoadingHandler, QueryModifier, SqlTypeOfPlaceholder};
use wundergraph::scalar::WundergraphScalarValue;
use wundergraph::WundergraphContext;

diesel::no_arg_sql_function!(LAST_INSERT_ID, Bigint);

fn handle_i32_pk_insert<L, I, T, Ctx>(
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

impl<L, Ctx> HandleInsert<L, NewHomeWorld, Mysql, Ctx> for super::home_worlds::table
where
    L: LoadingHandler<Mysql, Ctx, Table = super::home_worlds::table> + 'static,
    L::FieldList: WundergraphFieldList<Mysql, L::PrimaryKeyIndex, super::home_worlds::table, Ctx>,
    <L::Filter as BuildFilter<Mysql>>::Ret: AppearsOnTable<super::home_worlds::table>,
    L::Columns: BuildOrder<super::home_worlds::table, Mysql>
        + BuildSelect<
            super::home_worlds::table,
            Mysql,
            SqlTypeOfPlaceholder<
                L::FieldList,
                Mysql,
                L::PrimaryKeyIndex,
                super::home_worlds::table,
                Ctx,
            >,
        >,
    &'static L: Identifiable,
    Ctx: WundergraphContext + QueryModifier<L, Mysql>,
    Ctx::Connection: Connection<Backend = Mysql>,
    <Ctx::Connection as Connection>::Backend: HasSqlType<SqlTypeOf<super::home_worlds::id>>
        + HasSqlType<
            SqlTypeOfPlaceholder<
                L::FieldList,
                Mysql,
                L::PrimaryKeyIndex,
                super::home_worlds::table,
                Ctx,
            >,
        >,
{
    fn handle_insert(
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
        insertable: NewHomeWorld,
    ) -> ExecutionResult<WundergraphScalarValue> {
        handle_i32_pk_insert(selection, executor, insertable)
    }
}

impl<L, Ctx> HandleBatchInsert<L, NewHomeWorld, Mysql, Ctx> for super::home_worlds::table
where
    L: LoadingHandler<Mysql, Ctx, Table = super::home_worlds::table> + 'static,
    L::FieldList: WundergraphFieldList<Mysql, L::PrimaryKeyIndex, super::home_worlds::table, Ctx>,
    <L::Filter as BuildFilter<Mysql>>::Ret: AppearsOnTable<super::home_worlds::table>,
    L::Columns: BuildOrder<super::home_worlds::table, Mysql>
        + BuildSelect<
            super::home_worlds::table,
            Mysql,
            SqlTypeOfPlaceholder<
                L::FieldList,
                Mysql,
                L::PrimaryKeyIndex,
                super::home_worlds::table,
                Ctx,
            >,
        >,
    &'static L: Identifiable,
    Ctx: WundergraphContext + QueryModifier<L, Mysql>,
    Ctx::Connection: Connection<Backend = Mysql>,
    <Ctx::Connection as Connection>::Backend: HasSqlType<SqlTypeOf<super::home_worlds::id>>
        + HasSqlType<
            SqlTypeOfPlaceholder<
                L::FieldList,
                Mysql,
                L::PrimaryKeyIndex,
                super::home_worlds::table,
                Ctx,
            >,
        >,
{
    fn handle_batch_insert(
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
        batch: Vec<NewHomeWorld>,
    ) -> ExecutionResult<WundergraphScalarValue> {
        let r = batch
            .into_iter()
            .map(|i| handle_i32_pk_insert(selection, executor, i))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Value::List(r))
    }
}

impl<L, Ctx> HandleInsert<L, NewSpecies, Mysql, Ctx> for super::species::table
where
    L: LoadingHandler<Mysql, Ctx, Table = super::species::table> + 'static,
    L::FieldList: WundergraphFieldList<Mysql, L::PrimaryKeyIndex, super::species::table, Ctx>,
    <L::Filter as BuildFilter<Mysql>>::Ret: AppearsOnTable<super::species::table>,
    L::Columns: BuildOrder<super::species::table, Mysql>
        + BuildSelect<
            super::species::table,
            Mysql,
            SqlTypeOfPlaceholder<
                L::FieldList,
                Mysql,
                L::PrimaryKeyIndex,
                super::species::table,
                Ctx,
            >,
        >,
    &'static L: Identifiable,
    Ctx: WundergraphContext + QueryModifier<L, Mysql>,
    Ctx::Connection: Connection<Backend = Mysql>,
    <Ctx::Connection as Connection>::Backend: HasSqlType<SqlTypeOf<super::species::id>>
        + HasSqlType<
            SqlTypeOfPlaceholder<
                L::FieldList,
                Mysql,
                L::PrimaryKeyIndex,
                super::species::table,
                Ctx,
            >,
        >,
{
    fn handle_insert(
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
        insertable: NewSpecies,
    ) -> ExecutionResult<WundergraphScalarValue> {
        handle_i32_pk_insert(selection, executor, insertable)
    }
}

impl<L, Ctx> HandleBatchInsert<L, NewSpecies, Mysql, Ctx> for super::species::table
where
    L: LoadingHandler<Mysql, Ctx, Table = super::species::table> + 'static,
    L::FieldList: WundergraphFieldList<Mysql, L::PrimaryKeyIndex, super::species::table, Ctx>,
    <L::Filter as BuildFilter<Mysql>>::Ret: AppearsOnTable<super::species::table>,
    L::Columns: BuildOrder<super::species::table, Mysql>
        + BuildSelect<
            super::species::table,
            Mysql,
            SqlTypeOfPlaceholder<
                L::FieldList,
                Mysql,
                L::PrimaryKeyIndex,
                super::species::table,
                Ctx,
            >,
        >,
    &'static L: Identifiable,
    Ctx: WundergraphContext + QueryModifier<L, Mysql>,
    Ctx::Connection: Connection<Backend = Mysql>,
    <Ctx::Connection as Connection>::Backend: HasSqlType<SqlTypeOf<super::species::id>>
        + HasSqlType<
            SqlTypeOfPlaceholder<
                L::FieldList,
                Mysql,
                L::PrimaryKeyIndex,
                super::species::table,
                Ctx,
            >,
        >,
{
    fn handle_batch_insert(
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
        batch: Vec<NewSpecies>,
    ) -> ExecutionResult<WundergraphScalarValue> {
        let r = batch
            .into_iter()
            .map(|i| handle_i32_pk_insert(selection, executor, i))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Value::List(r))
    }
}

impl<L, Ctx> HandleInsert<L, NewHero, Mysql, Ctx> for super::heros::table
where
    L: LoadingHandler<Mysql, Ctx, Table = super::heros::table> + 'static,
    L::FieldList: WundergraphFieldList<Mysql, L::PrimaryKeyIndex, super::heros::table, Ctx>,
    <L::Filter as BuildFilter<Mysql>>::Ret: AppearsOnTable<super::heros::table>,
    L::Columns: BuildOrder<super::heros::table, Mysql>
        + BuildSelect<
            super::heros::table,
            Mysql,
            SqlTypeOfPlaceholder<L::FieldList, Mysql, L::PrimaryKeyIndex, super::heros::table, Ctx>,
        >,
    &'static L: Identifiable,
    Ctx: WundergraphContext + QueryModifier<L, Mysql>,
    Ctx::Connection: Connection<Backend = Mysql>,
    <Ctx::Connection as Connection>::Backend: HasSqlType<SqlTypeOf<super::heros::id>>
        + HasSqlType<
            SqlTypeOfPlaceholder<L::FieldList, Mysql, L::PrimaryKeyIndex, super::heros::table, Ctx>,
        >,
{
    fn handle_insert(
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
        insertable: NewHero,
    ) -> ExecutionResult<WundergraphScalarValue> {
        handle_i32_pk_insert(selection, executor, insertable)
    }
}

impl<L, Ctx> HandleBatchInsert<L, NewHero, Mysql, Ctx> for super::heros::table
where
    L: LoadingHandler<Mysql, Ctx, Table = super::heros::table> + 'static,
    L::FieldList: WundergraphFieldList<Mysql, L::PrimaryKeyIndex, super::heros::table, Ctx>,
    <L::Filter as BuildFilter<Mysql>>::Ret: AppearsOnTable<super::heros::table>,
    L::Columns: BuildOrder<super::heros::table, Mysql>
        + BuildSelect<
            super::heros::table,
            Mysql,
            SqlTypeOfPlaceholder<L::FieldList, Mysql, L::PrimaryKeyIndex, super::heros::table, Ctx>,
        >,
    &'static L: Identifiable,
    Ctx: WundergraphContext + QueryModifier<L, Mysql>,
    Ctx::Connection: Connection<Backend = Mysql>,
    <Ctx::Connection as Connection>::Backend: HasSqlType<SqlTypeOf<super::heros::id>>
        + HasSqlType<
            SqlTypeOfPlaceholder<L::FieldList, Mysql, L::PrimaryKeyIndex, super::heros::table, Ctx>,
        >,
{
    fn handle_batch_insert(
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
        batch: Vec<NewHero>,
    ) -> ExecutionResult<WundergraphScalarValue> {
        let r = batch
            .into_iter()
            .map(|i| handle_i32_pk_insert(selection, executor, i))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Value::List(r))
    }
}

fn handle_single_friend_insert<L, Ctx>(
    selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
    executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    insertable: NewFriend,
) -> ExecutionResult<WundergraphScalarValue>
where
    L: LoadingHandler<Mysql, Ctx, Table = super::friends::table> + 'static,
    L::FieldList: WundergraphFieldList<Mysql, L::PrimaryKeyIndex, super::friends::table, Ctx>,
    <L::Filter as BuildFilter<Mysql>>::Ret: AppearsOnTable<super::friends::table>,
    L::Columns: BuildOrder<super::friends::table, Mysql>
        + BuildSelect<
            super::friends::table,
            Mysql,
            SqlTypeOfPlaceholder<
                L::FieldList,
                Mysql,
                L::PrimaryKeyIndex,
                super::friends::table,
                Ctx,
            >,
        >,
    &'static L: Identifiable,
    Ctx: WundergraphContext + QueryModifier<L, Mysql>,
    Ctx::Connection: Connection<Backend = Mysql>,
    <Ctx::Connection as Connection>::Backend: HasSqlType<SqlTypeOf<(super::friends::hero_id, super::friends::friend_id)>>
        + HasSqlType<
            SqlTypeOfPlaceholder<
                L::FieldList,
                Mysql,
                L::PrimaryKeyIndex,
                super::friends::table,
                Ctx,
            >,
        >,
{
    let ctx = executor.context();
    let conn = ctx.get_connection();
    let look_ahead = executor.look_ahead();
    insertable
        .insert_into(super::friends::table)
        .execute(conn)
        .unwrap();
    let q = L::build_query(&[], &look_ahead)?;
    let q = FilterDsl::filter(q, super::friends::friend_id.eq_all(insertable.friend_id));
    let q = FilterDsl::filter(q, super::friends::hero_id.eq_all(insertable.hero_id));
    let items = L::load(&look_ahead, selection, executor, q)?;
    Ok(items.into_iter().next().unwrap_or(Value::Null))
}

impl<L, Ctx> HandleInsert<L, NewFriend, Mysql, Ctx> for super::friends::table
where
    L: LoadingHandler<Mysql, Ctx, Table = super::friends::table> + 'static,
    L::FieldList: WundergraphFieldList<Mysql, L::PrimaryKeyIndex, super::friends::table, Ctx>,
    <L::Filter as BuildFilter<Mysql>>::Ret: AppearsOnTable<super::friends::table>,
    L::Columns: BuildOrder<super::friends::table, Mysql>
        + BuildSelect<
            super::friends::table,
            Mysql,
            SqlTypeOfPlaceholder<
                L::FieldList,
                Mysql,
                L::PrimaryKeyIndex,
                super::friends::table,
                Ctx,
            >,
        >,
    &'static L: Identifiable,
    Ctx: WundergraphContext + QueryModifier<L, Mysql>,
    Ctx::Connection: Connection<Backend = Mysql>,
    <Ctx::Connection as Connection>::Backend: HasSqlType<SqlTypeOf<(super::friends::hero_id, super::friends::friend_id)>>
        + HasSqlType<
            SqlTypeOfPlaceholder<
                L::FieldList,
                Mysql,
                L::PrimaryKeyIndex,
                super::friends::table,
                Ctx,
            >,
        >,
{
    fn handle_insert(
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
        insertable: NewFriend,
    ) -> ExecutionResult<WundergraphScalarValue> {
        handle_single_friend_insert(selection, executor, insertable)
    }
}

impl<L, Ctx> HandleBatchInsert<L, NewFriend, Mysql, Ctx> for super::friends::table
where
    L: LoadingHandler<Mysql, Ctx, Table = super::friends::table> + 'static,
    L::FieldList: WundergraphFieldList<Mysql, L::PrimaryKeyIndex, super::friends::table, Ctx>,
    <L::Filter as BuildFilter<Mysql>>::Ret: AppearsOnTable<super::friends::table>,
    L::Columns: BuildOrder<super::friends::table, Mysql>
        + BuildSelect<
            super::friends::table,
            Mysql,
            SqlTypeOfPlaceholder<
                L::FieldList,
                Mysql,
                L::PrimaryKeyIndex,
                super::friends::table,
                Ctx,
            >,
        >,
    &'static L: Identifiable,
    Ctx: WundergraphContext + QueryModifier<L, Mysql>,
    Ctx::Connection: Connection<Backend = Mysql>,
    <Ctx::Connection as Connection>::Backend: HasSqlType<SqlTypeOf<(super::friends::hero_id, super::friends::friend_id)>>
        + HasSqlType<
            SqlTypeOfPlaceholder<
                L::FieldList,
                Mysql,
                L::PrimaryKeyIndex,
                super::friends::table,
                Ctx,
            >,
        >,
{
    fn handle_batch_insert(
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
        batch: Vec<NewFriend>,
    ) -> ExecutionResult<WundergraphScalarValue> {
        let r = batch
            .into_iter()
            .map(|i| handle_single_friend_insert(selection, executor, i))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Value::List(r))
    }
}

fn handle_single_appears_in_insert<L, Ctx>(
    selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
    executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    insertable: NewAppearsIn,
) -> ExecutionResult<WundergraphScalarValue>
where
    L: LoadingHandler<Mysql, Ctx, Table = super::appears_in::table> + 'static,
    L::FieldList: WundergraphFieldList<Mysql, L::PrimaryKeyIndex, super::appears_in::table, Ctx>,
    <L::Filter as BuildFilter<Mysql>>::Ret: AppearsOnTable<super::appears_in::table>,
    L::Columns: BuildOrder<super::appears_in::table, Mysql>
        + BuildSelect<
            super::appears_in::table,
            Mysql,
            SqlTypeOfPlaceholder<
                L::FieldList,
                Mysql,
                L::PrimaryKeyIndex,
                super::appears_in::table,
                Ctx,
            >,
        >,
    &'static L: Identifiable,
    Ctx: WundergraphContext + QueryModifier<L, Mysql>,
    Ctx::Connection: Connection<Backend = Mysql>,
    <Ctx::Connection as Connection>::Backend: HasSqlType<SqlTypeOf<(super::appears_in::hero_id, super::appears_in::episode)>>
        + HasSqlType<
            SqlTypeOfPlaceholder<
                L::FieldList,
                Mysql,
                L::PrimaryKeyIndex,
                super::appears_in::table,
                Ctx,
            >,
        >,
{
    let ctx = executor.context();
    let conn = ctx.get_connection();
    let look_ahead = executor.look_ahead();
    insertable
        .insert_into(super::appears_in::table)
        .execute(conn)
        .unwrap();
    let q = L::build_query(&[], &look_ahead)?;
    let q = FilterDsl::filter(q, super::appears_in::episode.eq_all(insertable.episode));
    let q = FilterDsl::filter(q, super::appears_in::hero_id.eq_all(insertable.hero_id));
    let items = L::load(&look_ahead, selection, executor, q)?;
    Ok(items.into_iter().next().unwrap_or(Value::Null))
}

impl<L, Ctx> HandleInsert<L, NewAppearsIn, Mysql, Ctx> for super::appears_in::table
where
    L: LoadingHandler<Mysql, Ctx, Table = super::appears_in::table> + 'static,
    L::FieldList: WundergraphFieldList<Mysql, L::PrimaryKeyIndex, super::appears_in::table, Ctx>,
    <L::Filter as BuildFilter<Mysql>>::Ret: AppearsOnTable<super::appears_in::table>,
    L::Columns: BuildOrder<super::appears_in::table, Mysql>
        + BuildSelect<
            super::appears_in::table,
            Mysql,
            SqlTypeOfPlaceholder<
                L::FieldList,
                Mysql,
                L::PrimaryKeyIndex,
                super::appears_in::table,
                Ctx,
            >,
        >,
    &'static L: Identifiable,
    Ctx: WundergraphContext + QueryModifier<L, Mysql>,
    Ctx::Connection: Connection<Backend = Mysql>,
    <Ctx::Connection as Connection>::Backend: HasSqlType<SqlTypeOf<(super::appears_in::hero_id, super::appears_in::episode)>>
        + HasSqlType<
            SqlTypeOfPlaceholder<
                L::FieldList,
                Mysql,
                L::PrimaryKeyIndex,
                super::appears_in::table,
                Ctx,
            >,
        >,
{
    fn handle_insert(
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
        insertable: NewAppearsIn,
    ) -> ExecutionResult<WundergraphScalarValue> {
        handle_single_appears_in_insert(selection, executor, insertable)
    }
}

impl<L, Ctx> HandleBatchInsert<L, NewAppearsIn, Mysql, Ctx> for super::appears_in::table
where
    L: LoadingHandler<Mysql, Ctx, Table = super::appears_in::table> + 'static,
    L::FieldList: WundergraphFieldList<Mysql, L::PrimaryKeyIndex, super::appears_in::table, Ctx>,
    <L::Filter as BuildFilter<Mysql>>::Ret: AppearsOnTable<super::appears_in::table>,
    L::Columns: BuildOrder<super::appears_in::table, Mysql>
        + BuildSelect<
            super::appears_in::table,
            Mysql,
            SqlTypeOfPlaceholder<
                L::FieldList,
                Mysql,
                L::PrimaryKeyIndex,
                super::appears_in::table,
                Ctx,
            >,
        >,
    &'static L: Identifiable,
    Ctx: WundergraphContext + QueryModifier<L, Mysql>,
    Ctx::Connection: Connection<Backend = Mysql>,
    <Ctx::Connection as Connection>::Backend: HasSqlType<SqlTypeOf<(super::appears_in::hero_id, super::appears_in::episode)>>
        + HasSqlType<
            SqlTypeOfPlaceholder<
                L::FieldList,
                Mysql,
                L::PrimaryKeyIndex,
                super::appears_in::table,
                Ctx,
            >,
        >,
{
    fn handle_batch_insert(
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
        batch: Vec<NewAppearsIn>,
    ) -> ExecutionResult<WundergraphScalarValue> {
        let r = batch
            .into_iter()
            .map(|i| handle_single_appears_in_insert(selection, executor, i))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Value::List(r))
    }
}
