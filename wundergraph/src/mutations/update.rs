use diesel::backend::Backend;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::{Connection, EqAll, QueryDsl, RunQueryDsl, Table};
use diesel::query_builder::{AsChangeset, IntoUpdateTarget, QueryFragment};
use diesel::associations::{HasTable, Identifiable};
use diesel::query_dsl::methods::{BoxedDsl, FilterDsl, FindDsl, LimitDsl};
use diesel::backend::UsesAnsiSavepointSyntax;
use diesel::connection::AnsiTransactionManager;
use diesel::query_builder::BoxedSelectStatement;
use diesel::dsl::{Filter, Find, Limit};

use juniper::{ExecutionResult, Executor, GraphQLType};
use LoadingHandler;

pub trait HandleUpdate<Conn, C, R>
where
    Conn: Connection + 'static,
{
    fn handle_update<'a, Tab>(
        execute: &Executor<PooledConnection<ConnectionManager<Conn>>>,
        change_set: &'a C,
    ) -> ExecutionResult
    where &'a C: Identifiable + AsChangeset<Target = Tab> + HasTable<Table = Tab>,
          <&'a C as AsChangeset>::Changeset: QueryFragment<Conn::Backend>,
          Tab: Table + HasTable<Table = Tab>,
          Tab::PrimaryKey: EqAll<<&'a C as Identifiable>::Id>,
          Tab::FromClause: QueryFragment<Conn::Backend>,
          Tab: FindDsl<<&'a C as Identifiable>::Id>,
          Find<Tab, <&'a C as Identifiable>::Id>: IntoUpdateTarget<Table = Tab>,
    <Find<Tab, <&'a C as Identifiable>::Id> as IntoUpdateTarget>::WhereClause: QueryFragment<Conn::Backend>,
          Tab::Query: FilterDsl<<Tab::PrimaryKey as EqAll<<&'a C as Identifiable>::Id>>::Output>,
    Filter<Tab::Query, <Tab::PrimaryKey as EqAll<<&'a C as Identifiable>::Id>>::Output>: LimitDsl,
    Limit<Filter<Tab::Query, <Tab::PrimaryKey as EqAll<<&'a C as Identifiable>::Id>>::Output>>: QueryDsl + BoxedDsl<'a, Conn::Backend, Output = BoxedSelectStatement<'a, Tab::SqlType, Tab, Conn::Backend>>,
        R:  LoadingHandler<Conn::Backend, Table = Tab, SqlType = Tab::SqlType>
        + GraphQLType<TypeInfo = (), Context = ()>,
    ;
}

impl<Conn, C, R> HandleUpdate<Conn, C, R> for Conn::Backend
where
    Conn: Connection<TransactionManager = AnsiTransactionManager> + 'static,
    Conn::Backend: UsesAnsiSavepointSyntax,
    <Conn::Backend as Backend>::QueryBuilder: Default,
{
    fn handle_update<'a, Tab>(
        executor: &Executor<PooledConnection<ConnectionManager<Conn>>>,
        change_set: &'a C,
    ) -> ExecutionResult
    where &'a C: Identifiable + AsChangeset<Target = Tab> + HasTable<Table = Tab>,
          <&'a C as AsChangeset>::Changeset: QueryFragment<Conn::Backend>,
          Tab: Table + HasTable<Table = Tab>,
          Tab::PrimaryKey: EqAll<<&'a C as Identifiable>::Id>,
          Tab::FromClause: QueryFragment<Conn::Backend>,
          Tab: FindDsl<<&'a C as Identifiable>::Id>,
          Find<Tab, <&'a C as Identifiable>::Id>: IntoUpdateTarget<Table = Tab>,
    <Find<Tab, <&'a C as Identifiable>::Id> as IntoUpdateTarget>::WhereClause: QueryFragment<Conn::Backend>,
              Tab::Query: FilterDsl<<Tab::PrimaryKey as EqAll<<&'a C as Identifiable>::Id>>::Output>,
    Filter<Tab::Query, <Tab::PrimaryKey as EqAll<<&'a C as Identifiable>::Id>>::Output>: LimitDsl,
    Limit<Filter<Tab::Query, <Tab::PrimaryKey as EqAll<<&'a C as Identifiable>::Id>>::Output>>: QueryDsl + BoxedDsl<'a, Conn::Backend, Output = BoxedSelectStatement<'a, Tab::SqlType, Tab, Conn::Backend>>,
        R:  LoadingHandler<Conn::Backend, Table = Tab, SqlType = Tab::SqlType>
        + GraphQLType<TypeInfo = (), Context = ()>,

    {
        let conn = executor.context();
        conn.transaction(|| -> ExecutionResult {
            let u = ::diesel::update(change_set).set(change_set);
            println!("{}", ::diesel::debug_query(&u));
            u.execute(conn)?;
            let f = FilterDsl::filter(
                Tab::table(),
                Tab::table().primary_key().eq_all(change_set.id()),
            );
            let q = LimitDsl::limit(f, 1).into_boxed();
            let items = R::load_item(&executor.look_ahead(), conn, q)?;
            executor.resolve_with_ctx(&(), &items.into_iter().next())
        })
    }
}
