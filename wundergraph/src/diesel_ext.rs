//! A module containing extension traits for various diesel types

use std::marker::PhantomData;

use diesel::backend::Backend;
use diesel::expression::{AppearsOnTable, Expression, NonAggregate, SelectableExpression};
use diesel::query_builder::BindCollector;
use diesel::query_builder::QueryBuilder;
use diesel::query_builder::{AstPass, QueryFragment};
use diesel::result::QueryResult;
use diesel::sql_types;
use diesel::sql_types::IntoNullable;
use diesel::types::HasSqlType;
use diesel::types::TypeMetadata;

/// A helper trait used when boxing filters
///
/// In Rust you cannot create a trait object with more than one trait.
/// This type has all of the additional traits you would want when using
/// `Box<Expression>` as a single trait object. This type is comparable to
/// diesels `BoxableExpression`, but allows to use non select able expressions,
/// which is mainly useful for constructing filters.
///
/// This is typically used as the return type of a function or as associated
/// types in traits.
pub trait BoxableFilter<QS, DB>
where
    DB: Backend,
    Self: Expression,
    Self: AppearsOnTable<QS>,
    Self: NonAggregate,
    Self: QueryFragment<DB>,
{
}

impl<QS, T, DB> BoxableFilter<QS, DB> for T
where
    DB: Backend,
    T: Expression,
    T: AppearsOnTable<QS>,
    T: NonAggregate,
    T: QueryFragment<DB>,
{
}

/// A diesel helper type that indicates if null or some expression selected
#[derive(Debug)]
pub struct MaybeNull<T> {
    expr: PhantomData<T>,
    as_null: bool,
}

impl<T: Default> MaybeNull<T> {
    pub fn expr() -> Self {
        Self {
            expr: PhantomData,
            as_null: false,
        }
    }

    pub fn as_null() -> Self {
        Self {
            expr: PhantomData,
            as_null: true,
        }
    }
}

impl<T> Expression for MaybeNull<T>
where
    T: Expression,
    T::SqlType: IntoNullable,
{
    type SqlType = <T::SqlType as IntoNullable>::Nullable;
}

impl<T, DB> QueryFragment<DB> for MaybeNull<T>
where
    DB: Backend,
    T: QueryFragment<DB> + QueryFragment<FakeBackend<DB>> + Default,
    DB::QueryBuilder: Default,
{
    fn walk_ast(&self, mut pass: AstPass<'_, DB>) -> QueryResult<()> {
        let expr = T::default();
        if self.as_null {
            let mut query_builder = MaybeNullQueryBuilder::new(DB::QueryBuilder::default(), true);
            let ast_pass = AstPass::<FakeBackend<DB>>::to_sql(&mut query_builder);
            expr.walk_ast(ast_pass)?;
            let identifier_pushed = query_builder.identifier_pushed;
            debug_assert!(identifier_pushed % 2 == 0);

            for i in 0..(identifier_pushed / 2) {
                if i != 0 {
                    pass.push_sql(", ");
                }
                pass.push_sql("NULL");
            }
            pass.push_sql(" ");
        } else {
            expr.walk_ast(pass)?;
        }
        Ok(())
    }
}

impl<T> NonAggregate for MaybeNull<T> {}

impl<T, QS> AppearsOnTable<QS> for MaybeNull<T> where Self: Expression {}

impl<T, ST> SelectableExpression<T> for MaybeNull<ST> where Self: Expression {}

pub(crate) use self::fake_query_builder::FakeBackend;
use self::fake_query_builder::MaybeNullQueryBuilder;

mod fake_query_builder {
    use super::*;

    #[derive(Debug)]
    pub struct MaybeNullQueryBuilder<Q> {
        inner: Q,
        generate_nulls: bool,
        pub(super) identifier_pushed: usize,
    }

    impl<Q> MaybeNullQueryBuilder<Q> {
        pub fn new(inner: Q, generate_nulls: bool) -> Self {
            Self {
                inner,
                generate_nulls,
                identifier_pushed: 0,
            }
        }
    }

    impl<Q, DB> QueryBuilder<FakeBackend<DB>> for MaybeNullQueryBuilder<Q>
    where
        Q: QueryBuilder<DB>,
        DB: Backend,
    {
        fn push_sql(&mut self, sql: &str) {
            self.inner.push_sql(sql)
        }

        fn push_identifier(&mut self, identifier: &str) -> QueryResult<()> {
            if self.generate_nulls {
                self.identifier_pushed += 1;
                Ok(())
            } else {
                self.inner.push_identifier(identifier)
            }
        }

        fn push_bind_param(&mut self) {
            self.inner.push_bind_param();
        }

        fn finish(self) -> String {
            self.inner.finish()
        }
    }

