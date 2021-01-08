use super::offset::ApplyOffset;
use super::LoadingHandler;
use crate::error::WundergraphError;
use crate::juniper_ext::FromLookAheadValue;
use crate::query_builder::selection::fields::FieldListExtractor;
use crate::scalar::WundergraphScalarValue;
use crate::{diesel_ext::MultipleColumnHelper, error::Result};
use diesel::query_builder::QueryFragment;
use diesel::{backend::Backend, ExpressionMethods};
use diesel::{expression::NonAggregate, Column};
use diesel::{BoxableExpression, QuerySource, SelectableExpression};
use juniper::{
    meta, FromInputValue, GraphQLEnum, GraphQLType, LookAheadValue, Registry, ToInputValue,
};
use std::marker::PhantomData;

/// Build a order clause out of a given GraphQL request
pub trait BuildOrder<T, DB> {
    /// Uses the given order argument to build a valid order
    /// clause for the wundergraph entity `T`
    fn build_order(
        order: &[LookAheadValue<'_, WundergraphScalarValue>],
        field_name: impl Fn(usize) -> &'static str,
    ) -> Result<Vec<Box<dyn BoxableExpression<T, DB, SqlType = ()>>>>;
}

/// Defines how to order the result of an query
#[derive(Debug, GraphQLEnum, Copy, Clone, PartialEq)]
pub enum Order {
    /// Order elements in ascending order
    Asc,
    /// Order elements in descending order
    Desc,
}

#[derive(Debug)]
pub struct OrderBy<L, DB, Ctx>(PhantomData<(L, DB, Ctx)>);

#[doc(hidden)]
#[derive(Debug)]
pub struct OrderByTypeInfo<L, DB, Ctx>(String, PhantomData<(L, DB, Ctx)>);

#[doc(hidden)]
#[derive(Debug)]
pub struct GraphqlOrderWrapper<T, DB, Ctx>(PhantomData<(T, DB, Ctx)>);

#[doc(hidden)]
#[derive(Debug)]
pub struct OrderTypeInfo<L, DB, Ctx>(String, PhantomData<(L, DB, Ctx)>);

impl FromLookAheadValue for Order {
    fn from_look_ahead(v: &LookAheadValue<'_, WundergraphScalarValue>) -> Option<Self> {
        if let LookAheadValue::Enum(e) = *v {
            match e {
                "ASC" => Some(Order::Asc),
                "DESC" => Some(Order::Desc),
                _ => None,
            }
        } else {
            None
        }
    }
}

impl<L, DB, Ctx> Default for OrderByTypeInfo<L, DB, Ctx>
where
    DB: Backend + ApplyOffset + 'static,
    L::Table: 'static,
    <L::Table as QuerySource>::FromClause: QueryFragment<DB>,
    L: LoadingHandler<DB, Ctx>,
    DB::QueryBuilder: Default,
{
    fn default() -> Self {
        Self(format!("{}OrderBy", L::TYPE_NAME), PhantomData)
    }
}

impl<T, DB, Ctx> FromInputValue<WundergraphScalarValue> for OrderBy<T, DB, Ctx> {
    fn from_input_value(_: &juniper::InputValue<WundergraphScalarValue>) -> Option<Self> {
        Some(Self(PhantomData))
    }
}

impl<T, DB, Ctx> ToInputValue<WundergraphScalarValue> for OrderBy<T, DB, Ctx> {
    fn to_input_value(&self) -> juniper::InputValue<WundergraphScalarValue> {
        juniper::InputValue::Null
    }
}

impl<T, DB, Ctx> GraphQLType<WundergraphScalarValue> for OrderBy<T, DB, Ctx>
where
    DB: Backend + ApplyOffset + 'static,
    T::Table: 'static,
    <T::Table as QuerySource>::FromClause: QueryFragment<DB>,
    T: LoadingHandler<DB, Ctx>,
    DB::QueryBuilder: Default,
    GraphqlOrderWrapper<T, DB, Ctx>: GraphQLType<WundergraphScalarValue>,
    <GraphqlOrderWrapper<T, DB, Ctx> as GraphQLType<WundergraphScalarValue>>::TypeInfo: Default,
{
    type Context = ();
    type TypeInfo = OrderByTypeInfo<T, DB, Ctx>;

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
        let args = &[
            registry.arg::<GraphqlOrderWrapper<T, DB, Ctx>>("column", &Default::default()),
            registry.arg_with_default("direction", &Order::Asc, &()),
        ];

        let obj = registry.build_input_object_type::<Self>(info, args);
        meta::MetaType::InputObject(obj)
    }
}

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
    <T::FieldList as FieldListExtractor>::Out: WundergraphGraphqlOrderHelper<T, DB, Ctx>,
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
        use crate::query_builder::selection::fields::WundergraphFieldList;

        <<T::FieldList as FieldListExtractor>::Out as WundergraphGraphqlOrderHelper<T, DB, Ctx>>::order_meta::<
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

