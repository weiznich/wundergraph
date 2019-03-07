use diesel::backend::Backend;
use diesel::expression::NonAggregate;
use diesel::query_builder::QueryFragment;
use diesel::{BoxableExpression, Column, Expression, ExpressionMethods, SelectableExpression};
use failure::Error;
use juniper::LookAheadMethods;
use juniper::LookAheadSelection;
use crate::query_helper::maybe_null::MaybeNull;
use crate::scalar::WundergraphScalarValue;

pub trait BuildSelect<T: ::diesel::Table, DB, ST> {
    fn build_select(
        select: &LookAheadSelection<WundergraphScalarValue>,
        get_field_name: impl Fn(usize) -> &'static str,
        is_primary_key_index: impl Fn(usize) -> bool,
        should_select_primary_key: bool,
    ) -> Result<Box<dyn BoxableExpression<T, DB, SqlType = ST>>, Error>;
}

macro_rules! impl_select_builder {
    ($(
        $Tuple:tt {
            $(($idx:tt) -> $T:ident, $ST: ident, $TT: ident,) +
        }
    )+) => {
        $(
            impl<Table, DB, $($T,)+> BuildSelect<
                Table, DB, ($(<MaybeNull<$T> as Expression>::SqlType,)+ ),
                > for ($($T,)+)
            where Table: ::diesel::Table,
                DB: Backend,
            $($T: Column<Table = Table> + Default + ExpressionMethods +
              SelectableExpression<Table> + NonAggregate + QueryFragment<DB> + 'static ,)+
                $(MaybeNull<$T>: Expression,)+
            {
                fn build_select(
                    select: &LookAheadSelection<WundergraphScalarValue>,
                    get_field_name: impl Fn(usize) -> &'static str,
                    is_primary_key_index: impl Fn(usize) -> bool,
                    should_select_primary_key: bool,
                ) -> Result<
                    Box<
                    dyn BoxableExpression<
                    Table,
                DB,
                SqlType = ($(<MaybeNull<$T> as Expression>::SqlType,)+)>,
                >,
                Error,
                >
                {
                    Ok(Box::new((
                        $(
                            if select.has_child(get_field_name($idx)) ||
                                (is_primary_key_index($idx) && should_select_primary_key)
                            {
                                MaybeNull::Expr($T::default())
                            } else {
                                MaybeNull::Null
                            },
                        )+
                    )) as Box<_>)
                }
            }
        )+
    }
}

__diesel_for_each_tuple!(impl_select_builder);
