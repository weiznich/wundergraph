use diesel::associations::HasTable;
use diesel::backend::Backend;
use diesel::query_builder::BoxedSelectStatement;
use diesel::query_builder::QueryFragment;
use diesel::query_dsl::methods::BoxedDsl;
use diesel::sql_types::Bool;
use diesel::{Connection, Insertable, QueryDsl, RunQueryDsl, Table};
use WundergraphContext;

#[cfg(feature = "postgres")]
use helper::primary_keys::UnRef;
#[cfg(feature = "postgres")]
use diesel::dsl::Filter;
#[cfg(feature = "postgres")]
use diesel::expression::{BoxableExpression, Expression, NonAggregate, SelectableExpression};
#[cfg(feature = "postgres")]
use diesel::insertable::CanInsertInSingleQuery;
#[cfg(feature = "postgres")]
use diesel::pg::Pg;
#[cfg(feature = "postgres")]
use diesel::query_builder::UndecoratedInsertRecord;
#[cfg(feature = "postgres")]
use diesel::query_dsl::methods::FilterDsl;
#[cfg(feature = "postgres")]
use diesel::sql_types::HasSqlType;
#[cfg(feature = "postgres")]
use diesel::Identifiable;
#[cfg(feature = "postgres")]
use diesel::{EqAll, Queryable};

#[cfg(feature = "sqlite")]
use diesel::dsl::Order;
#[cfg(feature = "sqlite")]
use diesel::expression::dsl::sql;
#[cfg(feature = "sqlite")]
use diesel::expression::SqlLiteral;
#[cfg(feature = "sqlite")]
use diesel::query_builder::InsertStatement;
#[cfg(feature = "sqlite")]
use diesel::query_dsl::methods::{ExecuteDsl, LimitDsl, OrderDsl};
#[cfg(feature = "sqlite")]
use diesel::sqlite::Sqlite;

use juniper::{Arguments, ExecutionResult, Executor, FieldError, FromInputValue, GraphQLType, Value};
use LoadingHandler;

pub fn handle_batch_insert<DB, I, R, Ctx>(
    executor: &Executor<Ctx>,
    arguments: &Arguments,
    field_name: &'static str,
) -> ExecutionResult
where
    DB: Backend,
    Ctx: WundergraphContext<DB>,
    I: HandleInsert<DB, R, Ctx> + FromInputValue,
{
    if let Some(n) = arguments.get::<Vec<I>>(field_name) {
        I::handle_batch_insert(executor, n)
    } else {
        let msg = format!("Missing argument {}", field_name);
        Err(FieldError::new(&msg, Value::Null))
    }
}

pub fn handle_insert<DB, I, R, Ctx>(
    executor: &Executor<Ctx>,
    arguments: &Arguments,
    field_name: &'static str,
) -> ExecutionResult
where
    DB: Backend,
    Ctx: WundergraphContext<DB>,
    I: HandleInsert<DB, R, Ctx> + FromInputValue,
{
    if let Some(n) = arguments.get::<I>(field_name) {
        I::handle_insert(executor, n)
    } else {
        let msg = format!("Missing argument {}", field_name);
        Err(FieldError::new(&msg, Value::Null))
    }
}

pub trait HandleInsert<DB, R, Ctx>: Sized {
    fn handle_insert(executor: &Executor<Ctx>, insertable: Self) -> ExecutionResult;
    fn handle_batch_insert(executor: &Executor<Ctx>, insertable: Vec<Self>) -> ExecutionResult;
}

