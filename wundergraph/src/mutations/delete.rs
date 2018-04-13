use diesel::backend::Backend;
use diesel::{Connection, EqAll, QueryDsl, RunQueryDsl, Table};
use diesel::query_builder::{IntoUpdateTarget, Query, QueryFragment, QueryId};
use diesel::associations::HasTable;
use diesel::query_dsl::methods::{BoxedDsl, FilterDsl, LimitDsl};
use diesel::query_builder::BoxedSelectStatement;
use diesel::query_builder::AsQuery;
use diesel::dsl::{Filter, Limit};
use diesel::Identifiable;
use WundergraphContext;

use juniper::{Arguments, ExecutionResult, Executor, FieldError, FromInputValue, GraphQLType, Value};
use LoadingHandler;

pub fn handle_delete<DB, I, R, Ctx>(
    executor: &Executor<Ctx>,
    arguments: &Arguments,
    field_name: &'static str,
) -> ExecutionResult
where
    DB: Backend,
    Ctx: WundergraphContext<DB>,
    I: HandleDelete<DB, R, Ctx>,
    I: FromInputValue,
{
    if let Some(n) = arguments.get::<I>(field_name) {
        HandleDelete::handle_delete(&n, executor)
    } else {
        let msg = format!("Missing argument {:?}", field_name);
        Err(FieldError::new(&msg, Value::Null))
    }
}

pub trait HandleDelete<DB, R, Ctx>: Sized {
    fn handle_delete(&self, executor: &Executor<Ctx>) -> ExecutionResult;
}

// We use the 'static static lifetime here because otherwise rustc will
// tell us that it could not find a applying lifetime (caused by broken projection
// on higher ranked lifetime bounds)
impl<DB, R, Ctx, T, I, Q> HandleDelete<DB, R, Ctx> for I
where
    I: 'static,
    DB: Backend,
    &'static I: Identifiable<Table = T>,
    T: Table + HasTable<Table = T> + AsQuery<Query = Q> + QueryId,
    T::PrimaryKey: EqAll<<&'static I as Identifiable>::Id>,
    Ctx: WundergraphContext<DB>,
    Q: Query<SqlType = T::SqlType>
        + FilterDsl<<T::PrimaryKey as EqAll<<&'static I as Identifiable>::Id>>::Output>,
    Filter<Q, <T::PrimaryKey as EqAll<<&'static I as Identifiable>::Id>>::Output>: Copy
        + IntoUpdateTarget<Table = T>
        + LimitDsl,
    Limit<Filter<Q, <T::PrimaryKey as EqAll<<&'static I as Identifiable>::Id>>::Output>>: QueryDsl
        + BoxedDsl<'static, DB, Output = BoxedSelectStatement<'static, T::SqlType, T, DB>>,
    R: LoadingHandler<DB, Table = T, SqlType = T::SqlType, Context = Ctx>
//        + WundergraphEntity<DB, Context = Ctx>
        + GraphQLType<TypeInfo = (), Context = ()>,
    T::FromClause: QueryFragment<DB>,
    DB::QueryBuilder: Default,
    <Filter<Q, <T::PrimaryKey as EqAll<<&'static I as Identifiable>::Id>>::Output> as IntoUpdateTarget>::WhereClause: QueryFragment<DB>
        + QueryId,
{
    fn handle_delete(&self, executor: &Executor<Ctx>) -> ExecutionResult {
        let ctx = executor.context();
        let conn = ctx.get_connection();
        conn.transaction(|| -> ExecutionResult {
            // this is safe becuse we do not leek self out of this function
            let static_self: &'static I = unsafe{ ::std::mem::transmute(self) };
            let to_delete =
                FilterDsl::filter(T::table(), T::table().primary_key().eq_all(static_self.id()));
            // We use identifiable so there should only be one element affected by this query
            let q = LimitDsl::limit(to_delete, 1).into_boxed();
            let items = R::load_item(&executor.look_ahead(), ctx, q)?;
            let d = ::diesel::delete(to_delete);
            println!("{}", ::diesel::debug_query(&d));
            assert_eq!(1, d.execute(conn)?);
            executor.resolve_with_ctx(&(), &items.into_iter().next())
        })
    }
}

// impl<T, Id, R, Conn, Q, Ctx> HandleDelete<Conn, T, Id, R, Ctx> for Conn::Backend
// where
//     Conn: Connection<TransactionManager = AnsiTransactionManager> + 'static,
//     Conn::Backend: UsesAnsiSavepointSyntax,
//     <Conn::Backend as Backend>::QueryBuilder: Default,
//     T: Table + HasTable<Table = T> + QueryId + AsQuery<Query = Q>,
//     Q: Query<SqlType = T::SqlType> + FilterDsl<<T::PrimaryKey as EqAll<Id>>::Output>,
//     T::PrimaryKey: EqAll<Id>,
//     T::FromClause: QueryFragment<Conn::Backend>,
//     T::AllColumns: QueryFragment<Conn::Backend> + QueryId,
//     Conn::Backend: HasSqlType<<T::AllColumns as Expression>::SqlType>,
//     R: LoadingHandler<Conn::Backend, Table = T, SqlType = T::SqlType>
//         + WundergraphEntity<Conn::Backend, Context = Ctx>
//         + GraphQLType<TypeInfo = (), Context = ()>,
//     R::QueryMiddleWare: BuildMiddleWare<R, Context = Ctx>,
//     Filter<Q, <T::PrimaryKey as EqAll<Id>>::Output>: Copy + IntoUpdateTarget<Table = T> + LimitDsl,
//     Limit<Filter<Q, <T::PrimaryKey as EqAll<Id>>::Output>>: QueryDsl
//         + BoxedDsl<
//         'static,
//         Conn::Backend,
//         Output = BoxedSelectStatement<'static, T::SqlType, T, Conn::Backend>,
//     >,
//     <Filter<Q, <T::PrimaryKey as EqAll<Id>>::Output> as IntoUpdateTarget>::WhereClause: QueryFragment<Conn::Backend>
//         + QueryId,
//     Ctx: WundergraphContext<Conn::Backend, Connection = Conn>,
// {
//     fn handle_delete(executor: &Executor<Ctx>, to_delete: Id) -> ExecutionResult {
//         let ctx = executor.context();
//         let conn = ctx.get_connection();
//         conn.transaction(|| -> ExecutionResult {
//             let to_delete =
//                 FilterDsl::filter(T::table(), T::table().primary_key().eq_all(to_delete));
//             let q = LimitDsl::limit(to_delete, 1).into_boxed();
//             let items = R::load_item(&executor.look_ahead(), ctx, q)?;
//             let d = ::diesel::delete(to_delete);
//             println!("{}", ::diesel::debug_query(&d));
//             d.execute(conn)?;
//             executor.resolve_with_ctx(&(), &items.into_iter().next())
//         })
//     }
// }
