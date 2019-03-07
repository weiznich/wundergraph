use diesel::backend::Backend;
use diesel::query_builder::QueryFragment;
use diesel::QuerySource;
use juniper::{meta, FromInputValue, GraphQLType, Registry};
use crate::query_helper::placeholder::FieldListExtractor;
use crate::scalar::WundergraphScalarValue;
use std::marker::PhantomData;
use crate::LoadingHandler;

#[derive(Debug)]
pub struct GraphqlWrapper<T, DB>(T, PhantomData<DB>);

#[derive(Debug)]
pub struct GraphqlOrderWrapper<T, DB>(PhantomData<(T,DB)>);

impl<T, DB> GraphQLType<WundergraphScalarValue> for GraphqlWrapper<T, DB>
where
    DB: Backend + 'static,
    T::Table: 'static,
    <T::Table as QuerySource>::FromClause: QueryFragment<DB>,
    T: LoadingHandler<DB>,
    T::FieldList: WundergraphGraphqlHelper<DB>,
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
        <T::FieldList as WundergraphGraphqlHelper<DB>>::object_meta::<Self>(
            T::FIELD_NAMES,
            registry,
        )
    }
}

#[derive(Debug)]
pub struct OrderTypeInfo<L, DB>(String, PhantomData<(L, DB)>);

impl<L, DB> Default for OrderTypeInfo<L, DB>
where
    DB: Backend + 'static,
    L::Table: 'static,
    <L::Table as QuerySource>::FromClause: QueryFragment<DB>,
    L: LoadingHandler<DB>,
    DB::QueryBuilder: Default,
{
    fn default() -> Self {
        OrderTypeInfo(format!("{}Columns", L::TYPE_NAME), PhantomData)
    }
}

impl<T, DB> GraphQLType<WundergraphScalarValue> for GraphqlOrderWrapper<T, DB>
where
    DB: Backend + 'static,
    T::Table: 'static,
    <T::Table as QuerySource>::FromClause: QueryFragment<DB>,
    T: LoadingHandler<DB>,
    T::FieldList: FieldListExtractor,
    <T::FieldList as FieldListExtractor>::Out: WundergraphGraphqlHelper<DB>,
    DB::QueryBuilder: Default,
{
    type Context = ();
    type TypeInfo = OrderTypeInfo<T, DB>;

    fn name(info: &Self::TypeInfo) -> Option<&str> {
        Some(&info.0)
    }

    fn meta<'r>(
        info: &Self::TypeInfo,
        registry: &mut Registry<'r, WundergraphScalarValue>,
    ) -> meta::MetaType<'r, WundergraphScalarValue>
    where
        WundergraphScalarValue: 'r,
    {
        use crate::query_helper::placeholder::WundergraphFieldList;

        <<T::FieldList as FieldListExtractor>::Out as WundergraphGraphqlHelper<DB>>::order_meta::<
            Self,
            _,
        >(
            info,
            |index| {
                T::FieldList::map_table_field(index, |index| T::FIELD_NAMES[index])
                    .expect("Field is there")
            },
            registry,
        )
    }
}

impl<T, DB> FromInputValue<WundergraphScalarValue> for GraphqlOrderWrapper<T, DB> {
    fn from_input_value(_: &juniper::InputValue<WundergraphScalarValue>) -> Option<Self> {
        Some(Self(PhantomData))
    }
}

pub trait WundergraphGraphqlMapper<DB> {
    type GraphQLType: GraphQLType<WundergraphScalarValue, TypeInfo = ()>;
}

impl<T, DB> WundergraphGraphqlMapper<DB> for T
where
    T: GraphQLType<WundergraphScalarValue, TypeInfo = ()>,
{
    type GraphQLType = T;
}

pub trait WundergraphGraphqlHelper<DB> {
    fn object_meta<'r, T>(
        names: &[&str],
        registry: &mut Registry<'r, WundergraphScalarValue>,
    ) -> meta::MetaType<'r, WundergraphScalarValue>
    where
        T: GraphQLType<WundergraphScalarValue, TypeInfo = ()>;

    fn order_meta<'r, T, F>(
        info: &T::TypeInfo,
        name: F,
        registry: &mut Registry<'r, WundergraphScalarValue>,
    ) -> meta::MetaType<'r, WundergraphScalarValue>
    where
        T: GraphQLType<WundergraphScalarValue> + FromInputValue<WundergraphScalarValue>,
        F: Fn(usize) -> &'static str;
}

macro_rules! wundergraph_graphql_helper_impl {
    ($(
        $Tuple:tt {
            $(($idx:tt) -> $T:ident, $ST: ident, $TT: ident,) +
        }
    )+) => {
        $(
            impl<$($T,)* Backend> WundergraphGraphqlHelper<Backend> for ($($T,)*)
            where $($T: WundergraphGraphqlMapper<Backend>,)* {
                fn object_meta<'r, Type>(
                    names: &[&str],
                    registry: &mut Registry<'r, WundergraphScalarValue>,
                ) -> meta::MetaType<'r, WundergraphScalarValue>
                    where Type: GraphQLType<WundergraphScalarValue, TypeInfo = ()>
                {
                    // TODO: get docs and deprecated from somewhere!!
                    let fields  = [
                        $(
                            registry.field::<<$T as WundergraphGraphqlMapper<Backend>>::GraphQLType>(names[$idx], &()),
                        )*
                    ];
                    let ty = registry.build_object_type::<Type>(
                        &(),
                        &fields,
                    );
                    meta::MetaType::Object(ty)
                }

                fn order_meta<'r, Type, Fun>(
                    info: &Type::TypeInfo,
                    names: Fun,
                    registry: &mut Registry<'r, WundergraphScalarValue>,
                ) -> meta::MetaType<'r, WundergraphScalarValue>
                where
                Type: GraphQLType<WundergraphScalarValue> + FromInputValue<WundergraphScalarValue>,
                Fun: Fn(usize) -> &'static str,
                {
                    use juniper::meta::EnumValue;
                    let values = [
                        $(
                            EnumValue::new(names($idx)),
                        )*
                    ];
                    let e = registry.build_enum_type::<Type>(
                        info,
                        &values,
                    );
                    meta::MetaType::Enum(e)
                }
            }
        )*
    };
}

__diesel_for_each_tuple!(wundergraph_graphql_helper_impl);
