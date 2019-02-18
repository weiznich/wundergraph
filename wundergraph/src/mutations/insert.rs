use diesel::associations::HasTable;
use diesel::backend::Backend;
use diesel::dsl::SqlTypeOf;
use diesel::query_builder::BoxedSelectStatement;
use diesel::query_builder::QueryFragment;
use diesel::query_dsl::methods::BoxedDsl;
use diesel::sql_types::{Bool, HasSqlType};
use diesel::QuerySource;
use diesel::{AppearsOnTable, Connection, Insertable, Queryable, RunQueryDsl, Table};

use filter::build_filter::BuildFilter;
use query_helper::order::BuildOrder;
use query_helper::placeholder::{SqlTypeOfPlaceholder, WundergraphFieldList};
use query_helper::select::BuildSelect;
use query_helper::tuple::TupleIndex;

#[cfg(feature = "postgres")]
use diesel::expression::{AsExpression, Expression, NonAggregate, SelectableExpression};
#[cfg(feature = "postgres")]
use diesel::insertable::CanInsertInSingleQuery;
#[cfg(feature = "postgres")]
use diesel::pg::Pg;
#[cfg(feature = "postgres")]
use diesel::query_builder::UndecoratedInsertRecord;
#[cfg(feature = "postgres")]
use diesel::query_dsl::methods::FilterDsl;
#[cfg(feature = "postgres")]
use diesel::{EqAll, ExpressionMethods, Identifiable};
#[cfg(feature = "postgres")]
use helper::primary_keys::UnRef;

#[cfg(feature = "sqlite")]
use diesel::expression::dsl::sql;
#[cfg(feature = "sqlite")]
use diesel::query_builder::InsertStatement;
#[cfg(feature = "sqlite")]
use diesel::query_dsl::methods::{ExecuteDsl, LimitDsl, OrderDsl};
#[cfg(feature = "sqlite")]
use diesel::sqlite::Sqlite;

use juniper::{
    Arguments, ExecutionResult, Executor, FieldError, FromInputValue, GraphQLType, Value,
};

use scalar::WundergraphScalarValue;
use LoadingHandler;
use WundergraphContext;

pub fn handle_batch_insert<DB, I, R, Ctx>(
    executor: &Executor<Ctx, WundergraphScalarValue>,
    arguments: &Arguments<WundergraphScalarValue>,
    field_name: &'static str,
) -> ExecutionResult<WundergraphScalarValue>
where
    DB: Backend,
    Ctx: WundergraphContext<DB>,
    I: InsertHelper<DB, R, Ctx> + FromInputValue<WundergraphScalarValue>,
    I::Handler: HandleInsert<DB, R, Ctx, Insert = I>,
{
    if let Some(n) = arguments.get::<Vec<I>>(field_name) {
        I::Handler::handle_batch_insert(executor, n)
    } else {
        let msg = format!("Missing argument {}", field_name);
        Err(FieldError::new(&msg, Value::Null))
    }
}

pub fn handle_insert<DB, I, R, Ctx>(
    executor: &Executor<Ctx, WundergraphScalarValue>,
    arguments: &Arguments<WundergraphScalarValue>,
    field_name: &'static str,
) -> ExecutionResult<WundergraphScalarValue>
where
    DB: Backend,
    Ctx: WundergraphContext<DB>,
    I: InsertHelper<DB, R, Ctx> + FromInputValue<WundergraphScalarValue>,
    I::Handler: HandleInsert<DB, R, Ctx, Insert = I>,
{
    if let Some(n) = arguments.get::<I>(field_name) {
        I::Handler::handle_insert(executor, n)
    } else {
        let msg = format!("Missing argument {}", field_name);
        Err(FieldError::new(&msg, Value::Null))
    }
}

pub trait InsertHelper<DB, R, Ctx> {
    type Handler: HandleInsert<DB, R, Ctx>;
}

