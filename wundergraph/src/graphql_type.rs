use crate::query_helper::placeholder::FieldListExtractor;
use crate::scalar::WundergraphScalarValue;
use crate::{LoadingHandler, ApplyOffset};
use diesel::backend::Backend;
use diesel::query_builder::QueryFragment;
use diesel::QuerySource;
use juniper::{meta, FromInputValue, GraphQLType, Registry};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct GraphqlWrapper<T, DB, Ctx>(T, PhantomData<(DB, Ctx)>);

#[derive(Debug)]
pub struct GraphqlOrderWrapper<T, DB, Ctx>(PhantomData<(T, DB, Ctx)>);

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

#[derive(Debug)]
pub struct OrderTypeInfo<L, DB, Ctx>(String, PhantomData<(L, DB, Ctx)>);

impl<L, DB, Ctx> Default for OrderTypeInfo<L, DB, Ctx>
where
    DB: Backend + ApplyOffset + 'static,
    L::Table: 'static,
    <L::Table as QuerySource>::FromClause: QueryFragment<DB>,
    L: LoadingHandler<DB, Ctx>,
    DB::QueryBuilder: Default,
{
    fn default() -> Self {
        Self(format!("{}Columns", L::TYPE_NAME), PhantomData)
    }
}

impl<T, DB, Ctx> GraphQLType<WundergraphScalarValue> for GraphqlOrderWrapper<T, DB, Ctx>
where
    DB: Backend + ApplyOffset + 'static,
    T::Table: 'static,
    <T::Table as QuerySource>::FromClause: QueryFragment<DB>,
    T: LoadingHandler<DB, Ctx>,
    T::FieldList: FieldListExtractor,
    <T::FieldList as FieldListExtractor>::Out: WundergraphGraphqlHelper<T, DB, Ctx>,
    DB::QueryBuilder: Default,
{
    type Context = ();
    type TypeInfo = OrderTypeInfo<T, DB, Ctx>;

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

        <<T::FieldList as FieldListExtractor>::Out as WundergraphGraphqlHelper<T, DB, Ctx>>::order_meta::<
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

impl<T, DB, Ctx> FromInputValue<WundergraphScalarValue> for GraphqlOrderWrapper<T, DB, Ctx> {
    fn from_input_value(_: &juniper::InputValue<WundergraphScalarValue>) -> Option<Self> {
        Some(Self(PhantomData))
    }
}

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

pub trait WundergraphGraphqlHelper<L, DB, Ctx> {
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
                    if let Some(doc) = Loading::type_description() {
                        ty = ty.description(doc);
                    }
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
