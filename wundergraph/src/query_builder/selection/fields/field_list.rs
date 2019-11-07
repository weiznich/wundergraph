use super::{FieldListExtractor, NonTableFieldExtractor, WundergraphResolveAssociations};
use crate::context::WundergraphContext;
use crate::query_builder::types::placeholder::PlaceHolderMarker;
use crate::query_builder::types::WundergraphValue;
use crate::query_builder::selection::query_resolver::WundergraphResolvePlaceHolderList;
use crate::helper::tuple::TupleIndex;
use crate::scalar::WundergraphScalarValue;
use crate::error::Result;
use diesel::backend::Backend;
use diesel::{Connection, Queryable};
use juniper::{Executor, Selection};
use std::hash::Hash;

/// A internal trait
pub trait WundergraphFieldList<DB: Backend, Key, Table, Ctx> {
    /// Placeholder type
    ///
    /// Normally a tuple with `TABLE_FIELD_COUNT` entries of type
    /// `PlaceHolder<RustFieldType>` for the corresponding entry in `SqlType`
    type PlaceHolder: Queryable<Self::SqlType, DB> + 'static;
    /// The sql type of the field list
    /// Normally a tuple with `TABLE_FIELD_COUNT` entries representing
    /// the (diesel) sql type of the executed query
    type SqlType: 'static;

    /// Number of fields representing a database column
    const TABLE_FIELD_COUNT: usize;
    /// Number of fields not representing a database column
    const NON_TABLE_FIELD_COUNT: usize;

    /// Resolve all fields in an already executed graphql request
    ///
    /// The results of the executed sql query are contained in `placeholder`
    fn resolve(
        placeholder: Vec<Self::PlaceHolder>,
        global_args: &[juniper::LookAheadArgument<WundergraphScalarValue>],
        select: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        name_list: &'static [&'static str],
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    ) -> Result<Vec<juniper::Value<WundergraphScalarValue>>>;

    #[doc(hidden)]
    fn map_table_field<F: Fn(usize) -> R, R>(local_index: usize, callback: F) -> Option<R>;
    #[doc(hidden)]
    fn map_non_table_field<Func: Fn(usize) -> Ret, Ret>(
        local_index: usize,
        callback: Func,
    ) -> Option<Ret>;
}

macro_rules! wundergraph_impl_field_list {
    ($(
        $Tuple:tt {
            $(($idx:tt) -> $T:ident, $ST: ident, $TT: ident,) +
        }
    )+) => {
        $(

            impl<Back, Key, Table, Ctx, $($T,)*> WundergraphFieldList<Back, Key, Table, Ctx> for ($($T,)*)
            where Back: Backend,
                  ($($T,)*): FieldListExtractor + NonTableFieldExtractor,
                  <($($T,)*) as FieldListExtractor>::Out: WundergraphValue,
                  <<($($T,)*) as FieldListExtractor>::Out as WundergraphValue>::PlaceHolder: TupleIndex<Key> +
                      Queryable<<<($($T,)*) as FieldListExtractor>::Out as WundergraphValue>::SqlType, Back> + 'static,
            Vec<<<($($T,)*) as FieldListExtractor>::Out as WundergraphValue>::PlaceHolder>:
            WundergraphResolvePlaceHolderList<<($($T,)*) as FieldListExtractor>::Out, Back, Ctx>,
            <<<($($T,)*) as FieldListExtractor>::Out as WundergraphValue>::PlaceHolder as TupleIndex<Key>>::Value: PlaceHolderMarker,
            <<<<($($T,)*) as FieldListExtractor>::Out as WundergraphValue>::PlaceHolder as TupleIndex<Key>>::Value as PlaceHolderMarker>::InnerType: Eq + Hash + Clone,
            <($($T,)*) as NonTableFieldExtractor>::Out: WundergraphResolveAssociations<<<<<($($T,)*) as FieldListExtractor>::Out as WundergraphValue>::PlaceHolder as TupleIndex<Key>>::Value as PlaceHolderMarker>::InnerType, Table, Back, Ctx>,
            Ctx: WundergraphContext,
            Ctx::Connection: Connection<Backend = Back>,
            {
                type PlaceHolder = <<($($T,)*) as FieldListExtractor>::Out as WundergraphValue>::PlaceHolder;
                type SqlType = <<($($T,)*) as FieldListExtractor>::Out as WundergraphValue>::SqlType;

                const TABLE_FIELD_COUNT: usize = <($($T,)*) as FieldListExtractor>::FIELD_COUNT;
                const NON_TABLE_FIELD_COUNT: usize = <($($T,)*) as NonTableFieldExtractor>::FIELD_COUNT;

                fn resolve(
                    placeholder: Vec<Self::PlaceHolder>,
                    global_args: &[juniper::LookAheadArgument<WundergraphScalarValue>],
                    look_ahead: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
                    selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
                    name_list: &'static [&'static str],
                    executor: &Executor<'_, Ctx, WundergraphScalarValue>,
                ) -> Result<Vec<juniper::Value<WundergraphScalarValue>>> {
                    let extern_values = {
                        let keys = || {
                            placeholder.iter()
                                .map(TupleIndex::<Key>::get)
                                .map(<_ as PlaceHolderMarker>::into_inner)
                                .collect::<Vec<_>>()
                        };

                        let name = |local_pos| {
                            <($($T,)*) as NonTableFieldExtractor>::map(
                                local_pos,
                                |pos| name_list[pos]
                            ).expect("Name is there")
                        };
                        <($($T,)*) as NonTableFieldExtractor>::Out::resolve(
                            global_args, look_ahead, selection, name, keys, executor,
                        )?
                    };
                    let name = |local_pos| {
                        <($($T,)*) as FieldListExtractor>::map(local_pos, |pos| {
                            name_list[pos]
                        }).expect("Name is there")
                    };
                    let objs = placeholder.resolve(
                        name,
                        global_args,
                        look_ahead,
                        selection,
                        executor,
                    )?;

                     Ok(extern_values.merge_with_object_list(objs))
                }

                #[inline(always)]
                fn map_table_field<Func: Fn(usize) -> Ret, Ret>(local_index: usize, callback: Func) -> Option<Ret> {
                    <($($T,)*) as FieldListExtractor>::map(local_index, callback)
                }

                #[inline(always)]
                fn map_non_table_field<Func: Fn(usize) -> Ret, Ret>(local_index: usize, callback: Func) -> Option<Ret> {
                    <($($T,)*) as NonTableFieldExtractor>::map(local_index, callback)
                }
            }
        )*
    }
}
__diesel_for_each_tuple!(wundergraph_impl_field_list);
