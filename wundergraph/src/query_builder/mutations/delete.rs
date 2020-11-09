use crate::context::WundergraphContext;
use crate::query_builder::selection::fields::WundergraphFieldList;
use crate::query_builder::selection::offset::ApplyOffset;
use crate::query_builder::selection::order::BuildOrder;
use crate::query_builder::selection::select::BuildSelect;
use crate::query_builder::selection::{LoadingHandler, SqlTypeOfPlaceholder};
use crate::scalar::WundergraphScalarValue;
use diesel::associations::HasTable;
use diesel::backend::Backend;
use diesel::dsl::Filter;
use diesel::query_builder::{IntoUpdateTarget, QueryFragment, QueryId};
use diesel::query_dsl::methods::FilterDsl;
use diesel::Identifiable;
use diesel::{Connection, EqAll, QuerySource, RunQueryDsl, Table};
use juniper::{
    Arguments, ExecutionResult, Executor, FieldError, FromInputValue, GraphQLObject, Value,
};

/// A struct representing the number of deleted entities
#[derive(Debug, GraphQLObject, Clone, Copy)]
#[graphql(scalar = WundergraphScalarValue)]
pub struct DeletedCount {
    /// Number of deleted entities
    pub count: i64,
}

#[doc(hidden)]
pub fn handle_delete<DB, D, R, Ctx>(
    executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    arguments: &Arguments<'_, WundergraphScalarValue>,
    field_name: &'static str,
) -> ExecutionResult<WundergraphScalarValue>
where
    R: HasTable,
    R::Table: HandleDelete<R, D, DB, Ctx> + 'static,
    DB: Backend + ApplyOffset + 'static,
    DB::QueryBuilder: Default,
    <R::Table as QuerySource>::FromClause: QueryFragment<DB>,
    D: FromInputValue<WundergraphScalarValue>,
{
    if let Some(n) = arguments.get::<D>(field_name) {
        <R::Table as HandleDelete<_, _, _, _>>::handle_delete(executor, &n)
    } else {
        let msg = format!("Missing argument {:?}", field_name);
        Err(FieldError::new(&msg, Value::Null))
    }
}

/// A trait to handle delete mutations for database entities
///
/// Type parameters:
/// * `Self`: database table type from diesel
/// * `L`: Struct implementing `LoadingHandler`
/// * `K`: Input type used determine which entities should be deleted.
///    Normally something representing the primary key of the table
/// * `DB`: Backend type from diesel, so one of `Pg` or `Sqlite`
/// * `Ctx`: The used wundergraph context type
pub trait HandleDelete<L, K, DB, Ctx> {
    /// Actual function called to delete a database entity
    fn handle_delete(
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
        to_delete: &K,
    ) -> ExecutionResult<WundergraphScalarValue>;
}

// We use the 'static static lifetime here because otherwise rustc will
// tell us that it could not find a applying lifetime (caused by broken projection
// on higher ranked lifetime bounds)
impl<L, K, DB, Ctx, T> HandleDelete<L, K, DB, Ctx> for T
where
    T: Table + HasTable<Table = T> + QueryId + 'static,
    DB: Backend + ApplyOffset + 'static,
    DB::QueryBuilder: Default,
    T::FromClause: QueryFragment<DB>,
    L: LoadingHandler<DB, Ctx, Table = T>,
    L::Columns: BuildOrder<T, DB>
        + BuildSelect<T, DB, SqlTypeOfPlaceholder<L::FieldList, DB, L::PrimaryKeyIndex, T, Ctx>>,
    Ctx: WundergraphContext,
    Ctx::Connection: Connection<Backend = DB>,
    L::FieldList: WundergraphFieldList<DB, L::PrimaryKeyIndex, T, Ctx>,
    K: 'static,
    &'static K: Identifiable<Table = T>,
    T::PrimaryKey: EqAll<<&'static K as Identifiable>::Id> + Default,
    T::Query: FilterDsl<<T::PrimaryKey as EqAll<<&'static K as Identifiable>::Id>>::Output>,
    Filter<T::Query, <T::PrimaryKey as EqAll<<&'static K as Identifiable>::Id>>::Output>: IntoUpdateTarget<Table = T>,
    <Filter<T::Query, <T::PrimaryKey as EqAll<<&'static K as Identifiable>::Id>>::Output> as IntoUpdateTarget>::WhereClause: QueryFragment<DB>
       + QueryId,
{
    fn handle_delete(
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
        to_delete: &K,
    ) -> ExecutionResult<WundergraphScalarValue> {
        let ctx = executor.context();
        let conn = ctx.get_connection();
        conn.transaction(|| -> ExecutionResult<WundergraphScalarValue> {
            // this is safe becuse we do not leak to_delete out of this function
            let static_to_delete: &'static K = unsafe { &*(to_delete as *const K) };
            let filter = T::PrimaryKey::default().eq_all(static_to_delete.id());
            let d = ::diesel::delete(FilterDsl::filter(Self::table(), filter));
            #[cfg(feature = "debug")]
            {
                log::debug!("{}", ::diesel::debug_query(&d));
            }

            executor.resolve_with_ctx(
                &(),
                &DeletedCount {
                    count: d.execute(conn)? as _,
                },
            )
        })
    }
}
