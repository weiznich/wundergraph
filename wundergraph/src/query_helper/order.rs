use crate::helper::FromLookAheadValue;
use crate::order::Order;
use crate::scalar::WundergraphScalarValue;
use crate::WundergraphError;
use diesel::backend::Backend;
use diesel::expression::NonAggregate;
use diesel::query_builder::QueryFragment;
use diesel::{BoxableExpression, Column, ExpressionMethods, SelectableExpression};
use failure::Error;
use juniper::LookAheadValue;

pub trait BuildOrder<T, DB> {
    fn build_order(
        order: &[LookAheadValue<'_, WundergraphScalarValue>],
        field_name: impl Fn(usize) -> &'static str,
    ) -> Result<Vec<Box<dyn BoxableExpression<T, DB, SqlType = ()>>>, Error>;
}

macro_rules! impl_order_builder {
    ($(
        $Tuple:tt {
            $(($idx:tt) -> $T:ident, $ST: ident, $TT: ident,) +
        }
    )+) => {
        $(
            impl<Table, DB, $($T,)+> BuildOrder<Table, DB> for ($($T,)+)
            where Table: ::diesel::Table,
                  DB: Backend,
            $($T: Column<Table = Table> + ExpressionMethods + Copy + Default +
              SelectableExpression<Table> + NonAggregate + QueryFragment<DB> + 'static,)+
            {
                fn build_order(
                    fields: &[LookAheadValue<'_, WundergraphScalarValue>],
                    field_name: impl Fn(usize) -> &'static str,
                ) -> Result<Vec<Box<dyn BoxableExpression<Table, DB, SqlType = ()>>>, Error>
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
                                x if x == field_name($idx) => if order == Order::Desc {
                                    ret.push(Box::new($T::default().desc())
                                             as Box<dyn BoxableExpression<Table, DB, SqlType = ()>>)
                                } else {
                                    ret.push(Box::new($T::default().asc()) as Box<_>)
                                }
                            )+
                                x => {
                                    return Err(Error::from(
                                        WundergraphError::UnknownDatabaseField{
                                            name: x.to_owned()
                                        }
                                    ))
                                }
                            }
                        } else {
                            return Err(Error::from(
                                WundergraphError::CouldNotBuildFilterArgument
                            ));
                        }
                    }
                    Ok(ret)
                }
            }
        )+
    }
}

__diesel_for_each_tuple!(impl_order_builder);