#[cfg(feature = "postgres")]
impl<I, T, R, Ctx, Id> HandleInsert<Pg, R, Ctx> for I
where
    I: Insertable<T> + UndecoratedInsertRecord<T>,
    T: Table + HasTable<Table = T> + 'static,
    Ctx: WundergraphContext<Pg>,
    R: LoadingHandler<Pg, Table = T, SqlType = T::SqlType, Context = Ctx>
        + GraphQLType<TypeInfo = (), Context = ()>
        + 'static,
    Pg: HasSqlType<<T::PrimaryKey as Expression>::SqlType>,
    Id: Queryable<<T::PrimaryKey as Expression>::SqlType, Pg>,
    T::FromClause: QueryFragment<Pg>,
    T::PrimaryKey: QueryFragment<Pg>,
    I::Values: QueryFragment<Pg> + CanInsertInSingleQuery<Pg>,
    <Vec<I> as Insertable<T>>::Values: QueryFragment<Pg> + CanInsertInSingleQuery<Pg>,
    &'static R: Identifiable,
    <&'static R as Identifiable>::Id: UnRef<'static, UnRefed = Id>,
    T::Query: FilterDsl<<T::PrimaryKey as EqAll<Id>>::Output>,
    T::Query: FilterDsl<Box<BoxableExpression<T, Pg, SqlType = Bool>>>,
    T::PrimaryKey: EqAll<Id>,
    <T::PrimaryKey as EqAll<Id>>::Output:
        SelectableExpression<T> + NonAggregate + QueryFragment<Pg> + 'static,
    Filter<T::Query, <T::PrimaryKey as EqAll<Id>>::Output>:
        QueryDsl + BoxedDsl<'static, Pg, Output = BoxedSelectStatement<'static, T::SqlType, T, Pg>>,
    Filter<T::Query, Box<BoxableExpression<T, Pg, SqlType = Bool>>>:
        QueryDsl + BoxedDsl<'static, Pg, Output = BoxedSelectStatement<'static, T::SqlType, T, Pg>>,
{
    fn handle_insert(executor: &Executor<Ctx>, insertable: I) -> ExecutionResult {
        let ctx = executor.context();
        let conn = ctx.get_connection();
        conn.transaction(|| -> ExecutionResult {
            let inserted = insertable
                .insert_into(T::table())
                .returning(T::table().primary_key());
            println!("{}", ::diesel::debug_query(&inserted));
            let inserted: Id = inserted.get_result(conn)?;

            let q = FilterDsl::filter(T::table(), T::table().primary_key().eq_all(inserted));
            let q = q.into_boxed();
            let items = R::load_items(&executor.look_ahead(), ctx, q)?;
            executor.resolve_with_ctx(&(), &items.iter().next())
        })
    }

    fn handle_batch_insert(executor: &Executor<Ctx>, batch: Vec<I>) -> ExecutionResult {
        use diesel::BoolExpressionMethods;
        let ctx = executor.context();
        let conn = ctx.get_connection();
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
                let items = R::load_items(&executor.look_ahead(), ctx, q)?;
                executor.resolve_with_ctx(&(), &items)
            } else {
                Ok(Value::Null)
            }
        })
    }
}

#[cfg(feature = "sqlite")]
impl<I, T, R, Ctx> HandleInsert<Sqlite, R, Ctx> for I
where
    I: Insertable<T>,
    Vec<I>: Insertable<T>,
    T: Table + HasTable<Table = T> + 'static,
    Ctx: WundergraphContext<Sqlite>,
    R: LoadingHandler<Sqlite, Table = T, SqlType = T::SqlType, Context = Ctx>
        + GraphQLType<TypeInfo = (), Context = ()>,
    T::FromClause: QueryFragment<Sqlite>,
    InsertStatement<T, I::Values>: ExecuteDsl<Ctx::Connection>,
    T::Query: OrderDsl<SqlLiteral<Bool>>,
    Order<T::Query, SqlLiteral<Bool>>: QueryDsl
        + BoxedDsl<
            'static,
            Sqlite,
            Output = BoxedSelectStatement<'static, T::SqlType, T, Sqlite>,
        >,
{
    fn handle_insert(executor: &Executor<Ctx>, insertable: I) -> ExecutionResult {
        let ctx = executor.context();
        let conn = ctx.get_connection();
        conn.transaction(|| -> ExecutionResult {
            insertable.insert_into(T::table()).execute(conn)?;
            let q = OrderDsl::order(T::table(), sql::<Bool>("rowid DESC")).into_boxed();
            let q = LimitDsl::limit(q, 1);
            let items = R::load_items(&executor.look_ahead(), ctx, q)?;
            executor.resolve_with_ctx(&(), &items.into_iter().next())
        })
    }

    fn handle_batch_insert(executor: &Executor<Ctx>, batch: Vec<I>) -> ExecutionResult {
        let ctx = executor.context();
        let conn = ctx.get_connection();
        conn.transaction(|| -> ExecutionResult {
            let n = batch
                .into_iter()
                .map(|i| i.insert_into(T::table()).execute(conn))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .fold(0, |acc, n| acc + n);
            let q = OrderDsl::order(T::table(), sql::<Bool>("rowid DESC")).into_boxed();
            let q = LimitDsl::limit(q, n as i64);
            let items = R::load_items(&executor.look_ahead(), ctx, q)?;
            executor.resolve_with_ctx(&(), &items)
        })
    }
}
