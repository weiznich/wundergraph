use crate::query_builder::selection::offset::ApplyOffset;
use crate::scalar::WundergraphScalarValue;
use diesel::associations::HasTable;
use diesel::backend::Backend;
use diesel::query_builder::QueryFragment;
use diesel::QuerySource;
use juniper::{Arguments, ExecutionResult, Executor, FieldError, FromInputValue, Selection, Value};

#[cfg(feature = "postgres")]
mod pg;

#[cfg(feature = "sqlite")]
mod sqlite;

#[doc(hidden)]
pub fn handle_insert<DB, I, R, Ctx>(
    selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
    executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    arguments: &Arguments<'_, WundergraphScalarValue>,
    field_name: &'static str,
) -> ExecutionResult<WundergraphScalarValue>
where
    R: HasTable,
    R::Table: HandleInsert<R, I, DB, Ctx> + 'static,
    DB: Backend + ApplyOffset + 'static,
    DB::QueryBuilder: Default,
    <R::Table as QuerySource>::FromClause: QueryFragment<DB>,
    I: FromInputValue<WundergraphScalarValue>,
{
    if let Some(n) = arguments.get::<I>(field_name) {
        <R::Table as HandleInsert<_, _, _, _>>::handle_insert(selection, executor, n)
    } else {
        let msg = format!("Missing argument {}", field_name);
        Err(FieldError::new(&msg, Value::Null))
    }
}

#[doc(hidden)]
pub fn handle_batch_insert<DB, I, R, Ctx>(
    selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
    executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    arguments: &Arguments<'_, WundergraphScalarValue>,
    field_name: &'static str,
) -> ExecutionResult<WundergraphScalarValue>
where
    R: HasTable,
    R::Table: HandleBatchInsert<R, I, DB, Ctx> + 'static,
    DB: Backend + ApplyOffset + 'static,
    DB::QueryBuilder: Default,
    <R::Table as QuerySource>::FromClause: QueryFragment<DB>,
    I: FromInputValue<WundergraphScalarValue>,
{
    if let Some(n) = arguments.get::<Vec<I>>(field_name) {
        <R::Table as HandleBatchInsert<_, _, _, _>>::handle_batch_insert(selection, executor, n)
    } else {
        let msg = format!("Missing argument {}", field_name);
        Err(FieldError::new(&msg, Value::Null))
    }
}

/// A trait to handle insert mutations for database entities
///
/// Type parameters:
/// * `Self`: database table type for diesel
/// * `I`: data to insert into the table
/// * `DB`: Backend type from diesel, so one of `Pg` or `Sqlite`
/// * `Ctx`: The used wundergraph context type
///
/// A default implementation is provided for all types implementing
/// `diesel::Insertable`
pub trait HandleInsert<L, I, DB, Ctx> {
    /// Actual function called to insert a database entity
    fn handle_insert(
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
        insertable: I,
    ) -> ExecutionResult<WundergraphScalarValue>;
}

/// A trait to handle batch insert mutations for database entities
///
/// Type parameters:
/// * `Self`: database table type for diesel
/// * `I`: data to insert into the table
/// * `DB`: Backend type from diesel, so one of `Pg` or `Sqlite`
/// * `Ctx`: The used wundergraph context type
///
/// A default implementation is provided for all types implementing
/// `diesel::Insertable`
pub trait HandleBatchInsert<L, I, DB, Ctx> {
    /// Actual function called to insert a batch of database entity
    fn handle_batch_insert(
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
        insertable: Vec<I>,
    ) -> ExecutionResult<WundergraphScalarValue>;
}
