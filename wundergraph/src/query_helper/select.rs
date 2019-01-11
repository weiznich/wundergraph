use diesel::backend::Backend;
use diesel::expression::NonAggregate;
use diesel::query_builder::QueryFragment;
use diesel::{BoxableExpression, Column, Expression, ExpressionMethods, SelectableExpression};
use failure::Error;
use juniper::LookAheadMethods;
use juniper::LookAheadSelection;
use query_helper::maybe_null::MaybeNull;
use scalar::WundergraphScalarValue;

pub trait BuildSelect<T: ::diesel::Table, DB, ST> {
    fn build_select(
        select: &LookAheadSelection<WundergraphScalarValue>,
        name_list: &'static [&'static str],
        sql_name_indices: &'static [usize],
        non_sql_name_indices: &'static [usize],
        primary_key_index: usize,
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
                    name_list: &'static [&'static str],
                    sql_name_indices: &'static [usize],
                    non_sql_name_indices: &'static [usize],
                    primary_key_index: usize,
    ) -> Result<
        Box<
            dyn BoxableExpression<
                Table,
                DB,
                SqlType = ($(<MaybeNull<$T> as Expression>::SqlType,)+)>,
        >,
        Error,
                > {
                    Ok(Box::new((
                        $(
                            if select.has_child(name_list[sql_name_indices[$idx]]) ||
                                (
                                    primary_key_index == $idx &&
                                        non_sql_name_indices.iter().any(|i| select.has_child(name_list[*i]))
                                ) {
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