pub trait HandleInsert<DB, R, Ctx>: Sized {
    type Insert;
    fn handle_insert(
        executor: &Executor<Ctx, WundergraphScalarValue>,
        insertable: Self::Insert,
    ) -> ExecutionResult<WundergraphScalarValue>;
    fn handle_batch_insert(
        executor: &Executor<Ctx, WundergraphScalarValue>,
        insertable: Vec<Self::Insert>,
    ) -> ExecutionResult<WundergraphScalarValue>;
}

#[doc(hidden)]
#[derive(Debug)]
pub struct InsertableWrapper<I>(I);

#[cfg_attr(feature = "cargo-clippy", allow(use_self))]
impl<I, DB, R, Ctx> InsertHelper<DB, R, Ctx> for I
where
    I: Insertable<R::Table>,
    R: LoadingHandler<DB>,
    DB: Backend + 'static,
    InsertableWrapper<I>: HandleInsert<DB, R, Ctx>,
    <R::Table as QuerySource>::FromClause: QueryFragment<DB>,
    R::Table: 'static,
    DB::QueryBuilder: Default,
{
    type Handler = InsertableWrapper<I>;
}

#[cfg(feature = "postgres")]
impl<I, T, R, Ctx, Id> HandleInsert<Pg, R, Ctx> for InsertableWrapper<I>
where
    I: Insertable<T> + UndecoratedInsertRecord<T>,
    T: Table + HasTable<Table = T> + 'static,
    Ctx: WundergraphContext<Pg>,
    R: LoadingHandler<Pg, Table = T>
        + GraphQLType<WundergraphScalarValue, TypeInfo = (), Context = ()>
        + 'static,
    Pg: HasSqlType<<T::PrimaryKey as Expression>::SqlType>,
    Id: Queryable<<T::PrimaryKey as Expression>::SqlType, Pg>,
    T::FromClause: QueryFragment<Pg>,
    T::PrimaryKey: QueryFragment<Pg>,
    I::Values: QueryFragment<Pg> + CanInsertInSingleQuery<Pg>,
    <Vec<I> as Insertable<T>>::Values: QueryFragment<Pg> + CanInsertInSingleQuery<Pg>,
    &'static R: Identifiable,
    <&'static R as Identifiable>::Id: UnRef<'static, UnRefed = Id>,
    T::PrimaryKey: EqAll<Id> + ExpressionMethods,
    <T::PrimaryKey as EqAll<Id>>::Output:
        SelectableExpression<T> + NonAggregate + QueryFragment<Pg> + 'static,
    R::Columns: BuildOrder<T, Pg>
        + BuildSelect<
            T,
            Pg,
            SqlTypeOfPlaceholder<R::FieldList, Pg, R::PrimaryKeyIndex, R::Table>,
        >,
    R::FieldList: WundergraphFieldList<Pg, R::PrimaryKeyIndex, T>
        + TupleIndex<R::PrimaryKeyIndex>,
    <R::FieldList as WundergraphFieldList<Pg, R::PrimaryKeyIndex, T>>::PlaceHolder:
        Queryable<SqlTypeOfPlaceholder<R::FieldList, Pg, R::PrimaryKeyIndex, R::Table>, Pg>,
    for<'a> R::Table: BoxedDsl<
        'a,
        Pg,
        Output = BoxedSelectStatement<'a, SqlTypeOf<<R::Table as Table>::AllColumns>, R::Table, Pg>,
    >,
    <R::Filter as BuildFilter<Pg>>::Ret: AppearsOnTable<T>,
    Pg: HasSqlType<SqlTypeOfPlaceholder<R::FieldList, Pg, R::PrimaryKeyIndex, R::Table>>,
    Id: AsExpression<SqlTypeOf<T::PrimaryKey>>,
    <Id as AsExpression<SqlTypeOf<T::PrimaryKey>>>::Expression:
        AppearsOnTable<T> + QueryFragment<Pg>,
{
    type Insert = I;

    fn handle_insert(
        executor: &Executor<Ctx, WundergraphScalarValue>,
        insertable: Self::Insert,
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
            let q = R::build_query(&look_ahead)?;
            let q = FilterDsl::filter(q, T::table().primary_key().eq_all(inserted));
            let items = R::load(&look_ahead, conn, q)?;
            Ok(items.into_iter().next().unwrap_or(Value::Null))
        })
    }

    fn handle_batch_insert(
        executor: &Executor<Ctx, WundergraphScalarValue>,
        batch: Vec<Self::Insert>,
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
            let q = FilterDsl::filter(
                R::build_query(&look_ahead)?,
                T::table().primary_key().eq_any(inserted),
            );
            let items = R::load(&look_ahead, conn, q)?;
            Ok(Value::list(items))
        })
    }
}

