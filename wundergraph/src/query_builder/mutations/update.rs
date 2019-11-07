use crate::context::WundergraphContext;
use crate::query_builder::selection::fields::WundergraphFieldList;
use crate::query_builder::selection::filter::build_filter::BuildFilter;
use crate::query_builder::selection::offset::ApplyOffset;
use crate::query_builder::selection::order::BuildOrder;
use crate::query_builder::selection::query_modifier::QueryModifier;
use crate::query_builder::selection::select::BuildSelect;
use crate::query_builder::selection::{LoadingHandler, SqlTypeOfPlaceholder};
use crate::scalar::WundergraphScalarValue;
use diesel::associations::{HasTable, Identifiable};
use diesel::backend::Backend;
use diesel::dsl::{Find, SqlTypeOf};
use diesel::expression::NonAggregate;
use diesel::query_builder::BoxedSelectStatement;
use diesel::query_builder::{AsChangeset, IntoUpdateTarget, QueryFragment};
use diesel::query_dsl::methods::{BoxedDsl, FilterDsl, FindDsl, LimitDsl};
use diesel::sql_types::HasSqlType;
use diesel::{AppearsOnTable, Connection, EqAll, QuerySource, RunQueryDsl, Table};
use juniper::{Arguments, ExecutionResult, Executor, FieldError, FromInputValue, Selection, Value};

#[doc(hidden)]
pub fn handle_update<DB, U, R, Ctx>(
    selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
    executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    arguments: &Arguments<'_, WundergraphScalarValue>,
    field_name: &'static str,
) -> ExecutionResult<WundergraphScalarValue>
where
    R: LoadingHandler<DB, Ctx>,
    R::Table: HandleUpdate<R, U, DB, Ctx> + 'static,
    DB: Backend + ApplyOffset + 'static,
    DB::QueryBuilder: Default,
    R::Columns: BuildOrder<R::Table, DB>
        + BuildSelect<
            R::Table,
            DB,
            SqlTypeOfPlaceholder<R::FieldList, DB, R::PrimaryKeyIndex, R::Table, Ctx>,
        >,
    <R::Table as QuerySource>::FromClause: QueryFragment<DB>,
    U: FromInputValue<WundergraphScalarValue>,
{
    if let Some(n) = arguments.get::<U>(field_name) {
        <R::Table as HandleUpdate<_, _, _, _>>::handle_update(selection, executor, &n)
    } else {
        let msg = format!("Missing argument {:?}", field_name);
        Err(FieldError::new(&msg, Value::Null))
    }
}

/// A trait to handle update mutations for database entities
///
/// Type parameters:
/// * `Self`: database table type for diesel
/// * `U`: data to update into the table. This type should contain information
///    which entry should be updated (by containing the primary key) and the data
///    that should be actually updated
/// * `DB`: Backend type from diesel, so one of `Pg` or `Sqlite`
/// * `Ctx`: The used wundergraph context type
///
/// A default implementation is provided for all types implementing
/// `diesel::AsChangeset`
pub trait HandleUpdate<L, U, DB, Ctx> {
    /// Actual function called to insert a database entity
    fn handle_update(
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        executor: &Executor<Ctx, WundergraphScalarValue>,
        update: &U,
    ) -> ExecutionResult<WundergraphScalarValue>;
}

// We use the 'static static lifetime here because otherwise rustc will
// tell us that it could not find a applying lifetime (caused by broken projection
// on higher ranked lifetime bounds)
impl<L, U, DB, Ctx, T> HandleUpdate<L, U, DB, Ctx> for T
where
    T: Table + HasTable<Table = T> + FindDsl<<&'static U as Identifiable>::Id> + 'static,
    DB: Backend + ApplyOffset + 'static,
    DB::QueryBuilder: Default,
    T::FromClause: QueryFragment<DB>,
    L: LoadingHandler<DB, Ctx, Table = T>,
    L::Columns: BuildOrder<T, DB>
        + BuildSelect<
            T,
            DB,
            SqlTypeOfPlaceholder<L::FieldList, DB, L::PrimaryKeyIndex, T, Ctx>,
        >,
    Ctx: WundergraphContext + QueryModifier<L, DB>,
    Ctx::Connection: Connection<Backend = DB>,
    L::FieldList: WundergraphFieldList<DB, L::PrimaryKeyIndex, T, Ctx>,
    T: BoxedDsl<
        'static,
        DB,
        Output = BoxedSelectStatement<'static, SqlTypeOf<<T as Table>::AllColumns>, T, DB>,
    >,
    <L::Filter as BuildFilter<DB>>::Ret: AppearsOnTable<T>,
    U: 'static,
    &'static U: AsChangeset<Target = T> + Identifiable + HasTable<Table = T>,
    Find<T, <&'static U as Identifiable>::Id>: IntoUpdateTarget<Table = T>,
    <Find<T, <&'static U as Identifiable>::Id> as IntoUpdateTarget>::WhereClause: QueryFragment<DB>,
    <&'static U as AsChangeset>::Changeset: QueryFragment<DB>,
    T::PrimaryKey: EqAll<<&'static U as Identifiable>::Id> + Default,
    DB: HasSqlType<SqlTypeOfPlaceholder<L::FieldList, DB, L::PrimaryKeyIndex, T, Ctx>>,
    <T::PrimaryKey as EqAll<<&'static U as Identifiable>::Id>>::Output:
        AppearsOnTable<T> + NonAggregate + QueryFragment<DB>,
{
    fn handle_update(
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
        change_set: &U,
    ) -> ExecutionResult<WundergraphScalarValue> {
        let ctx = executor.context();
        let conn = ctx.get_connection();
        conn.transaction(|| -> ExecutionResult<WundergraphScalarValue> {
            let look_ahead = executor.look_ahead();
            // this is safe becuse we do not leak change_set out of this function
            // this is required because otherwise rustc fails to project the temporary
            // lifetime
            let change_set: &'static U = unsafe { &*(change_set as *const U) };
            let u = ::diesel::update(change_set).set(change_set);
            #[cfg(feature = "debug")]
            {
                log::debug!("{}", ::diesel::debug_query(&u));
            }
            u.execute(conn)?;
            let f = FilterDsl::filter(
                L::build_query(&[], &look_ahead)?,
                T::PrimaryKey::default().eq_all(change_set.id()),
            );
            // We use identifiable so there should only be one element affected by this query
            let q = LimitDsl::limit(f, 1);
            let items = L::load(&look_ahead, selection, executor, q)?;
            Ok(items.into_iter().next().unwrap_or(Value::Null))
        })
    }
}
