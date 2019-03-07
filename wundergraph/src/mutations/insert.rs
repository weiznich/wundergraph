use diesel::associations::HasTable;
use diesel::backend::Backend;
use diesel::dsl::SqlTypeOf;
use diesel::query_builder::BoxedSelectStatement;
use diesel::query_builder::QueryFragment;
use diesel::query_dsl::methods::BoxedDsl;
use diesel::sql_types::HasSqlType;
use diesel::QuerySource;
use diesel::{AppearsOnTable, Connection, Insertable, Queryable, RunQueryDsl, Table};

use filter::build_filter::BuildFilter;
use query_helper::order::BuildOrder;
use query_helper::placeholder::{SqlTypeOfPlaceholder, WundergraphFieldList};
use query_helper::select::BuildSelect;

#[cfg(feature = "postgres")]
use diesel::expression::{Expression, NonAggregate, SelectableExpression};
#[cfg(feature = "postgres")]
use diesel::insertable::CanInsertInSingleQuery;
#[cfg(feature = "postgres")]
use diesel::pg::Pg;
#[cfg(feature = "postgres")]
use diesel::query_dsl::methods::FilterDsl;
#[cfg(feature = "postgres")]
use diesel::{EqAll, Identifiable};
#[cfg(feature = "postgres")]
use helper::primary_keys::UnRef;

#[cfg(feature = "sqlite")]
use diesel::expression::dsl::sql;
#[cfg(feature = "sqlite")]
use diesel::query_builder::InsertStatement;
#[cfg(feature = "sqlite")]
use diesel::query_dsl::methods::{ExecuteDsl, LimitDsl, OrderDsl};
#[cfg(feature = "sqlite")]
use diesel::sql_types::Bool;
#[cfg(feature = "sqlite")]
use diesel::sqlite::Sqlite;

use juniper::{Arguments, ExecutionResult, Executor, FieldError, FromInputValue, Value};

use scalar::WundergraphScalarValue;
use LoadingHandler;
use WundergraphContext;

pub fn handle_insert<DB, I, R, Ctx>(
    executor: &Executor<Ctx, WundergraphScalarValue>,
    arguments: &Arguments<WundergraphScalarValue>,
    field_name: &'static str,
) -> ExecutionResult<WundergraphScalarValue>
where
    R: LoadingHandler<DB>,
    R::Table: HandleInsert<R, I, DB, Ctx> + 'static,
    DB: Backend + 'static,
    DB::QueryBuilder: Default,
    R::Columns: BuildOrder<R::Table, DB>
        + BuildSelect<
            R::Table,
            DB,
            SqlTypeOfPlaceholder<R::FieldList, DB, R::PrimaryKeyIndex, R::Table>,
        >,
    <R::Table as QuerySource>::FromClause: QueryFragment<DB>,
    I: FromInputValue<WundergraphScalarValue>,
{
    if let Some(n) = arguments.get::<I>(field_name) {
        <R::Table as HandleInsert<_, _, _, _>>::handle_insert(executor, n)
    } else {
        let msg = format!("Missing argument {}", field_name);
        Err(FieldError::new(&msg, Value::Null))
    }
}

pub fn handle_batch_insert<DB, I, R, Ctx>(
    executor: &Executor<Ctx, WundergraphScalarValue>,
    arguments: &Arguments<WundergraphScalarValue>,
    field_name: &'static str,
) -> ExecutionResult<WundergraphScalarValue>
where
    R: LoadingHandler<DB>,
    R::Table: HandleBatchInsert<R, I, DB, Ctx> + 'static,
    DB: Backend + 'static,
    DB::QueryBuilder: Default,
    R::Columns: BuildOrder<R::Table, DB>
        + BuildSelect<
            R::Table,
            DB,
            SqlTypeOfPlaceholder<R::FieldList, DB, R::PrimaryKeyIndex, R::Table>,
        >,
    <R::Table as QuerySource>::FromClause: QueryFragment<DB>,
    I: FromInputValue<WundergraphScalarValue>,
{
    if let Some(n) = arguments.get::<Vec<I>>(field_name) {
        <R::Table as HandleBatchInsert<_, _, _, _>>::handle_batch_insert(executor, n)
    } else {
        let msg = format!("Missing argument {}", field_name);
        Err(FieldError::new(&msg, Value::Null))
    }
}

pub trait HandleInsert<L, I, DB, Ctx> {
    fn handle_insert(
        executor: &Executor<Ctx, WundergraphScalarValue>,
        insertable: I,
    ) -> ExecutionResult<WundergraphScalarValue>;
}

pub trait HandleBatchInsert<L, I, DB, Ctx> {
    fn handle_batch_insert(
        executor: &Executor<Ctx, WundergraphScalarValue>,
        insertable: Vec<I>,
    ) -> ExecutionResult<WundergraphScalarValue>;
}

