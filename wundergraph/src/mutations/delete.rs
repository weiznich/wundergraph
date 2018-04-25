use diesel::associations::HasTable;
use diesel::backend::Backend;
use diesel::dsl::{Filter, Limit};
use diesel::query_builder::AsQuery;
use diesel::query_builder::BoxedSelectStatement;
use diesel::query_builder::{IntoUpdateTarget, QueryFragment, QueryId};
use diesel::query_dsl::methods::{BoxedDsl, FilterDsl, LimitDsl};
use diesel::Identifiable;
use diesel::{Connection, EqAll, QueryDsl, RunQueryDsl, Table};
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
impl<DB, R, Ctx, T, I> HandleDelete<DB, R, Ctx> for I
where
    I: 'static,
    DB: Backend,
    &'static I: Identifiable<Table = T>,
    T: Table + HasTable<Table = T> + AsQuery + QueryId,
    T::PrimaryKey: EqAll<<&'static I as Identifiable>::Id>,
    Ctx: WundergraphContext<DB>,
    T::Query: FilterDsl<<T::PrimaryKey as EqAll<<&'static I as Identifiable>::Id>>::Output>,
    R::Query:FilterDsl<<T::PrimaryKey as EqAll<<&'static I as Identifiable>::Id>>::Output>,
    Filter<T::Query, <T::PrimaryKey as EqAll<<&'static I as Identifiable>::Id>>::Output>: IntoUpdateTarget<Table = T>,
    Filter<R::Query, <T::PrimaryKey as EqAll<<&'static I as Identifiable>::Id>>::Output>:
         LimitDsl,
    Limit<Filter<R::Query, <T::PrimaryKey as EqAll<<&'static I as Identifiable>::Id>>::Output>>: QueryDsl
        + BoxedDsl<'static, DB, Output = BoxedSelectStatement<'static, R::SqlType, T, DB>>,
    R: LoadingHandler<DB, Table = T, Context = Ctx>
        + GraphQLType<TypeInfo = (), Context = ()>,
    T::FromClause: QueryFragment<DB>,
    DB::QueryBuilder: Default,
    <Filter<T::Query, <T::PrimaryKey as EqAll<<&'static I as Identifiable>::Id>>::Output> as IntoUpdateTarget>::WhereClause: QueryFragment<DB>
        + QueryId,
    <T::PrimaryKey as EqAll<<&'static I as Identifiable>::Id>>::Output: Copy
{
    fn handle_delete(&self, executor: &Executor<Ctx>) -> ExecutionResult {
        let ctx = executor.context();
        let conn = ctx.get_connection();
        conn.transaction(|| -> ExecutionResult {
            // this is safe becuse we do not leek self out of this function
            let static_self: &'static I = unsafe{ ::std::mem::transmute(self) };
            let filter =  T::table().primary_key().eq_all(static_self.id());
            let to_delete = FilterDsl::filter(R::default_query(), filter);
            // We use identifiable so there should only be one element affected by this query
            let q = LimitDsl::limit(to_delete, 1).into_boxed();
            let items = R::load_items(&executor.look_ahead(), ctx, q)?;
            let to_delete = FilterDsl::filter(T::table(), filter);
            let d = ::diesel::delete(to_delete);
            println!("{}", ::diesel::debug_query(&d));
            assert_eq!(1, d.execute(conn)?);
            executor.resolve_with_ctx(&(), &items.into_iter().next())
        })
    }
}
