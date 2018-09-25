use diesel::associations::HasTable;
use diesel::backend::Backend;
use diesel::query_builder::BoxedSelectStatement;
use diesel::query_builder::QueryFragment;
use diesel::query_dsl::methods::BoxedDsl;
use diesel::sql_types::Bool;
use diesel::{Connection, Insertable, QueryDsl, RunQueryDsl, Table};

#[cfg(feature = "postgres")]
use diesel::dsl::Filter;
#[cfg(feature = "postgres")]
use diesel::expression::{Expression, NonAggregate, SelectableExpression};
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
#[cfg(feature = "postgres")]
use diesel_ext::BoxableFilter;
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
    DB: Backend,
    InsertableWrapper<I>: HandleInsert<DB, R, Ctx>,
{
    type Handler = InsertableWrapper<I>;
}

#[cfg(feature = "postgres")]
impl<I, T, R, Ctx, Id> HandleInsert<Pg, R, Ctx> for InsertableWrapper<I>
where
    I: Insertable<T> + UndecoratedInsertRecord<T>,
    T: Table + HasTable<Table = T> + 'static,
    Ctx: WundergraphContext<Pg>,
    R: LoadingHandler<Pg, Table = T, Context = Ctx>
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
    R::Query: FilterDsl<<T::PrimaryKey as EqAll<Id>>::Output>,
    R::Query: FilterDsl<Box<BoxableFilter<T, Pg, SqlType = Bool>>>,
    T::PrimaryKey: EqAll<Id>,
    <T::PrimaryKey as EqAll<Id>>::Output:
        SelectableExpression<T> + NonAggregate + QueryFragment<Pg> + 'static,
    Filter<R::Query, <T::PrimaryKey as EqAll<Id>>::Output>:
        QueryDsl + BoxedDsl<'static, Pg, Output = BoxedSelectStatement<'static, R::SqlType, T, Pg>>,
    Filter<R::Query, Box<BoxableFilter<T, Pg, SqlType = Bool>>>:
        QueryDsl + BoxedDsl<'static, Pg, Output = BoxedSelectStatement<'static, R::SqlType, T, Pg>>,
{
    type Insert = I;

    fn handle_insert(
        executor: &Executor<Ctx, WundergraphScalarValue>,
        insertable: Self::Insert,
    ) -> ExecutionResult<WundergraphScalarValue> {
        let ctx = executor.context();
        let conn = ctx.get_connection();
        conn.transaction(|| -> ExecutionResult<WundergraphScalarValue> {
            let inserted = insertable
                .insert_into(T::table())
                .returning(T::table().primary_key());
            if cfg!(feature = "debug") {
                debug!("{}", ::diesel::debug_query(&inserted));
            }
            let inserted: Id = inserted.get_result(conn)?;

            let q = FilterDsl::filter(
                R::default_query(),
                T::table().primary_key().eq_all(inserted),
            );
            let q = q.into_boxed();
            let items = R::load_items(&executor.look_ahead(), ctx, q)?;
            executor.resolve_with_ctx(&(), &items.iter().next())
        })
    }

    fn handle_batch_insert(
        executor: &Executor<Ctx, WundergraphScalarValue>,
        batch: Vec<Self::Insert>,
    ) -> ExecutionResult<WundergraphScalarValue> {
        use diesel::BoolExpressionMethods;
        let ctx = executor.context();
        let conn = ctx.get_connection();
        conn.transaction(|| -> ExecutionResult<WundergraphScalarValue> {
            let inserted = batch
                .insert_into(T::table())
                .returning(T::table().primary_key());
            if cfg!(feature = "debug") {
                debug!("{}", ::diesel::debug_query(&inserted));
            }
            let inserted: Vec<Id> = inserted.get_results(conn)?;

            let mut ids = inserted.into_iter();
            if let Some(id) = ids.next() {
                let mut f = Box::new(T::table().primary_key().eq_all(id))
                    as Box<BoxableFilter<T, Pg, SqlType = Bool>>;
                for id in ids {
                    f = Box::new(f.or(T::table().primary_key().eq_all(id))) as Box<_>;
                }
                let q = FilterDsl::filter(R::default_query(), f).into_boxed();
                let items = R::load_items(&executor.look_ahead(), ctx, q)?;
                executor.resolve_with_ctx(&(), &items)
            } else {
                Ok(Value::Null)
            }
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
    R: LoadingHandler<Sqlite, Table = T, Context = Ctx>
        + GraphQLType<WundergraphScalarValue, TypeInfo = (), Context = ()>,
    T::FromClause: QueryFragment<Sqlite>,
    InsertStatement<T, I::Values>: ExecuteDsl<Ctx::Connection>,
    R::Query: QueryDsl
        + BoxedDsl<
            'static,
            Sqlite,
            Output = BoxedSelectStatement<'static, R::SqlType, T, Sqlite>,
        >,
{
    type Insert = I;

    fn handle_insert(
        executor: &Executor<Ctx, WundergraphScalarValue>,
        insertable: Self::Insert,
    ) -> ExecutionResult<WundergraphScalarValue> {
        let ctx = executor.context();
        let conn = ctx.get_connection();
        conn.transaction(|| -> ExecutionResult<WundergraphScalarValue> {
            insertable.insert_into(T::table()).execute(conn)?;
            let q = OrderDsl::order(R::default_query().into_boxed(), sql::<Bool>("rowid DESC"));
            let q = LimitDsl::limit(q, 1);
            let items = R::load_items(&executor.look_ahead(), ctx, q)?;
            executor.resolve_with_ctx(&(), &items.into_iter().next())
        })
    }

    fn handle_batch_insert(
        executor: &Executor<Ctx, WundergraphScalarValue>,
        batch: Vec<Self::Insert>,
    ) -> ExecutionResult<WundergraphScalarValue> {
        let ctx = executor.context();
        let conn = ctx.get_connection();
        conn.transaction(|| -> ExecutionResult<WundergraphScalarValue> {
            let n: usize = batch
                .into_iter()
                .map(|i| i.insert_into(T::table()).execute(conn))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .sum();
            let q = OrderDsl::order(R::default_query().into_boxed(), sql::<Bool>("rowid DESC"));
            let q = LimitDsl::limit(q, n as i64);
            let items = R::load_items(&executor.look_ahead(), ctx, q)?;
            executor.resolve_with_ctx(&(), &items)
        })
    }
}
