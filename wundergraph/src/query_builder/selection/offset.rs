use crate::error::WundergraphError;
use crate::juniper_ext::FromLookAheadValue;
use crate::query_builder::selection::{BoxedQuery, LoadingHandler};
use crate::scalar::WundergraphScalarValue;
use diesel::backend::Backend;
#[cfg(feature = "sqlite")]
use diesel::query_dsl::methods::LimitDsl;
use diesel::query_dsl::methods::OffsetDsl;
use failure::Error;
use juniper::LookAheadSelection;

pub trait ApplyOffset: Backend {
    fn apply_offset<'a, L, Ctx>(
        query: BoxedQuery<'a, L, Self, Ctx>,
        select: &LookAheadSelection<'_, WundergraphScalarValue>,
    ) -> Result<BoxedQuery<'a, L, Self, Ctx>, Error>
    where
        L: LoadingHandler<Self, Ctx>;
}

#[cfg(feature = "postgres")]
impl ApplyOffset for diesel::pg::Pg {
    fn apply_offset<'a, L, Ctx>(
        query: BoxedQuery<'a, L, Self, Ctx>,
        select: &LookAheadSelection<'_, WundergraphScalarValue>,
    ) -> Result<BoxedQuery<'a, L, Self, Ctx>, Error>
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
    ) -> Result<BoxedQuery<'a, L, Self, Ctx>, Error>
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
