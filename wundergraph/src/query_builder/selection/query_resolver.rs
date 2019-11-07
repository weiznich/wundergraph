use crate::error::Result;
use crate::query_builder::selection::fields::WundergraphFieldList;
use crate::query_builder::types::field_value_resolver::FieldValueResolver;
use crate::query_builder::types::placeholder::{PlaceHolder, PlaceHolderMarker};
use crate::query_builder::types::{ResolveWundergraphFieldValue, WundergraphValue};
use crate::scalar::WundergraphScalarValue;
use diesel::backend::Backend;
use juniper::parser::SourcePosition;
use juniper::{Executor, LookAheadMethods, Selection};

/// A helper type that represents the diesel sql type of a given entity
///
/// # Generic Arguments:
/// * `T`: Wundergraph field list (Just a tuple of all table field types
///    of an entity)
/// * `DB`: Diesel backend
/// * `K`: Type level index of the primary key
/// * `Table`: Table this field list corresponds to
/// * `Ctx`: Used wundergraph context type
pub type SqlTypeOfPlaceholder<T, DB, K, Table, Ctx> =
    <T as WundergraphFieldList<DB, K, Table, Ctx>>::SqlType;

/// An internal trait used to resolve a wundergraph field list into
/// an actuall GraphQL query response
pub trait WundergraphResolvePlaceHolderList<R, DB: Backend, Ctx> {
    /// Resolve the field list by executing the required sql queries
    fn resolve(
        self,
        get_name: impl Fn(usize) -> &'static str,
        global_args: &[juniper::LookAheadArgument<WundergraphScalarValue>],
        look_ahead: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
        selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
        executor: &Executor<Ctx, WundergraphScalarValue>,
    ) -> Result<Vec<juniper::Object<WundergraphScalarValue>>>;
}

macro_rules! wundergraph_add_one_to_index {
    ($idx_head: tt $($idx: tt)+) => {
        wundergraph_add_one_to_index!{$($idx)*}
    };
    ($idx: tt) => {
        $idx + 1
    }
}

macro_rules! wundergraph_value_impl {
    ($(
        $Tuple:tt {
            $(($idx:tt) -> $T:ident, $ST: ident, $TT: ident,) +
        }
    )+) => {
        $(
            #[allow(clippy::use_self)]
            impl<Back, $($T,)+ $($ST,)+ Ctx> WundergraphResolvePlaceHolderList<($($ST,)*), Back, Ctx> for Vec<($(PlaceHolder<$T>,)+)>
            where $($ST: WundergraphValue<PlaceHolder = PlaceHolder<$T>> +
                    ResolveWundergraphFieldValue<Back, Ctx> ,)*
                  $($T: 'static,)*
                  Back: Backend,
            {
                fn resolve(
                    self,
                    get_name: impl Fn(usize) -> &'static str,
                    global_args: &[juniper::LookAheadArgument<WundergraphScalarValue>],
                    look_ahead: &juniper::LookAheadSelection<'_, WundergraphScalarValue>,
                    selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
                    executor: &Executor<Ctx, WundergraphScalarValue>,
                ) -> Result<Vec<juniper::Object<WundergraphScalarValue>>>
                {
                    let mut resolver = (
                        $(<$ST as ResolveWundergraphFieldValue<Back, Ctx>>::Resolver::new(self.len()),)*
                    );
                    let mut objs: Vec<juniper::Object<WundergraphScalarValue>>
                        = vec![juniper::Object::with_capacity(wundergraph_add_one_to_index!($($idx)*)-1); self.len()];

                    self.into_iter().zip(objs.iter_mut()).map(|(placeholder, obj)|{
                        $(
                            if let Some(look_ahead) = look_ahead.select_child(get_name($idx)) {
                                let (name, alias, pos, selection) = get_sub_field(get_name($idx), selection);
                                let executor = executor.field_sub_executor(alias, name, pos, selection);
                                if let Some(value) = resolver.$idx.resolve_value(
                                    placeholder.$idx,
                                    look_ahead,
                                    selection,
                                    &executor
                                )? {
                                    obj.add_field(alias, value);
                                }
                            }
                        )*
                        Ok(())
                    }).collect::<Result<Vec<_>>>()?;
                    $(
                        if let Some(look_ahead) = look_ahead.select_child(get_name($idx)) {
                            let (name, alias, pos, selection) = get_sub_field(get_name($idx), selection);
                            let executor = executor.field_sub_executor(alias, name, pos, selection);
                            let vals = resolver.$idx.finalize(global_args, look_ahead, selection, &executor)?;
                            if let Some(vals) = vals {
                                for (obj, val) in objs.iter_mut().zip(vals.into_iter()) {
                                    obj.add_field(alias, val);
                                }
                            }
                        }
                    )*
                    Ok(objs)
                }

            }



            impl<$($T,)*> PlaceHolderMarker for ($($T,)*)
            where $($T: PlaceHolderMarker,)*
            {
                type InnerType = ($(<$T as PlaceHolderMarker>::InnerType,)*);

                fn into_inner(self) -> Option<Self::InnerType> {
                    Some((
                        $(
                            <$T as PlaceHolderMarker>::into_inner(self.$idx)?,
                        )*
                    ))
                }
            }

        )+
    }
}

__diesel_for_each_tuple!(wundergraph_value_impl);

pub(crate) fn get_sub_field<'a>(
    field_name: &'a str,
    selection: Option<&'a [Selection<'a, WundergraphScalarValue>]>,
) -> (
    &'a str,
    &'a str,
    SourcePosition,
    Option<&'a [Selection<'a, WundergraphScalarValue>]>,
) {
    use juniper::parser::Spanning;
    if let Some(selection) = selection {
        selection
            .iter()
            .filter_map(|s| {
                if let Selection::Field(Spanning {
                    item: ref f,
                    ref start,
                    ..
                }) = *s
                {
                    if f.name.item == field_name {
                        Some((
                            f.name.item,
                            f.alias.unwrap_or(f.name).item,
                            *start,
                            f.selection_set.as_ref().map(|s| s as _),
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .next()
            .unwrap_or((field_name, field_name, SourcePosition::new(0, 0, 0), None))
    } else {
        (field_name, field_name, SourcePosition::new(0, 0, 0), None)
    }
}
