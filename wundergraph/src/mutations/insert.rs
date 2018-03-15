use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::{Connection, EqAll, Insertable, QueryDsl, Queryable, RunQueryDsl, Table};
use diesel::query_builder::{InsertStatement, QueryFragment, UndecoratedInsertRecord};
use diesel::associations::HasTable;
use diesel::query_dsl::methods::{BoxedDsl, ExecuteDsl, FilterDsl, LimitDsl, OrderDsl};
use diesel::insertable::CanInsertInSingleQuery;
#[cfg(feature = "sqlite")]
use diesel::sqlite::{Sqlite, SqliteConnection};
#[cfg(feature = "postgres")]
use diesel::pg::{Pg, PgConnection};
use diesel::expression::{BoxableExpression, Expression, NonAggregate, SelectableExpression,
                         SqlLiteral};
use diesel::expression::dsl::sql;
use diesel::sql_types::Bool;
use diesel::sql_types::HasSqlType;
use diesel::query_builder::BoxedSelectStatement;
use diesel::dsl::{Filter, Order};

use juniper::{Arguments, ExecutionResult, Executor, FieldError, FromInputValue, GraphQLType, Value};
use LoadingHandler;

pub fn handle_batch_insert<Conn, I, R, T, Id>(
    executor: &Executor<PooledConnection<ConnectionManager<Conn>>>,
    arguments: &Arguments,
    field_name: &'static str,
) -> ExecutionResult
where
    Conn: Connection + 'static,
    Conn::Backend: HandleInsert<Conn, I, T, R, Id>,
    I: FromInputValue,
{
    if let Some(n) = arguments.get::<Vec<I>>(field_name) {
        Conn::Backend::handle_batch_insert(executor, n)
    } else {
        let msg = format!("Missing argument {}", field_name);
        Err(FieldError::new(&msg, Value::Null))
    }
}

pub fn handle_insert<Conn, I, R, T, Id>(
    executor: &Executor<PooledConnection<ConnectionManager<Conn>>>,
    arguments: &Arguments,
    field_name: &'static str,
) -> ExecutionResult
where
    Conn: Connection + 'static,
    Conn::Backend: HandleInsert<Conn, I, T, R, Id>,
    I: FromInputValue,
{
    if let Some(n) = arguments.get::<I>(field_name) {
        Conn::Backend::handle_insert(executor, n)
    } else {
        let msg = format!("Missing argument {}", field_name);
        Err(FieldError::new(&msg, Value::Null))
    }
}

pub trait HandleInsert<Conn, I, T, R, Id>
where
    Conn: Connection + 'static,
{
    fn handle_insert(
        executor: &Executor<PooledConnection<ConnectionManager<Conn>>>,
        insertable: I,
    ) -> ExecutionResult;

    fn handle_batch_insert(
        executor: &Executor<PooledConnection<ConnectionManager<Conn>>>,
        batch: Vec<I>,
    ) -> ExecutionResult;
}

