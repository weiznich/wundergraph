use diesel::query_dsl::methods::BoxedDsl;

pub type BoxedQuery<'a, Q, DB> = <Q as BoxedDsl<'a, DB>>::Output;
