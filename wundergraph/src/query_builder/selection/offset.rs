use crate::error::Result;
#[cfg(any(feature = "postgres", feature = "sqlite", feature = "mysql"))]
use crate::error::WundergraphError;
#[cfg(any(feature = "postgres", feature = "sqlite", feature = "mysql"))]
use crate::juniper_ext::FromLookAheadValue;
use crate::query_builder::selection::{BoxedQuery, LoadingHandler};
use crate::scalar::WundergraphScalarValue;
use diesel::backend::Backend;
#[cfg(any(feature = "sqlite", feature = "mysql"))]
use diesel::query_dsl::methods::LimitDsl;
#[cfg(any(feature = "postgres", feature = "sqlite", feature = "mysql"))]
use diesel::query_dsl::methods::OffsetDsl;

use juniper::LookAheadSelection;

/// A trait abstracting over the different behaviour of limit/offset
/// clauses in different database systems
pub trait ApplyOffset: Backend {
    /// Add a offset clause to the given query if requested
    fn apply_offset<'a, L, Ctx>(
        query: BoxedQuery<'a, L, Self, Ctx>,
        select: &LookAheadSelection<'_, WundergraphScalarValue>,
    ) -> Result<BoxedQuery<'a, L, Self, Ctx>>
    where
        L: LoadingHandler<Self, Ctx>;
}

#[cfg(feature = "postgres")]
impl ApplyOffset for diesel::pg::Pg {
    fn apply_offset<'a, L, Ctx>(
        query: BoxedQuery<'a, L, Self, Ctx>,
        select: &LookAheadSelection<'_, WundergraphScalarValue>,
    ) -> Result<BoxedQuery<'a, L, Self, Ctx>>
    where
        L: LoadingHandler<Self, Ctx>,
    {
        use juniper::LookAheadMethods;
        if let Some(offset) = select.argument("offset") {
            Ok(<_ as OffsetDsl>::offset(
                query,
                i64::from_look_ahead(offset.value())
                    .ok_or(WundergraphError::CouldNotBuildFilterArgument)?,
            ))
        } else {
            Ok(query)
        }
    }
}

#[cfg(feature = "sqlite")]
impl ApplyOffset for diesel::sqlite::Sqlite {
    fn apply_offset<'a, L, Ctx>(
        query: BoxedQuery<'a, L, Self, Ctx>,
        select: &LookAheadSelection<'_, WundergraphScalarValue>,
    ) -> Result<BoxedQuery<'a, L, Self, Ctx>>
    where
        L: LoadingHandler<Self, Ctx>,
    {
        use juniper::LookAheadMethods;
        if let Some(offset) = select.argument("offset") {
            let q = <_ as OffsetDsl>::offset(
                query,
                i64::from_look_ahead(offset.value())
                    .ok_or(WundergraphError::CouldNotBuildFilterArgument)?,
            );
            if select.argument("limit").is_some() {
                Ok(q)
            } else {
                Ok(<_ as LimitDsl>::limit(q, -1))
            }
        } else {
            Ok(query)
        }
    }
}

#[cfg(feature = "mysql")]
impl ApplyOffset for diesel::mysql::Mysql {
    fn apply_offset<'a, L, Ctx>(
        query: BoxedQuery<'a, L, Self, Ctx>,
        select: &LookAheadSelection<'_, WundergraphScalarValue>,
    ) -> Result<BoxedQuery<'a, L, Self, Ctx>>
    where
        L: LoadingHandler<Self, Ctx>,
    {
        use juniper::LookAheadMethods;
        if let Some(offset) = select.argument("offset") {
            let q = <_ as OffsetDsl>::offset(
                query,
                i64::from_look_ahead(offset.value())
                    .ok_or(WundergraphError::CouldNotBuildFilterArgument)?,
            );
            if select.argument("limit").is_some() {
                Ok(q)
            } else {
                Ok(<_ as LimitDsl>::limit(q, std::i64::MAX))
            }
        } else {
            Ok(query)
        }
    }
}