#[cfg(feature = "postgres")]
impl<I, T, R, Id> HandleInsert<PgConnection, I, T, R, Id> for Pg
where
    I: Insertable<T> + UndecoratedInsertRecord<T>,
    Vec<I>: Insertable<T>,
    T: Table + HasTable<Table = T> + 'static,
    T::FromClause: QueryFragment<Pg>,
    T::AllColumns: QueryFragment<Pg>,
    T::PrimaryKey: QueryFragment<Pg>,
    I::Values: QueryFragment<Pg> + CanInsertInSingleQuery<Pg>,
    <Vec<I> as Insertable<T>>::Values: QueryFragment<Pg> + CanInsertInSingleQuery<Pg>,
    Pg: HasSqlType<<T::PrimaryKey as Expression>::SqlType>,
    R: LoadingHandler<PgConnection, Table = T, SqlType = T::SqlType>
        + GraphQLType<TypeInfo = (), Context = ()>,
    Id: Queryable<<T::PrimaryKey as Expression>::SqlType, Pg>,
    T::Query: FilterDsl<<T::PrimaryKey as EqAll<Id>>::Output>,
    T::Query: FilterDsl<Box<BoxableExpression<T, Pg, SqlType = Bool>>>,
    T::PrimaryKey: EqAll<Id>,
    <T::PrimaryKey as EqAll<Id>>::Output: SelectableExpression<T>
        + NonAggregate
        + QueryFragment<Pg>
        + 'static,
    Filter<T::Query, <T::PrimaryKey as EqAll<Id>>::Output>: QueryDsl
        + BoxedDsl<'static, Pg, Output = BoxedSelectStatement<'static, T::SqlType, T, Pg>>,
    Filter<T::Query, Box<BoxableExpression<T, Pg, SqlType = Bool>>>: QueryDsl
        + BoxedDsl<'static, Pg, Output = BoxedSelectStatement<'static, T::SqlType, T, Pg>>,
{
    fn handle_insert(
        executor: &Executor<PooledConnection<ConnectionManager<PgConnection>>>,
        insertable: I,
    ) -> ExecutionResult {
        let conn = executor.context();
        conn.transaction(|| -> ExecutionResult {
            let inserted = insertable
                .insert_into(T::table())
                .returning(T::table().primary_key());
            println!("{}", ::diesel::debug_query(&inserted));
            let inserted: Id = inserted.get_result(conn)?;

            let q = FilterDsl::filter(T::table(), T::table().primary_key().eq_all(inserted));
            let q = q.into_boxed();
            let items = R::load_item(&executor.look_ahead(), conn, q)?;
            executor.resolve_with_ctx(&(), &items.iter().next())
        })
    }

    fn handle_batch_insert(
        executor: &Executor<PooledConnection<ConnectionManager<PgConnection>>>,
        batch: Vec<I>,
    ) -> ExecutionResult {
        use diesel::BoolExpressionMethods;

        let conn = executor.context();
        conn.transaction(|| -> ExecutionResult {
            let inserted = batch
                .insert_into(T::table())
                .returning(T::table().primary_key());
            println!("{}", ::diesel::debug_query(&inserted));
            let inserted: Vec<Id> = inserted.get_results(conn)?;

            let mut ids = inserted.into_iter();
            if let Some(id) = ids.next() {
                let mut f = Box::new(T::table().primary_key().eq_all(id))
                    as Box<BoxableExpression<T, Pg, SqlType = Bool>>;
                for id in ids {
                    f = Box::new(f.or(T::table().primary_key().eq_all(id))) as Box<_>;
                }
                let q = FilterDsl::filter(T::table(), f).into_boxed();
                let items = R::load_item(&executor.look_ahead(), conn, q)?;
                executor.resolve_with_ctx(&(), &items)
            } else {
                Ok(Value::Null)
            }
        })
    }
}

#[cfg(feature = "sqlite")]
impl<I, T, R, Id> HandleInsert<SqliteConnection, I, T, R, Id> for Sqlite
where
    I: Insertable<T>,
    Vec<I>: Insertable<T>,
    T: Table + HasTable<Table = T>,
    T::FromClause: QueryFragment<Sqlite>,
    InsertStatement<T, I::Values>: ExecuteDsl<SqliteConnection>,
    InsertStatement<T, <Vec<I> as Insertable<T>>::Values>: ExecuteDsl<SqliteConnection>,
    T::Query: OrderDsl<SqlLiteral<Bool>>,
    Order<T::Query, SqlLiteral<Bool>>: QueryDsl
        + BoxedDsl<'static, Sqlite, Output = BoxedSelectStatement<'static, T::SqlType, T, Sqlite>>,
    R: LoadingHandler<SqliteConnection, Table = T, SqlType = T::SqlType>
        + GraphQLType<Context = (), TypeInfo = ()>,
{
    fn handle_insert(
        executor: &Executor<PooledConnection<ConnectionManager<SqliteConnection>>>,
        insertable: I,
    ) -> ExecutionResult {
        let conn = executor.context();
        conn.transaction(|| -> ExecutionResult {
            insertable.insert_into(T::table()).execute(conn)?;
            let q = OrderDsl::order(T::table(), sql::<Bool>("rowid DESC")).into_boxed();
            let q = LimitDsl::limit(q, 1);
            let items = R::load_item(&executor.look_ahead(), conn, q)?;
            executor.resolve_with_ctx(&(), &items.into_iter().next())
        })
    }

    fn handle_batch_insert(
        executor: &Executor<PooledConnection<ConnectionManager<SqliteConnection>>>,
        batch: Vec<I>,
    ) -> ExecutionResult {
        let conn = executor.context();
        conn.transaction(|| -> ExecutionResult {
            let n = batch.insert_into(T::table()).execute(conn)?;
            let q = OrderDsl::order(T::table(), sql::<Bool>("rowid DESC")).into_boxed();
            let q = LimitDsl::limit(q, n as i64);
            let items = R::load_item(&executor.look_ahead(), conn, q)?;
            executor.resolve_with_ctx(&(), &items)
        })
    }
}
