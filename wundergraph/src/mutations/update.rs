use diesel::associations::{HasTable, Identifiable};
use diesel::backend::Backend;
use diesel::dsl::{Filter, Find, Limit};
use diesel::query_builder::BoxedSelectStatement;
use diesel::query_builder::{AsChangeset, IntoUpdateTarget, QueryFragment};
use diesel::query_dsl::methods::{BoxedDsl, FilterDsl, FindDsl, LimitDsl};
use diesel::{Connection, EqAll, QueryDsl, RunQueryDsl, Table};
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
    T::FromClause: QueryFragment<DB>,
    <Find<T, <&'static U as Identifiable>::Id> as IntoUpdateTarget>::WhereClause: QueryFragment<DB>,
    <&'static U as AsChangeset>::Changeset: QueryFragment<DB>,
    DB::QueryBuilder: Default,
    T::Query: FilterDsl<<T::PrimaryKey as EqAll<<&'static U as Identifiable>::Id>>::Output>,
    Filter<T::Query, <T::PrimaryKey as EqAll<<&'static U as Identifiable>::Id>>::Output>: LimitDsl,
    Limit<Filter<T::Query, <T::PrimaryKey as EqAll<<&'static U as Identifiable>::Id>>::Output>>:
        QueryDsl + BoxedDsl<'static, DB, Output = BoxedSelectStatement<'static, T::SqlType, T, DB>>,
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
            let items = R::load_items(&executor.look_ahead(), ctx, q)?;
            executor.resolve_with_ctx(&(), &items.into_iter().next())
        })
    }
}