    #[derive(Debug)]
    pub struct FakeBackend<DB>(DB);

    impl<DB> Backend for FakeBackend<DB>
    where
        DB: Backend,
    {
        type QueryBuilder = MaybeNullQueryBuilder<DB::QueryBuilder>;

        type BindCollector = FakeBindCollector<DB::BindCollector>;

        type RawValue = DB::RawValue;

        type ByteOrder = DB::ByteOrder;
    }

    #[derive(Debug)]
    pub struct FakeBindCollector<B>(B);

    impl<B, DB> BindCollector<FakeBackend<DB>> for FakeBindCollector<B>
    where
        B: BindCollector<DB>,
        DB: Backend,
    {
        fn push_bound_value<T, U>(
            &mut self,
            _bind: &U,
            _metadata_lookup: &<FakeBackend<DB> as TypeMetadata>::MetadataLookup,
        ) -> QueryResult<()>
        where
            FakeBackend<DB>: HasSqlType<T>,
            U: diesel::types::ToSql<T, FakeBackend<DB>>,
        {
            unimplemented!()
            //    self.0.push_bound_value(bind, metadata_lookup)
        }
    }

    impl<DB> TypeMetadata for FakeBackend<DB>
    where
        DB: TypeMetadata,
    {
        type TypeMetadata = DB::TypeMetadata;

        type MetadataLookup = DB::MetadataLookup;
    }

    impl<DB> HasSqlType<sql_types::Timestamp> for FakeBackend<DB>
    where
        DB: HasSqlType<sql_types::Timestamp>,
    {
        fn metadata(lookup: &Self::MetadataLookup) -> Self::TypeMetadata {
            DB::metadata(lookup)
        }
    }

    impl<DB> HasSqlType<sql_types::Date> for FakeBackend<DB>
    where
        DB: HasSqlType<sql_types::Date>,
    {
        fn metadata(lookup: &Self::MetadataLookup) -> Self::TypeMetadata {
            DB::metadata(lookup)
        }
    }

    impl<DB> HasSqlType<sql_types::Time> for FakeBackend<DB>
    where
        DB: HasSqlType<sql_types::Time>,
    {
        fn metadata(lookup: &Self::MetadataLookup) -> Self::TypeMetadata {
            DB::metadata(lookup)
        }
    }

    impl<DB> HasSqlType<sql_types::SmallInt> for FakeBackend<DB>
    where
        DB: HasSqlType<sql_types::SmallInt>,
    {
        fn metadata(lookup: &Self::MetadataLookup) -> Self::TypeMetadata {
            DB::metadata(lookup)
        }
    }

    impl<DB> HasSqlType<sql_types::Integer> for FakeBackend<DB>
    where
        DB: HasSqlType<sql_types::Integer>,
    {
        fn metadata(lookup: &Self::MetadataLookup) -> Self::TypeMetadata {
            DB::metadata(lookup)
        }
    }

    impl<DB> HasSqlType<sql_types::BigInt> for FakeBackend<DB>
    where
        DB: HasSqlType<sql_types::BigInt>,
    {
        fn metadata(lookup: &Self::MetadataLookup) -> Self::TypeMetadata {
            DB::metadata(lookup)
        }
    }

    impl<DB> HasSqlType<sql_types::Text> for FakeBackend<DB>
    where
        DB: HasSqlType<sql_types::Text>,
    {
        fn metadata(lookup: &Self::MetadataLookup) -> Self::TypeMetadata {
            DB::metadata(lookup)
        }
    }

    impl<DB> HasSqlType<sql_types::Bool> for FakeBackend<DB>
    where
        DB: HasSqlType<sql_types::Bool>,
    {
        fn metadata(lookup: &Self::MetadataLookup) -> Self::TypeMetadata {
            DB::metadata(lookup)
        }
    }

    impl<DB> HasSqlType<sql_types::Float> for FakeBackend<DB>
    where
        DB: HasSqlType<sql_types::Float>,
    {
        fn metadata(lookup: &Self::MetadataLookup) -> Self::TypeMetadata {
            DB::metadata(lookup)
        }
    }

    impl<DB> HasSqlType<sql_types::Double> for FakeBackend<DB>
    where
        DB: HasSqlType<sql_types::Double>,
    {
        fn metadata(lookup: &Self::MetadataLookup) -> Self::TypeMetadata {
            DB::metadata(lookup)
        }
    }

    impl<DB> HasSqlType<sql_types::Binary> for FakeBackend<DB>
    where
        DB: HasSqlType<sql_types::Binary>,
    {
        fn metadata(lookup: &Self::MetadataLookup) -> Self::TypeMetadata {
            DB::metadata(lookup)
        }
    }
}