#[cfg(feature = "postgres")]
impl<I, Ctx, L, T, Id> HandleInsert<L, I, Pg, Ctx> for T
where
    T: Table + HasTable<Table = T> + 'static,
    T::FromClause: QueryFragment<Pg>,
    L: LoadingHandler<Pg, Table = T> + 'static,
    L::Columns: BuildOrder<T, Pg>
        + BuildSelect<T, Pg, SqlTypeOfPlaceholder<L::FieldList, Pg, L::PrimaryKeyIndex, T>>,
    Ctx: WundergraphContext<Pg>,
    L::FieldList: WundergraphFieldList<Pg, L::PrimaryKeyIndex, T>,
    I: Insertable<T>,
    I::Values: QueryFragment<Pg> + CanInsertInSingleQuery<Pg>,
    T::PrimaryKey: QueryFragment<Pg>,
    T: BoxedDsl<
        'static,
        Pg,
        Output = BoxedSelectStatement<'static, SqlTypeOf<<T as Table>::AllColumns>, T, Pg>,
    >,
    Pg: HasSqlType<SqlTypeOf<T::PrimaryKey>>
        + HasSqlType<SqlTypeOfPlaceholder<L::FieldList, Pg, L::PrimaryKeyIndex, T>>,
    <L::Filter as BuildFilter<Pg>>::Ret: AppearsOnTable<T>,
    T::PrimaryKey: EqAll<Id>,
    &'static L: Identifiable,
    <&'static L as Identifiable>::Id: UnRef<'static, UnRefed = Id>,
    Id: Queryable<<T::PrimaryKey as Expression>::SqlType, Pg>,
    <T::PrimaryKey as EqAll<Id>>::Output:
        SelectableExpression<T> + NonAggregate + QueryFragment<Pg> + 'static,
{
    fn handle_insert(
        executor: &Executor<Ctx, WundergraphScalarValue>,
        insertable: I,
    ) -> ExecutionResult<WundergraphScalarValue> {
        let ctx = executor.context();
        let conn = ctx.get_connection();
        conn.transaction(|| -> ExecutionResult<WundergraphScalarValue> {
            let look_ahead = executor.look_ahead();
            let inserted = insertable
                .insert_into(T::table())
                .returning(T::table().primary_key());
            if cfg!(feature = "debug") {
                debug!("{}", ::diesel::debug_query(&inserted));
            }
            let inserted: Id = inserted.get_result(conn)?;
            let q = L::build_query(&look_ahead)?;
            let q = FilterDsl::filter(q, T::table().primary_key().eq_all(inserted));
            let items = L::load(&look_ahead, conn, q)?;
            Ok(items.into_iter().next().unwrap_or(Value::Null))
        })
    }
}

#[cfg(feature = "postgres")]
impl<I, Ctx, L, T, Id> HandleBatchInsert<L, I, Pg, Ctx> for T
where
    T: Table + HasTable<Table = T> + 'static,
    T::FromClause: QueryFragment<Pg>,
    L: LoadingHandler<Pg, Table = T> + 'static,
    L::Columns: BuildOrder<T, Pg>
        + BuildSelect<T, Pg, SqlTypeOfPlaceholder<L::FieldList, Pg, L::PrimaryKeyIndex, T>>,
    Ctx: WundergraphContext<Pg>,
    L::FieldList: WundergraphFieldList<Pg, L::PrimaryKeyIndex, T>,
    Vec<I>: Insertable<T>,
    <Vec<I> as Insertable<T>>::Values: QueryFragment<Pg> + CanInsertInSingleQuery<Pg>,
    T::PrimaryKey: QueryFragment<Pg>,
    T: BoxedDsl<
        'static,
        Pg,
        Output = BoxedSelectStatement<'static, SqlTypeOf<<T as Table>::AllColumns>, T, Pg>,
    >,
    Pg: HasSqlType<SqlTypeOf<T::PrimaryKey>>
        + HasSqlType<SqlTypeOfPlaceholder<L::FieldList, Pg, L::PrimaryKeyIndex, T>>,
    <L::Filter as BuildFilter<Pg>>::Ret: AppearsOnTable<T>,
    T::PrimaryKey: EqAll<Id>,
    &'static L: Identifiable,
    <&'static L as Identifiable>::Id: UnRef<'static, UnRefed = Id>,
    Id: Queryable<<T::PrimaryKey as Expression>::SqlType, Pg>,
    // <Id as AsExpression<SqlTypeOf<T::PrimaryKey>>>::Expression:
    //     AppearsOnTable<T> + QueryFragment<Pg>,
    <T::PrimaryKey as EqAll<Id>>::Output:
        SelectableExpression<T> + NonAggregate + QueryFragment<Pg> + 'static,
{
    fn handle_batch_insert(
        executor: &Executor<Ctx, WundergraphScalarValue>,
        batch: Vec<I>,
    ) -> ExecutionResult<WundergraphScalarValue> {
        let ctx = executor.context();
        let conn = ctx.get_connection();
        conn.transaction(|| -> ExecutionResult<WundergraphScalarValue> {
            let look_ahead = executor.look_ahead();
            let inserted = batch
                .insert_into(T::table())
                .returning(T::table().primary_key());
            if cfg!(feature = "debug") {
                debug!("{}", ::diesel::debug_query(&inserted));
            }
            let inserted: Vec<Id> = inserted.get_results(conn)?;
            let mut q = L::build_query(&look_ahead)?;
            for i in inserted {
                q = FilterDsl::filter(q, T::table().primary_key().eq_all(i));
            }
            let items = L::load(&look_ahead, conn, q)?;
            Ok(Value::list(items))
        })
    }
}

