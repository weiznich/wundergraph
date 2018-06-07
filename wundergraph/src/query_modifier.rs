use super::LoadingHandler;
use diesel::associations::HasTable;
use diesel::backend::Backend;
use diesel::query_builder::BoxedSelectStatement;
use failure::Error;
use juniper::LookAheadSelection;

pub trait BuildQueryModifier<T>: Sized {
    type Context;
    fn from_ctx(ctx: &Self::Context) -> Result<Self, Error>;
}

pub trait QueryModifier<DB: Backend> {
    type Entity: LoadingHandler<DB>;
    fn modify_query<'a>(
        &self,
        final_query: BoxedSelectStatement<
            'a,
            <Self::Entity as LoadingHandler<DB>>::SqlType,
            <Self::Entity as HasTable>::Table,
            DB,
        >,
        selection: &LookAheadSelection,
    ) -> Result<
        BoxedSelectStatement<
            'a,
            <Self::Entity as LoadingHandler<DB>>::SqlType,
            <Self::Entity as HasTable>::Table,
            DB,
        >,
        Error,
    >;
}

#[derive(Debug, Clone, Copy)]
pub struct DefaultModifier<C, T>(::std::marker::PhantomData<(T, C)>);

impl<T, C> BuildQueryModifier<T> for DefaultModifier<C, T> {
    type Context = C;
    fn from_ctx(_ctx: &Self::Context) -> Result<Self, Error> {
        Ok(DefaultModifier(Default::default()))
    }
}

impl<C, DB, T> QueryModifier<DB> for DefaultModifier<C, T>
where
    DB: Backend,
    T: LoadingHandler<DB>,
{
    type Entity = T;

    fn modify_query<'a>(
        &self,
        final_query: BoxedSelectStatement<
            'a,
            <Self::Entity as LoadingHandler<DB>>::SqlType,
            <Self::Entity as HasTable>::Table,
            DB,
        >,
        _selection: &LookAheadSelection,
    ) -> Result<
        BoxedSelectStatement<
            'a,
            <Self::Entity as LoadingHandler<DB>>::SqlType,
            <Self::Entity as HasTable>::Table,
            DB,
        >,
        Error,
    > {
        Ok(final_query)
    }
}
