use diesel::backend::Backend;
use diesel::{Connection, EqAll, QueryDsl, RunQueryDsl, Table};
use diesel::query_builder::{AsChangeset, IntoUpdateTarget, QueryFragment};
use diesel::associations::{HasTable, Identifiable};
use diesel::query_dsl::methods::{BoxedDsl, FilterDsl, FindDsl, LimitDsl};
use diesel::query_builder::BoxedSelectStatement;
use diesel::dsl::{Filter, Find, Limit};
use WundergraphContext;

use juniper::{Arguments, ExecutionResult, Executor, FieldError, FromInputValue, GraphQLType, Value};
use LoadingHandler;

pub fn handle_update<DB, I, R, Ctx>(
    executor: &Executor<Ctx>,
    arguments: &Arguments,
    field_name: &'static str,
) -> ExecutionResult
where
    DB: Backend,
    Ctx: WundergraphContext<DB>,
    I: HandleUpdate<DB, R, Ctx>,
    I: FromInputValue,
{
    if let Some(n) = arguments.get::<I>(field_name) {
        HandleUpdate::handle_update(&n, executor)
    } else {
        let msg = format!("Missing argument {:?}", field_name);
        Err(FieldError::new(&msg, Value::Null))
    }
}

pub trait HandleUpdate<DB, R, Ctx>: Sized {
    fn handle_update(&self, executor: &Executor<Ctx>) -> ExecutionResult;
}

// We use the 'static static lifetime here because otherwise rustc will
// tell us that it could not find a applying lifetime (caused by broken projection
// on higher ranked lifetime bounds)
impl<DB, R, Ctx, T, U> HandleUpdate<DB, R, Ctx> for U
where
    U: 'static,
    DB: Backend,
    &'static U: AsChangeset<Target = T> + HasTable<Table = T> + Identifiable,
    T: Table + HasTable<Table = T> + FindDsl<<&'static U as Identifiable>::Id>,
    Ctx: WundergraphContext<DB>,
    Find<T, <&'static U as Identifiable>::Id>: IntoUpdateTarget<Table = T>,
    R: LoadingHandler<DB, Table = T, SqlType = T::SqlType, Context = Ctx>
        + GraphQLType<TypeInfo = (), Context = ()>,
//        + WundergraphEntity<DB, Context = Ctx>,
    T::FromClause: QueryFragment<DB>,
    <Find<T, <&'static U as Identifiable>::Id> as IntoUpdateTarget>::WhereClause: QueryFragment<DB>,
    <&'static U as AsChangeset>::Changeset: QueryFragment<DB>,
    DB::QueryBuilder: Default,
    T::Query: FilterDsl<<T::PrimaryKey as EqAll<<&'static U as Identifiable>::Id>>::Output>,
    Filter<T::Query, <T::PrimaryKey as EqAll<<&'static U as Identifiable>::Id>>::Output>: LimitDsl,
    Limit<Filter<T::Query, <T::PrimaryKey as EqAll<<&'static U as Identifiable>::Id>>::Output>>: QueryDsl
        + BoxedDsl<'static, DB, Output = BoxedSelectStatement<'static, T::SqlType, T, DB>>,
    T::PrimaryKey: EqAll<<&'static U as Identifiable>::Id>,
{
    fn handle_update(&self, executor: &Executor<Ctx>) -> ExecutionResult {
        let ctx = executor.context();
        let conn = ctx.get_connection();
        conn.transaction(|| -> ExecutionResult {
            let change_set: &'static U = unsafe { ::std::mem::transmute(self) };
            let u = ::diesel::update(change_set).set(change_set);
            println!("{}", ::diesel::debug_query(&u));
            u.execute(conn)?;
            let f = FilterDsl::filter(T::table(), T::table().primary_key().eq_all(change_set.id()));
            // We use identifiable so there should only be one element affected by this query
            let q = LimitDsl::limit(f, 1).into_boxed();
            let items = R::load_item(&executor.look_ahead(), ctx, q)?;
            executor.resolve_with_ctx(&(), &items.into_iter().next())
        })
    }
}