#[doc(hidden)]
pub trait WundergraphGraphqlOrderHelper<L, DB, Ctx> {
    fn order_meta<'r, T, F>(
        info: &T::TypeInfo,
        name: F,
        registry: &mut Registry<'r, WundergraphScalarValue>,
    ) -> meta::MetaType<'r, WundergraphScalarValue>
    where
        T: GraphQLType<WundergraphScalarValue> + FromInputValue<WundergraphScalarValue>,
        F: Fn(usize) -> &'static str;
}

pub trait AsOrderExpression<T, DB> {
    fn as_order_expression(order: Order) -> Box<dyn BoxableExpression<T, DB, SqlType = ()>>;
}

impl<T, DB, C> AsOrderExpression<T, DB> for C
where
    DB: Backend,
    C: ExpressionMethods
        + Column
        + Default
        + QueryFragment<DB>
        + SelectableExpression<T>
        + NonAggregate
        + 'static,
{
    fn as_order_expression(order: Order) -> Box<dyn BoxableExpression<T, DB, SqlType = ()>> {
        if order == Order::Desc {
            Box::new(C::default().desc()) as Box<dyn BoxableExpression<T, DB, SqlType = ()>>
        } else {
            Box::new(C::default().asc()) as Box<_>
        }
    }
}

impl<T, DB, PK, V> AsOrderExpression<T, DB> for MultipleColumnHelper<(PK, V)>
where
    DB: Backend,
    PK: ExpressionMethods
        + Default
        + QueryFragment<DB>
        + SelectableExpression<T>
        + NonAggregate
        + 'static,
{
    fn as_order_expression(order: Order) -> Box<dyn BoxableExpression<T, DB, SqlType = ()>> {
        if order == Order::Desc {
            Box::new(PK::default().desc()) as Box<dyn BoxableExpression<T, DB, SqlType = ()>>
        } else {
            Box::new(PK::default().asc()) as Box<_>
        }
    }
}

macro_rules! impl_order_traits {
    ($(
        $Tuple:tt {
            $(($idx:tt) -> $T:ident, $ST: ident, $TT: ident,) +
        }
    )+) => {
        $(

            impl<Table, DB, $($T,)+> BuildOrder<Table, DB> for ($($T,)+)
            where Table: ::diesel::Table,
                  DB: Backend,
            $(
                $T: AsOrderExpression<Table, DB>,
            )+
            {
                fn build_order(
                    fields: &[LookAheadValue<'_, WundergraphScalarValue>],
                    field_name: impl Fn(usize) -> &'static str,
                ) -> Result<Vec<Box<dyn BoxableExpression<Table, DB, SqlType = ()>>>>
                {
                    let mut ret = Vec::with_capacity(fields.len());
                    for f in fields {
                        if let LookAheadValue::Object(o) = f {
                            let column = o.iter().find(|(k, _)| *k == "column")
                                .and_then(|(_, v)| if let LookAheadValue::Enum(c) = v {
                                    Some(c)
                                } else {
                                    None
                                })
                                .ok_or(WundergraphError::CouldNotBuildFilterArgument)?;
                            let order = o.iter().find(|(k, _)| *k == "direction")
                                .and_then(|(_, v)| Order::from_look_ahead(v))
                                .unwrap_or(Order::Asc);
                            match *column {
                            $(
                                x if x == field_name($idx) => ret.push($T::as_order_expression(order)),
                            )+
                                x => {
                                    return Err(WundergraphError::UnknownDatabaseField {
                                            name: x.to_owned()
                                        });
                                }
                            }
                        } else {
                            return Err(
                                WundergraphError::CouldNotBuildFilterArgument
                            );
                        }
                    }
                    Ok(ret)
                }
            }

            impl<$($T,)* Loading, Back, Ctx> WundergraphGraphqlOrderHelper<Loading, Back, Ctx> for ($($T,)*)
            where Back: Backend + ApplyOffset + 'static,
                  Loading::Table: 'static,
                  <Loading::Table as QuerySource>::FromClause: QueryFragment<Back>,
                  Loading: LoadingHandler<Back, Ctx>,
                  Back::QueryBuilder: Default,
            {

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

__diesel_for_each_tuple!(impl_order_traits);