#[cfg(feature = "sqlite")]
impl<I, T, R, Ctx> HandleInsert<Sqlite, R, Ctx> for InsertableWrapper<I>
where
    I: Insertable<T>,
    Vec<I>: Insertable<T>,
    T: Table + HasTable<Table = T> + 'static,
    Ctx: WundergraphContext<Sqlite>,
    R: LoadingHandler<Sqlite, Table = T>
        + GraphQLType<WundergraphScalarValue, TypeInfo = (), Context = ()>,
    T::FromClause: QueryFragment<Sqlite>,
    InsertStatement<T, I::Values>: ExecuteDsl<Ctx::Connection>,
    R::Columns: BuildOrder<T, Sqlite>
        + BuildSelect<
            T,
            Sqlite,
            SqlTypeOfPlaceholder<R::FieldList, Sqlite, R::PrimaryKeyIndex, R::Table>,
        >,
    R::FieldList: WundergraphFieldList<Sqlite, R::PrimaryKeyIndex, T>
        + TupleIndex<R::PrimaryKeyIndex>,
    <R::FieldList as WundergraphFieldList<Sqlite, R::PrimaryKeyIndex, T>>::PlaceHolder:
        Queryable<SqlTypeOfPlaceholder<R::FieldList, Sqlite, R::PrimaryKeyIndex, R::Table>, Sqlite>,
    for<'a> R::Table: BoxedDsl<
        'a,
        Sqlite,
        Output = BoxedSelectStatement<
            'a,
            SqlTypeOf<<R::Table as Table>::AllColumns>,
            R::Table,
            Sqlite,
        >,
    >,
    <R::Filter as BuildFilter<Sqlite>>::Ret: AppearsOnTable<T>,
    Sqlite: HasSqlType<SqlTypeOfPlaceholder<R::FieldList, Sqlite, R::PrimaryKeyIndex, R::Table>>,
{
    type Insert = I;

    fn handle_insert(
        executor: &Executor<Ctx, WundergraphScalarValue>,
        insertable: Self::Insert,
    ) -> ExecutionResult<WundergraphScalarValue> {
        let ctx = executor.context();
        let conn = ctx.get_connection();
        conn.transaction(|| -> ExecutionResult<WundergraphScalarValue> {
            let look_ahead = executor.look_ahead();
            insertable.insert_into(T::table()).execute(conn)?;
            let q = OrderDsl::order(R::build_query(&look_ahead)?, sql::<Bool>("rowid DESC"));
            let q = LimitDsl::limit(q, 1);
            let items = R::load(&look_ahead, conn, q)?;

            Ok(items.into_iter().next().unwrap_or(Value::Null))
        })
    }

    fn handle_batch_insert(
        executor: &Executor<Ctx, WundergraphScalarValue>,
        batch: Vec<Self::Insert>,
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
            let q = OrderDsl::order(R::build_query(&look_ahead)?, sql::<Bool>("rowid DESC"));
            let q = LimitDsl::limit(q, n as i64);
            let items = R::load(&look_ahead, conn, q)?;
            Ok(Value::list(items))
        })
    }
}