// pub trait HandleUpdate<Conn, C, R, Ctx>
// where
//     Conn: Connection + 'static,
//     Ctx: WundergraphContext<Conn::Backend>,
// {
//     fn handle_update<'a, Tab>(
//         execute: &Executor<Ctx>,
//         change_set: &'a C,
//     ) -> ExecutionResult
//     where &'a C: Identifiable + AsChangeset<Target = Tab> + HasTable<Table = Tab>,
//           <&'a C as AsChangeset>::Changeset: QueryFragment<Conn::Backend>,
//           Tab: Table + HasTable<Table = Tab>,
//           Tab::PrimaryKey: EqAll<<&'a C as Identifiable>::Id>,
//           Tab::FromClause: QueryFragment<Conn::Backend>,
//           Tab: FindDsl<<&'a C as Identifiable>::Id>,
//           Find<Tab, <&'a C as Identifiable>::Id>: IntoUpdateTarget<Table = Tab>,
//     <Find<Tab, <&'a C as Identifiable>::Id> as IntoUpdateTarget>::WhereClause: QueryFragment<Conn::Backend>,
//           Tab::Query: FilterDsl<<Tab::PrimaryKey as EqAll<<&'a C as Identifiable>::Id>>::Output>,
//     Filter<Tab::Query, <Tab::PrimaryKey as EqAll<<&'a C as Identifiable>::Id>>::Output>: LimitDsl,
//     Limit<Filter<Tab::Query, <Tab::PrimaryKey as EqAll<<&'a C as Identifiable>::Id>>::Output>>: QueryDsl + BoxedDsl<'a, Conn::Backend, Output = BoxedSelectStatement<'a, Tab::SqlType, Tab, Conn::Backend>>,
//         R:  LoadingHandler<Conn::Backend, Table = Tab, SqlType = Tab::SqlType>
//         + GraphQLType<TypeInfo = (), Context = ()>
//         + WundergraphEntity<Conn::Backend, Context = Ctx>,
//     ;
// }

// impl<Conn, C, R, Ctx> HandleUpdate<Conn, C, R, Ctx> for Conn::Backend
// where
//     Conn: Connection<TransactionManager = AnsiTransactionManager> + 'static,
//     Conn::Backend: UsesAnsiSavepointSyntax,
//     <Conn::Backend as Backend>::QueryBuilder: Default,
//     Ctx: WundergraphContext<Conn::Backend>,
// {
//     fn handle_update<'a, Tab>(
//         executor: &Executor<Ctx>,
//         change_set: &'a C,
//     ) -> ExecutionResult
//     where &'a C: Identifiable + AsChangeset<Target = Tab> + HasTable<Table = Tab>,
//           <&'a C as AsChangeset>::Changeset: QueryFragment<Conn::Backend>,
//           Tab: Table + HasTable<Table = Tab>,
//           Tab::PrimaryKey: EqAll<<&'a C as Identifiable>::Id>,
//           Tab::FromClause: QueryFragment<Conn::Backend>,
//           Tab: FindDsl<<&'a C as Identifiable>::Id>,
//           Find<Tab, <&'a C as Identifiable>::Id>: IntoUpdateTarget<Table = Tab>,
//     <Find<Tab, <&'a C as Identifiable>::Id> as IntoUpdateTarget>::WhereClause: QueryFragment<Conn::Backend>,
//               Tab::Query: FilterDsl<<Tab::PrimaryKey as EqAll<<&'a C as Identifiable>::Id>>::Output>,
//     Filter<Tab::Query, <Tab::PrimaryKey as EqAll<<&'a C as Identifiable>::Id>>::Output>: LimitDsl,
//     Limit<Filter<Tab::Query, <Tab::PrimaryKey as EqAll<<&'a C as Identifiable>::Id>>::Output>>: QueryDsl + BoxedDsl<'a, Conn::Backend, Output = BoxedSelectStatement<'a, Tab::SqlType, Tab, Conn::Backend>>,
//         R:  LoadingHandler<Conn::Backend, Table = Tab, SqlType = Tab::SqlType>
//         + GraphQLType<TypeInfo = (), Context = ()>
//         + WundergraphEntity<Conn::Backend, Context = Ctx>,
//     {
//         let ctx = executor.context();
//         let conn = ctx.get_connection();
//         conn.transaction(|| -> ExecutionResult {
//             let u = ::diesel::update(change_set).set(change_set);
//             println!("{}", ::diesel::debug_query(&u));
//             u.execute(conn)?;
//             let f = FilterDsl::filter(
//                 Tab::table(),
//                 Tab::table().primary_key().eq_all(change_set.id()),
//             );
//             let q = LimitDsl::limit(f, 1).into_boxed();
//             let items = R::load_item(&executor.look_ahead(), ctx, q)?;
//             executor.resolve_with_ctx(&(), &items.into_iter().next())
//         })
//     }
// }