#[cfg(feature = "sqlite")]
impl<I, Ctx, L, T> HandleInsert<L, I, Sqlite, Ctx> for T
where
    T: Table + HasTable<Table = T> + 'static,
    T::FromClause: QueryFragment<Sqlite>,
    L: LoadingHandler<Sqlite, Table = T>,
    L::Columns: BuildOrder<T, Sqlite>
        + BuildSelect<
            T,
            Sqlite,
            SqlTypeOfPlaceholder<L::FieldList, Sqlite, L::PrimaryKeyIndex, T>,
        >,
    Ctx: WundergraphContext<Sqlite>,
    L::FieldList: WundergraphFieldList<Sqlite, L::PrimaryKeyIndex, T>,
    I: Insertable<T>,
    I::Values: QueryFragment<Sqlite>,
    InsertStatement<T, I::Values>: ExecuteDsl<Ctx::Connection>,
    T: BoxedDsl<
        'static,
        Sqlite,
        Output = BoxedSelectStatement<'static, SqlTypeOf<<T as Table>::AllColumns>, T, Sqlite>,
    >,
    <L::Filter as BuildFilter<Sqlite>>::Ret: AppearsOnTable<T>,
    Sqlite: HasSqlType<SqlTypeOfPlaceholder<L::FieldList, Sqlite, L::PrimaryKeyIndex, T>>,
{
    fn handle_insert(
        executor: &Executor<Ctx, WundergraphScalarValue>,
        insertable: I,
    ) -> ExecutionResult<WundergraphScalarValue> {
        let ctx = executor.context();
        let conn = ctx.get_connection();
        conn.transaction(|| -> ExecutionResult<WundergraphScalarValue> {
            let look_ahead = executor.look_ahead();
            insertable.insert_into(T::table()).execute(conn)?;
            let q = OrderDsl::order(L::build_query(&look_ahead)?, sql::<Bool>("rowid DESC"));
            let q = LimitDsl::limit(q, 1);
            let items = L::load(&look_ahead, conn, q)?;

            Ok(items.into_iter().next().unwrap_or(Value::Null))
        })
    }
}

#[cfg(feature = "sqlite")]
impl<I, Ctx, L, T> HandleBatchInsert<L, I, Sqlite, Ctx> for T
where
    T: Table + HasTable<Table = T> + 'static,
    T::FromClause: QueryFragment<Sqlite>,
    L: LoadingHandler<Sqlite, Table = T>,
    L::Columns: BuildOrder<T, Sqlite>
        + BuildSelect<
            T,
            Sqlite,
            SqlTypeOfPlaceholder<L::FieldList, Sqlite, L::PrimaryKeyIndex, T>,
        >,
    Ctx: WundergraphContext<Sqlite>,
    L::FieldList: WundergraphFieldList<Sqlite, L::PrimaryKeyIndex, T>,
    I: Insertable<T>,
    I::Values: QueryFragment<Sqlite>,
    InsertStatement<T, I::Values>: ExecuteDsl<Ctx::Connection>,
    T: BoxedDsl<
        'static,
        Sqlite,
        Output = BoxedSelectStatement<'static, SqlTypeOf<<T as Table>::AllColumns>, T, Sqlite>,
    >,
    <L::Filter as BuildFilter<Sqlite>>::Ret: AppearsOnTable<T>,
    Sqlite: HasSqlType<SqlTypeOfPlaceholder<L::FieldList, Sqlite, L::PrimaryKeyIndex, T>>,
{
    fn handle_batch_insert(
        executor: &Executor<Ctx, WundergraphScalarValue>,
        batch: Vec<I>,
    ) -> ExecutionResult<WundergraphScalarValue> {
        let ctx = executor.context();
        let conn = ctx.get_connection();
        conn.transaction(|| -> ExecutionResult<WundergraphScalarValue> {
            let look_ahead = executor.look_ahead();
            let n: usize = batch
                .into_iter()
                .map(|i| i.insert_into(T::table()).execute(conn))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .sum();
            let q = OrderDsl::order(L::build_query(&look_ahead)?, sql::<Bool>("rowid DESC"));
            let q = LimitDsl::limit(q, n as i64);
            let items = L::load(&look_ahead, conn, q)?;
            Ok(Value::list(items))
        })
    }
}
