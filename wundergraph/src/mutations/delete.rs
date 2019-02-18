use diesel::associations::HasTable;
use diesel::backend::Backend;
use diesel::dsl::{Filter, SqlTypeOf};
use diesel::expression::NonAggregate;
use diesel::query_builder::AsQuery;
use diesel::query_builder::BoxedSelectStatement;
use diesel::query_builder::{IntoUpdateTarget, QueryFragment, QueryId};
use diesel::query_dsl::methods::{BoxedDsl, FilterDsl, LimitDsl};
use diesel::sql_types::HasSqlType;
use diesel::Identifiable;
use diesel::Queryable;
use diesel::{AppearsOnTable, Connection, EqAll, RunQueryDsl, Table};

use juniper::{
    Arguments, ExecutionResult, Executor, FieldError, FromInputValue, GraphQLType, Value,
};

use filter::build_filter::BuildFilter;
use query_helper::order::BuildOrder;
use query_helper::placeholder::SqlTypeOfPlaceholder;
use query_helper::placeholder::WundergraphFieldList;
use query_helper::select::BuildSelect;
use query_helper::tuple::TupleIndex;
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

#[cfg_attr(feature = "cargo-clippy", allow(use_self))]
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
{
    type Handler = DeleteableWrapper<I>;
}

// We use the 'static static lifetime here because otherwise rustc will
// tell us that it could not find a applying lifetime (caused by broken projection
// on higher ranked lifetime bounds)
impl<DB, R, Ctx, T, I> HandleDelete<DB, R, Ctx> for DeleteableWrapper<I>
where
    I: 'static,
    DB: Backend + 'static,
    &'static I: Identifiable<Table = T>,
    T: Table + HasTable<Table = T> + AsQuery + QueryId + 'static,
    T::PrimaryKey: EqAll<<&'static I as Identifiable>::Id>,
    Ctx: WundergraphContext<DB>,
    T::Query: FilterDsl<<T::PrimaryKey as EqAll<<&'static I as Identifiable>::Id>>::Output>,
    Filter<T::Query, <T::PrimaryKey as EqAll<<&'static I as Identifiable>::Id>>::Output>:
        IntoUpdateTarget<Table = T>,
    R::Columns: BuildOrder<T, DB>
        + BuildSelect<
            T,
            DB,
            SqlTypeOfPlaceholder<R::FieldList, DB, R::PrimaryKeyIndex, R::Table>,
        >,
    R::FieldList: WundergraphFieldList<DB, R::PrimaryKeyIndex, T> + TupleIndex<R::PrimaryKeyIndex>,
    <R::FieldList as WundergraphFieldList<DB, R::PrimaryKeyIndex, T>>::PlaceHolder:
        Queryable<SqlTypeOfPlaceholder<R::FieldList, DB, R::PrimaryKeyIndex, R::Table>, DB>,
    R: LoadingHandler<DB, Table = T>
        + GraphQLType<WundergraphScalarValue, TypeInfo = (), Context = ()>,
    T::FromClause: QueryFragment<DB>,
    DB::QueryBuilder: Default,
    <Filter<T::Query, <T::PrimaryKey as EqAll<<&'static I as Identifiable>::Id>>::Output> as IntoUpdateTarget>::WhereClause: QueryFragment<DB>
       + QueryId,
    <T::PrimaryKey as EqAll<<&'static I as Identifiable>::Id>>::Output: Copy,
    for<'a> R::Table: BoxedDsl<
        'a,
        DB,
        Output = BoxedSelectStatement<'a, SqlTypeOf<<R::Table as Table>::AllColumns>, R::Table, DB>,
    >,
    <R::Filter as BuildFilter<DB>>::Ret: AppearsOnTable<T>,
    for<'a> BoxedSelectStatement<'a, SqlTypeOf<<R::Table as Table>::AllColumns>, R::Table, DB>:
        FilterDsl<<T::PrimaryKey as EqAll<<&'static I as Identifiable>::Id>>::Output>,
    <T::PrimaryKey as EqAll<<&'static I as Identifiable>::Id>>::Output:
    AppearsOnTable<T> + NonAggregate + QueryFragment<DB>,
    DB: HasSqlType<SqlTypeOfPlaceholder<R::FieldList, DB, R::PrimaryKeyIndex, R::Table>>
{
    type Delete = I;

    #[cfg_attr(feature = "clippy", allow(print_stdout))]
    fn handle_delete(
        executor: &Executor<Ctx, WundergraphScalarValue>,
        to_delete: &Self::Delete,
    ) -> ExecutionResult<WundergraphScalarValue> {
        let ctx = executor.context();
        let conn = ctx.get_connection();
        conn.transaction(|| -> ExecutionResult<WundergraphScalarValue> {
            // this is safe becuse we do not leek self out of this function
            let static_to_delete: &'static I = unsafe { &*(to_delete as *const I) };
            let filter = T::table().primary_key().eq_all(static_to_delete.id());
            let look_ahead = &executor.look_ahead();
            let query = R::build_query(&look_ahead)?;
            // We use identifiable so there should only be one element affected by this query
            let to_delete = LimitDsl::limit(FilterDsl::filter(query, filter), 1);
            let r = R::load(&look_ahead, conn, to_delete)?;
            let d = ::diesel::delete(FilterDsl::filter(T::table(), filter));
            if cfg!(feature = "debug") {
                debug!("{}", ::diesel::debug_query(&d));
            }
            assert_eq!(1, d.execute(conn)?);
            Ok(r.into_iter().next().unwrap_or(Value::Null))
        })
    }
}
