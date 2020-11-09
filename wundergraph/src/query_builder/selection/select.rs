use crate::diesel_ext::MaybeNull;
use crate::error::Result;
use crate::scalar::WundergraphScalarValue;
use diesel::backend::Backend;
use diesel::expression::NonAggregate;
use diesel::query_builder::QueryFragment;
use diesel::{BoxableExpression, Expression, SelectableExpression};
use juniper::LookAheadMethods;
use juniper::LookAheadSelection;

/// A helper trait to construct a select clause for a given table out of
/// a given graphql request
pub trait BuildSelect<T: ::diesel::Table, DB, ST> {
    /// Construct the select clause out of a given graphql request
    fn build_select(
        select: &LookAheadSelection<'_, WundergraphScalarValue>,
        get_field_name: impl Fn(usize) -> &'static str,
        is_primary_key_index: impl Fn(usize) -> bool,
        should_select_primary_key: bool,
    ) -> Result<Box<dyn BoxableExpression<T, DB, SqlType = ST>>>;
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
                  DB::QueryBuilder: Default,
            $(
                $T: Default +
                    SelectableExpression<Table> +
                    NonAggregate +
                    QueryFragment<DB> +
                    QueryFragment<crate::diesel_ext::FakeBackend<DB>> +
                    'static ,
            )+
                $(MaybeNull<$T>: Expression,)+
            {
                fn build_select(
                    select: &LookAheadSelection<'_, WundergraphScalarValue>,
                    get_field_name: impl Fn(usize) -> &'static str,
                    is_primary_key_index: impl Fn(usize) -> bool,
                    should_select_primary_key: bool,
                ) -> Result<
                    Box<
                    dyn BoxableExpression<
                    Table,
                DB,
                SqlType = ($(<MaybeNull<$T> as Expression>::SqlType,)+)>,
                >>
                {
                    Ok(Box::new((
                        $({
                            if select.has_child(get_field_name($idx)) ||
                                (is_primary_key_index($idx) && should_select_primary_key)
                            {
                                MaybeNull::<$T>::expr()
                            } else {
                                MaybeNull::<$T>::as_null()
                            }
                        },)+
                    )) as Box<_>)
                }
            }
        )+
    }
}

__diesel_for_each_tuple!(impl_select_builder);
