use diesel::associations::{HasTable, Identifiable};
use diesel::backend::Backend;
use diesel::dsl::{Filter, Find, Limit};
use diesel::query_builder::BoxedSelectStatement;
use diesel::query_builder::{AsChangeset, IntoUpdateTarget, QueryFragment};
use diesel::query_dsl::methods::{BoxedDsl, FilterDsl, FindDsl, LimitDsl};
use diesel::{Connection, EqAll, QueryDsl, RunQueryDsl, Table};
use WundergraphContext;

use juniper::{
    Arguments, ExecutionResult, Executor, FieldError, FromInputValue, GraphQLType, Value,
};
use LoadingHandler;

pub fn handle_update<DB, U, R, Ctx>(
    executor: &Executor<Ctx>,
    arguments: &Arguments,
    field_name: &'static str,
) -> ExecutionResult
where
    DB: Backend,
    Ctx: WundergraphContext<DB>,
    U: UpdateHelper<DB, R, Ctx> + FromInputValue,
    U::Handler: HandleUpdate<DB, R, Ctx, Update = U>,
{
    if let Some(n) = arguments.get::<U>(field_name) {
        U::Handler::handle_update(executor, &n)
    } else {
        let msg = format!("Missing argument {:?}", field_name);
        Err(FieldError::new(&msg, Value::Null))
    }
}

pub trait HandleUpdate<DB, R, Ctx>: Sized {
    type Update;
    fn handle_update(executor: &Executor<Ctx>, update: &Self::Update) -> ExecutionResult;
}

pub trait UpdateHelper<DB, R, Ctx> {
    type Handler: HandleUpdate<DB, R, Ctx>;
}

#[doc(hidden)]
pub struct AsChangeSetWrapper<U>(U);

impl<U, DB, R, Ctx, T> UpdateHelper<DB, R, Ctx> for U
where
    U: 'static,
    DB: Backend,
    &'static U: AsChangeset<Target = T> + HasTable<Table = T> + Identifiable,
    AsChangeSetWrapper<U>: HandleUpdate<DB, R, Ctx>,

    T: Table + HasTable<Table = T> + FindDsl<<&'static U as Identifiable>::Id>,
    Ctx: WundergraphContext<DB>,
    Find<T, <&'static U as Identifiable>::Id>: IntoUpdateTarget<Table = T>,
    R: LoadingHandler<DB, Table = T, Context = Ctx> + GraphQLType<TypeInfo = (), Context = ()>,
    R::Query: FilterDsl<<T::PrimaryKey as EqAll<<&'static U as Identifiable>::Id>>::Output>,
    Filter<R::Query, <T::PrimaryKey as EqAll<<&'static U as Identifiable>::Id>>::Output>: LimitDsl,
    Limit<Filter<R::Query, <T::PrimaryKey as EqAll<<&'static U as Identifiable>::Id>>::Output>>:
        QueryDsl + BoxedDsl<'static, DB, Output = BoxedSelectStatement<'static, R::SqlType, T, DB>>,
    T::PrimaryKey: EqAll<<&'static U as Identifiable>::Id>,
{
    type Handler = AsChangeSetWrapper<U>;
}

// We use the 'static static lifetime here because otherwise rustc will
// tell us that it could not find a applying lifetime (caused by broken projection
// on higher ranked lifetime bounds)
impl<DB, R, Ctx, T, U> HandleUpdate<DB, R, Ctx> for AsChangeSetWrapper<U>
where
    U: 'static,
    DB: Backend,
    &'static U: AsChangeset<Target = T> + HasTable<Table = T> + Identifiable,
    T: Table + HasTable<Table = T> + FindDsl<<&'static U as Identifiable>::Id>,
    Ctx: WundergraphContext<DB>,
    Find<T, <&'static U as Identifiable>::Id>: IntoUpdateTarget<Table = T>,
    R: LoadingHandler<DB, Table = T, Context = Ctx> + GraphQLType<TypeInfo = (), Context = ()>,
    T::FromClause: QueryFragment<DB>,
    <Find<T, <&'static U as Identifiable>::Id> as IntoUpdateTarget>::WhereClause: QueryFragment<DB>,
    <&'static U as AsChangeset>::Changeset: QueryFragment<DB>,
    DB::QueryBuilder: Default,
    R::Query: FilterDsl<<T::PrimaryKey as EqAll<<&'static U as Identifiable>::Id>>::Output>,
    Filter<R::Query, <T::PrimaryKey as EqAll<<&'static U as Identifiable>::Id>>::Output>: LimitDsl,
    Limit<Filter<R::Query, <T::PrimaryKey as EqAll<<&'static U as Identifiable>::Id>>::Output>>:
        QueryDsl + BoxedDsl<'static, DB, Output = BoxedSelectStatement<'static, R::SqlType, T, DB>>,
    T::PrimaryKey: EqAll<<&'static U as Identifiable>::Id>,
{
    type Update = U;

    fn handle_update(executor: &Executor<Ctx>, change_set: &Self::Update) -> ExecutionResult {
        let ctx = executor.context();
        let conn = ctx.get_connection();
        conn.transaction(|| -> ExecutionResult {
            let change_set: &'static U = unsafe { &*(change_set as *const U) };
            let u = ::diesel::update(change_set).set(change_set);
            println!("{}", ::diesel::debug_query(&u));
            u.execute(conn)?;
            let f = FilterDsl::filter(
                R::default_query(),
                T::table().primary_key().eq_all(change_set.id()),
            );
            // We use identifiable so there should only be one element affected by this query
            let q = LimitDsl::limit(f, 1).into_boxed();
            let items = R::load_items(&executor.look_ahead(), ctx, q)?;
            executor.resolve_with_ctx(&(), &items.into_iter().next())
        })
    }
}
