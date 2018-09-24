use diesel::associations::HasTable;
use diesel::backend::Backend;
use diesel::dsl::{Filter, Limit};
use diesel::query_builder::AsQuery;
use diesel::query_builder::BoxedSelectStatement;
use diesel::query_builder::{IntoUpdateTarget, QueryFragment, QueryId};
use diesel::query_dsl::methods::{BoxedDsl, FilterDsl, LimitDsl};
use diesel::Identifiable;
use diesel::{Connection, EqAll, QueryDsl, RunQueryDsl, Table};

use juniper::{
    Arguments, ExecutionResult, Executor, FieldError, FromInputValue, GraphQLType, Value,
};

use scalar::WundergraphScalarValue;
use LoadingHandler;
use WundergraphContext;

pub fn handle_delete<DB, D, R, Ctx>(
    executor: &Executor<Ctx, WundergraphScalarValue>,
    arguments: &Arguments<WundergraphScalarValue>,
    field_name: &'static str,
) -> ExecutionResult<WundergraphScalarValue>
where
    DB: Backend,
    Ctx: WundergraphContext<DB>,
    D: DeleteHelper<DB, R, Ctx> + FromInputValue<WundergraphScalarValue>,
    D::Handler: HandleDelete<DB, R, Ctx, Delete = D>,
{
    if let Some(n) = arguments.get::<D>(field_name) {
        D::Handler::handle_delete(executor, &n)
    } else {
        let msg = format!("Missing argument {:?}", field_name);
        Err(FieldError::new(&msg, Value::Null))
    }
}

pub trait HandleDelete<DB, R, Ctx>: Sized {
    type Delete;

    fn handle_delete(
        executor: &Executor<Ctx, WundergraphScalarValue>,
        to_delete: &Self::Delete,
    ) -> ExecutionResult<WundergraphScalarValue>;
}

pub trait DeleteHelper<DB, R, Ctx> {
    type Handler: HandleDelete<DB, R, Ctx>;
}

#[doc(hidden)]
#[derive(Debug)]
pub struct DeleteableWrapper<D>(D);

impl<I, R, DB, Ctx, T> DeleteHelper<DB, R, Ctx> for I
where
    I: 'static,
    DB: Backend,
    &'static I: Identifiable<Table = T>,
    DeleteableWrapper<I>: HandleDelete<DB, R, Ctx>,
    T: Table + HasTable<Table = T> + AsQuery + QueryId,
    T::PrimaryKey: EqAll<<&'static I as Identifiable>::Id>,
    Ctx: WundergraphContext<DB>,
    T::Query: FilterDsl<<T::PrimaryKey as EqAll<<&'static I as Identifiable>::Id>>::Output>,
    R::Query: FilterDsl<<T::PrimaryKey as EqAll<<&'static I as Identifiable>::Id>>::Output>,
    Filter<R::Query, <T::PrimaryKey as EqAll<<&'static I as Identifiable>::Id>>::Output>: LimitDsl,
    Limit<Filter<R::Query, <T::PrimaryKey as EqAll<<&'static I as Identifiable>::Id>>::Output>>:
        QueryDsl + BoxedDsl<'static, DB, Output = BoxedSelectStatement<'static, R::SqlType, T, DB>>,
    R: LoadingHandler<DB, Table = T, Context = Ctx>
        + GraphQLType<WundergraphScalarValue, TypeInfo = (), Context = ()>,
{
    type Handler = DeleteableWrapper<I>;
}

// We use the 'static static lifetime here because otherwise rustc will
// tell us that it could not find a applying lifetime (caused by broken projection
// on higher ranked lifetime bounds)
impl<DB, R, Ctx, T, I> HandleDelete<DB, R, Ctx> for DeleteableWrapper<I>
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
    + GraphQLType<WundergraphScalarValue, TypeInfo = (), Context = ()>,
    T::FromClause: QueryFragment<DB>,
    DB::QueryBuilder: Default,
    <Filter<T::Query, <T::PrimaryKey as EqAll<<&'static I as Identifiable>::Id>>::Output> as IntoUpdateTarget>::WhereClause: QueryFragment<DB>
        + QueryId,
    <T::PrimaryKey as EqAll<<&'static I as Identifiable>::Id>>::Output: Copy
{
    type Delete = I;

    fn handle_delete(executor: &Executor<Ctx, WundergraphScalarValue>, to_delete: &Self::Delete) -> ExecutionResult<WundergraphScalarValue> {
        let ctx = executor.context();
        let conn = ctx.get_connection();
        conn.transaction(|| -> ExecutionResult<WundergraphScalarValue> {
            // this is safe becuse we do not leek self out of this function
            let static_to_delete: &'static I = unsafe{ &*(to_delete as *const I) };
            let filter =  T::table().primary_key().eq_all(static_to_delete.id());
            let to_delete = FilterDsl::filter(R::default_query(), filter);
            // We use identifiable so there should only be one element affected by this query
            let q = LimitDsl::limit(to_delete, 1).into_boxed();
            let items = R::load_items(&executor.look_ahead(), ctx, q)?;
            let to_delete = FilterDsl::filter(T::table(), filter);
            let d = ::diesel::delete(to_delete);
            if cfg!(feature = "debug") {
                println!("{}", ::diesel::debug_query(&d));
            }
            assert_eq!(1, d.execute(conn)?);
            executor.resolve_with_ctx(&(), &items.into_iter().next())
        })
    }
}
