use super::{FieldListExtractor, NonTableFieldExtractor, WundergraphResolveAssociations};
use crate::context::WundergraphContext;
use crate::query_builder::types::placeholder::PlaceHolderMarker;
use crate::query_builder::types::WundergraphValue;
use crate::query_builder::selection::query_resolver::WundergraphResolvePlaceHolderList;
use crate::helper::tuple::TupleIndex;
use crate::scalar::WundergraphScalarValue;
use diesel::backend::Backend;
use diesel::{Connection, Queryable};
use failure::Error;
use juniper::{Executor, Selection};
use std::hash::Hash;

pub trait WundergraphFieldList<DB: Backend, Key, Table, Ctx> {
    type PlaceHolder: Queryable<Self::SqlType, DB> + 'static;
    type SqlType: 'static;

    const TABLE_FIELD_COUNT: usize;
    const NON_TABLE_FIELD_COUNT: usize;

    fn resolve(
        placeholder: Vec<Self::PlaceHolder>,
        select: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        name_list: &'static [&'static str],
        executor: &Executor<'_, Ctx, WundergraphScalarValue>,
    ) -> Result<Vec<juniper::Value<WundergraphScalarValue>>, Error>;

    fn map_table_field<F: Fn(usize) -> R, R>(local_index: usize, callback: F) -> Option<R>;
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
                    look_ahead: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
                    selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
                    name_list: &'static [&'static str],
                    executor: &Executor<'_, Ctx, WundergraphScalarValue>,
                ) -> Result<Vec<juniper::Value<WundergraphScalarValue>>, Error> {
                    let extern_values = {
                        let keys = ||{
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
                            look_ahead, selection, name, keys, executor,
                        )?
                    };
                    let name = |local_pos| {
                        <($($T,)*) as FieldListExtractor>::map(local_pos, |pos| {
                            name_list[pos]
                        }).expect("Name is there")
                    };
                    let objs = placeholder.resolve(
                        name,
                        look_ahead,
                        selection,
                        executor,
                    )?;

                     Ok(extern_values.merge_with_object_list(objs))
                }

                fn map_table_field<Func: Fn(usize) -> Ret, Ret>(local_index: usize, callback: Func) -> Option<Ret> {
                    <($($T,)*) as FieldListExtractor>::map(local_index, callback)
                }

                fn map_non_table_field<Func: Fn(usize) -> Ret, Ret>(local_index: usize, callback: Func) -> Option<Ret> {
                    <($($T,)*) as NonTableFieldExtractor>::map(local_index, callback)
                }
            }
        )*
    }
}
__diesel_for_each_tuple!(wundergraph_impl_field_list);
