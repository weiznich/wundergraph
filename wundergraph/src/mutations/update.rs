use diesel::associations::{HasTable, Identifiable};
use diesel::backend::Backend;
use diesel::dsl::{Find, SqlTypeOf};
use diesel::expression::NonAggregate;
use diesel::query_builder::BoxedSelectStatement;
use diesel::query_builder::{AsChangeset, IntoUpdateTarget, QueryFragment};
use diesel::query_dsl::methods::{BoxedDsl, FilterDsl, FindDsl, LimitDsl};
use diesel::sql_types::HasSqlType;
use diesel::{AppearsOnTable, Connection, EqAll, Queryable, RunQueryDsl, Table};

use juniper::{
    Arguments, ExecutionResult, Executor, FieldError, FromInputValue, GraphQLType, Value,
};

use filter::build_filter::BuildFilter;
use query_helper::order::BuildOrder;
use query_helper::placeholder::{SqlTypeOfPlaceholder, WundergraphFieldList};
use query_helper::select::BuildSelect;
use query_helper::tuple::TupleIndex;
use scalar::WundergraphScalarValue;
use LoadingHandler;
use WundergraphContext;

pub fn handle_update<DB, U, R, Ctx>(
    executor: &Executor<Ctx, WundergraphScalarValue>,
    arguments: &Arguments<WundergraphScalarValue>,
    field_name: &'static str,
) -> ExecutionResult<WundergraphScalarValue>
where
    DB: Backend,
    Ctx: WundergraphContext<DB>,
    U: UpdateHelper<DB, R, Ctx> + FromInputValue<WundergraphScalarValue>,
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
    fn handle_update(
        executor: &Executor<Ctx, WundergraphScalarValue>,
        update: &Self::Update,
    ) -> ExecutionResult<WundergraphScalarValue>;
}

pub trait UpdateHelper<DB, R, Ctx> {
    type Handler: HandleUpdate<DB, R, Ctx>;
}

#[doc(hidden)]
#[derive(Debug)]
pub struct AsChangeSetWrapper<U>(U);

#[cfg_attr(feature = "cargo-clippy", allow(use_self))]
impl<U, DB, R, Ctx, T> UpdateHelper<DB, R, Ctx> for U
where
    U: 'static,
    DB: Backend + 'static,
    &'static U: AsChangeset<Target = T> + HasTable<Table = T> + Identifiable,
    AsChangeSetWrapper<U>: HandleUpdate<DB, R, Ctx>,

    T: Table + HasTable<Table = T> + FindDsl<<&'static U as Identifiable>::Id> + 'static,
    Ctx: WundergraphContext<DB>,
    Find<T, <&'static U as Identifiable>::Id>: IntoUpdateTarget<Table = T>,
    R: LoadingHandler<DB, Table = T>
        + GraphQLType<WundergraphScalarValue, TypeInfo = (), Context = ()>,
    T::FromClause: QueryFragment<DB>,
    T::PrimaryKey: EqAll<<&'static U as Identifiable>::Id>,
    R::Columns: BuildOrder<T, DB>
        + BuildSelect<
            T,
            DB,
            SqlTypeOfPlaceholder<R::FieldList, DB, R::PrimaryKeyIndex, R::Table>,
    >,
    DB::QueryBuilder: Default,
    R::FieldList: WundergraphFieldList<DB, R::PrimaryKeyIndex, T>
        + TupleIndex<R::PrimaryKeyIndex>,
    <R::FieldList as WundergraphFieldList<DB, R::PrimaryKeyIndex, T>>::PlaceHolder:
        Queryable<SqlTypeOfPlaceholder<R::FieldList, DB, R::PrimaryKeyIndex, R::Table>, DB>,
{
    type Handler = AsChangeSetWrapper<U>;
}

// We use the 'static static lifetime here because otherwise rustc will
// tell us that it could not find a applying lifetime (caused by broken projection
// on higher ranked lifetime bounds)
impl<DB, R, Ctx, T, U> HandleUpdate<DB, R, Ctx> for AsChangeSetWrapper<U>
where
    U: 'static,
    DB: Backend + 'static,
    &'static U: AsChangeset<Target = T> + HasTable<Table = T> + Identifiable,
    T: Table + HasTable<Table = T> + FindDsl<<&'static U as Identifiable>::Id> + 'static,
    Ctx: WundergraphContext<DB>,
    Find<T, <&'static U as Identifiable>::Id>: IntoUpdateTarget<Table = T>,
    R: LoadingHandler<DB, Table = T>
        + GraphQLType<WundergraphScalarValue, TypeInfo = (), Context = ()>,
    T::FromClause: QueryFragment<DB>,
    <Find<T, <&'static U as Identifiable>::Id> as IntoUpdateTarget>::WhereClause: QueryFragment<DB>,
    <&'static U as AsChangeset>::Changeset: QueryFragment<DB>,
    DB::QueryBuilder: Default,
    T::PrimaryKey: EqAll<<&'static U as Identifiable>::Id>,
    R::Columns: BuildOrder<T, DB>
        + BuildSelect<
            T,
            DB,
            SqlTypeOfPlaceholder<R::FieldList, DB, R::PrimaryKeyIndex, R::Table>,
        >,
    R::FieldList: WundergraphFieldList<DB, R::PrimaryKeyIndex, T>
        + TupleIndex<R::PrimaryKeyIndex>,
    <R::FieldList as WundergraphFieldList<DB, R::PrimaryKeyIndex, T>>::PlaceHolder:
        Queryable<SqlTypeOfPlaceholder<R::FieldList, DB, R::PrimaryKeyIndex, R::Table>, DB>,
    for<'a> R::Table: BoxedDsl<
        'a,
        DB,
        Output = BoxedSelectStatement<'a, SqlTypeOf<<R::Table as Table>::AllColumns>, R::Table, DB>,
    >,
    <R::Filter as BuildFilter<DB>>::Ret: AppearsOnTable<T>,
    DB: HasSqlType<SqlTypeOfPlaceholder<R::FieldList, DB, R::PrimaryKeyIndex, R::Table>>,
    <T::PrimaryKey as EqAll<<&'static U as Identifiable>::Id>>::Output:
        AppearsOnTable<T> + NonAggregate + QueryFragment<DB>,
{
    type Update = U;

    fn handle_update(
        executor: &Executor<Ctx, WundergraphScalarValue>,
        change_set: &Self::Update,
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
            if cfg!(feature = "debug") {
                debug!("{}", ::diesel::debug_query(&u));
            }
            u.execute(conn)?;
            let f = FilterDsl::filter(
                R::build_query(&look_ahead)?,
                T::table().primary_key().eq_all(change_set.id()),
            );
            // We use identifiable so there should only be one element affected by this query
            let q = LimitDsl::limit(f, 1);
            let items = R::load(&look_ahead, conn, q)?;
            Ok(items.into_iter().next().unwrap_or(Value::Null))
        })
    }
}
