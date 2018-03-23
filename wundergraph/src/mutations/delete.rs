use diesel::backend::Backend;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::{Connection, EqAll, QueryDsl, RunQueryDsl, Table};
use diesel::query_builder::{IntoUpdateTarget, Query, QueryFragment, QueryId};
use diesel::associations::HasTable;
use diesel::query_dsl::methods::{BoxedDsl, FilterDsl, LimitDsl};
use diesel::backend::UsesAnsiSavepointSyntax;
use diesel::connection::AnsiTransactionManager;
use diesel::expression::Expression;
use diesel::sql_types::HasSqlType;
use diesel::query_builder::BoxedSelectStatement;
use diesel::query_builder::AsQuery;
use diesel::dsl::{Filter, Limit};

use juniper::{Arguments, ExecutionResult, Executor, FieldError, FromInputValue, GraphQLType, Value};
use LoadingHandler;

pub fn handle_delete<'a, Conn, R, Id, T, A>(
    executor: &Executor<PooledConnection<ConnectionManager<Conn>>>,
    arguments: &Arguments<'a>,
    field_names: A,
) -> ExecutionResult
where
    Conn: Connection + 'static,
    Conn::Backend: HandleDelete<Conn, T, Id, R>,
    Arguments<'a>: ReceiveDeleteId<A, Id>,
    A: ::std::fmt::Debug + Copy,
{
    if let Some(n) = arguments.id_from_args(field_names) {
        Conn::Backend::handle_delete(executor, n)
    } else {
        let msg = format!("Missing argument {:?}", field_names);
        Err(FieldError::new(&msg, Value::Null))
    }
}

pub trait ReceiveDeleteId<F, Id> {
    fn id_from_args(&self, arg_names: F) -> Option<Id>;
}

impl<'a, Id> ReceiveDeleteId<&'static str, Id> for Arguments<'a>
where
    Id: FromInputValue,
{
    fn id_from_args(&self, arg_name: &'static str) -> Option<Id> {
        self.get::<Id>(arg_name)
    }
}

impl<'a, A, B> ReceiveDeleteId<(&'static str, &'static str), (A, B)> for Arguments<'a>
where
    A: FromInputValue,
    B: FromInputValue,
{
    fn id_from_args(&self, (a1, a2): (&'static str, &'static str)) -> Option<(A, B)> {
        self.get::<A>(a1)
            .and_then(|a| self.get::<B>(a2).map(|b| (a, b)))
    }
}

impl<'a, A, B, C> ReceiveDeleteId<(&'static str, &'static str, &'static str), (A, B, C)>
    for Arguments<'a>
where
    A: FromInputValue,
    B: FromInputValue,
    C: FromInputValue,
{
    fn id_from_args(
        &self,
        (a1, a2, a3): (&'static str, &'static str, &'static str),
    ) -> Option<(A, B, C)> {
        self.get::<A>(a1)
            .and_then(|a| self.get::<B>(a2).map(|b| (a, b)))
            .and_then(|(a, b)| self.get::<C>(a3).map(|c| (a, b, c)))
    }
}

impl<
    'a,
    A,
    B,
    C,
    D,
> ReceiveDeleteId<(&'static str, &'static str, &'static str, &'static str), (A, B, C, D)>
    for Arguments<'a>
where
    A: FromInputValue,
    B: FromInputValue,
    C: FromInputValue,
    D: FromInputValue,
{
    fn id_from_args(
        &self,
        (a1, a2, a3, a4): (&'static str, &'static str, &'static str, &'static str),
    ) -> Option<(A, B, C, D)> {
        self.get::<A>(a1)
            .and_then(|a| self.get::<B>(a2).map(|b| (a, b)))
            .and_then(|(a, b)| self.get::<C>(a3).map(|c| (a, b, c)))
            .and_then(|(a, b, c)| self.get::<D>(a4).map(|d| (a, b, c, d)))
    }
}

impl<
    'a,
    A,
    B,
    C,
    D,
    E,
> ReceiveDeleteId<
    (
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
    ),
    (A, B, C, D, E),
> for Arguments<'a>
where
    A: FromInputValue,
    B: FromInputValue,
    C: FromInputValue,
    D: FromInputValue,
    E: FromInputValue,
{
    fn id_from_args(
        &self,
        (a1, a2, a3, a4, a5): (
            &'static str,
            &'static str,
            &'static str,
            &'static str,
            &'static str,
        ),
    ) -> Option<(A, B, C, D, E)> {
        self.get::<A>(a1)
            .and_then(|a| self.get::<B>(a2).map(|b| (a, b)))
            .and_then(|(a, b)| self.get::<C>(a3).map(|c| (a, b, c)))
            .and_then(|(a, b, c)| self.get::<D>(a4).map(|d| (a, b, c, d)))
            .and_then(|(a, b, c, d)| self.get::<E>(a5).map(|e| (a, b, c, d, e)))
    }
}

pub trait HandleDelete<Conn, T, Id, R>
where
    Conn: Connection + 'static,
{
    fn handle_delete(
        executor: &Executor<PooledConnection<ConnectionManager<Conn>>>,
        to_delete: Id,
    ) -> ExecutionResult;
}

impl<T, Id, R, Conn, Q> HandleDelete<Conn, T, Id, R> for Conn::Backend
where
    Conn: Connection<TransactionManager = AnsiTransactionManager> + 'static,
    Conn::Backend: UsesAnsiSavepointSyntax,
    <Conn::Backend as Backend>::QueryBuilder: Default,
    T: Table + HasTable<Table = T> + QueryId + AsQuery<Query = Q>,
    Q: Query<SqlType = T::SqlType> + FilterDsl<<T::PrimaryKey as EqAll<Id>>::Output>,
    T::PrimaryKey: EqAll<Id>,
    T::FromClause: QueryFragment<Conn::Backend>,
    T::AllColumns: QueryFragment<Conn::Backend> + QueryId,
    Conn::Backend: HasSqlType<<T::AllColumns as Expression>::SqlType>,
    R: LoadingHandler<Conn::Backend, Table = T, SqlType = T::SqlType>
        + GraphQLType<TypeInfo = (), Context = ()>,
    Filter<Q, <T::PrimaryKey as EqAll<Id>>::Output>: Copy + IntoUpdateTarget<Table = T> + LimitDsl,
    Limit<Filter<Q, <T::PrimaryKey as EqAll<Id>>::Output>>: QueryDsl
        + BoxedDsl<
        'static,
        Conn::Backend,
        Output = BoxedSelectStatement<'static, T::SqlType, T, Conn::Backend>,
    >,
    <Filter<Q, <T::PrimaryKey as EqAll<Id>>::Output> as IntoUpdateTarget>::WhereClause: QueryFragment<Conn::Backend>
        + QueryId,
{
    fn handle_delete(
        executor: &Executor<PooledConnection<ConnectionManager<Conn>>>,
        to_delete: Id,
    ) -> ExecutionResult {
        let conn = executor.context();
        conn.transaction(|| -> ExecutionResult {
            let to_delete =
                FilterDsl::filter(T::table(), T::table().primary_key().eq_all(to_delete));
            let q = LimitDsl::limit(to_delete, 1).into_boxed();
            let items = R::load_item(&executor.look_ahead(), conn, q)?;
            let d = ::diesel::delete(to_delete);
            println!("{}", ::diesel::debug_query(&d));
            d.execute(conn)?;
            executor.resolve_with_ctx(&(), &items.into_iter().next())
        })
    }
}
