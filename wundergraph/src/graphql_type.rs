use crate::query_builder::selection::offset::ApplyOffset;
use crate::query_builder::selection::LoadingHandler;
use crate::scalar::WundergraphScalarValue;
use diesel::backend::Backend;
use diesel::query_builder::QueryFragment;
use diesel::QuerySource;
use juniper::{meta, GraphQLType, Registry};
use std::marker::PhantomData;

/// A helper type to automatically provide `juniper::GraphQLObject` implementation
/// for types that also implement `LoadingHandler`
#[derive(Debug)]
pub struct GraphqlWrapper<T, DB, Ctx>(T, PhantomData<(DB, Ctx)>);

impl<T, DB, Ctx> GraphQLType<WundergraphScalarValue> for GraphqlWrapper<T, DB, Ctx>
where
    DB: Backend + ApplyOffset + 'static,
    T::Table: 'static,
    <T::Table as QuerySource>::FromClause: QueryFragment<DB>,
    T: LoadingHandler<DB, Ctx>,
    T::FieldList: WundergraphGraphqlHelper<T, DB, Ctx>,
    DB::QueryBuilder: Default,
{
    type Context = ();
    type TypeInfo = ();

    fn name(_info: &Self::TypeInfo) -> Option<&str> {
        Some(T::TYPE_NAME)
    }

    fn meta<'r>(
        _info: &Self::TypeInfo,
        registry: &mut Registry<'r, WundergraphScalarValue>,
    ) -> meta::MetaType<'r, WundergraphScalarValue>
    where
        WundergraphScalarValue: 'r,
    {
        <T::FieldList as WundergraphGraphqlHelper<T, DB, Ctx>>::object_meta::<Self>(
            T::FIELD_NAMES,
            registry,
        )
    }
}

#[doc(hidden)]
pub trait WundergraphGraphqlMapper<DB, Ctx> {
    type GraphQLType: GraphQLType<WundergraphScalarValue, TypeInfo = ()>;

    fn register_arguments<'r>(
        _registry: &mut Registry<'r, WundergraphScalarValue>,
        field: meta::Field<'r, WundergraphScalarValue>,
    ) -> meta::Field<'r, WundergraphScalarValue> {
        field
    }
}

impl<T, DB, Ctx> WundergraphGraphqlMapper<DB, Ctx> for T
where
    T: GraphQLType<WundergraphScalarValue, TypeInfo = ()>,
{
    type GraphQLType = Self;
}

#[doc(hidden)]
pub trait WundergraphGraphqlHelper<L, DB, Ctx> {
    fn object_meta<'r, T>(
        names: &[&str],
        registry: &mut Registry<'r, WundergraphScalarValue>,
    ) -> meta::MetaType<'r, WundergraphScalarValue>
    where
        T: GraphQLType<WundergraphScalarValue, TypeInfo = ()>;
}

macro_rules! wundergraph_graphql_helper_impl {
    ($(
        $Tuple:tt {
            $(($idx:tt) -> $T:ident, $ST: ident, $TT: ident,) +
        }
    )+) => {
        $(
            impl<$($T,)* Loading, Back, Ctx> WundergraphGraphqlHelper<Loading, Back, Ctx> for ($($T,)*)
            where $($T: WundergraphGraphqlMapper<Back, Ctx>,)*
                  Back: Backend + ApplyOffset + 'static,
                  Loading::Table: 'static,
                  <Loading::Table as QuerySource>::FromClause: QueryFragment<Back>,
                  Loading: LoadingHandler<Back, Ctx>,
                  Back::QueryBuilder: Default,
            {
                fn object_meta<'r, Type>(
                    names: &[&str],
                    registry: &mut Registry<'r, WundergraphScalarValue>,
                ) -> meta::MetaType<'r, WundergraphScalarValue>
                    where Type: GraphQLType<WundergraphScalarValue, TypeInfo = ()>
                {
                    let fields  = [
                        $({
                            let mut field = registry.field::<<$T as WundergraphGraphqlMapper<Back, Ctx>>::GraphQLType>(names[$idx], &());
                            field = <$T as WundergraphGraphqlMapper<Back, Ctx>>::register_arguments(registry, field);
                            if let Some(doc) = Loading::field_description($idx) {
                                field = field.description(doc);
                            }
                            if let Some(deprecated) = Loading::field_deprecation($idx) {
                                field = field.deprecated(deprecated);
                            }
                            field
                        },)*
                    ];
                    let mut ty = registry.build_object_type::<Type>(
                        &(),
                        &fields,
                    );
                    if let Some(doc) = Loading::TYPE_DESCRIPTION {
                        ty = ty.description(doc);
                    }
                    meta::MetaType::Object(ty)
                }
            }
        )*
    };
}

__diesel_for_each_tuple!(wundergraph_graphql_helper_impl);
