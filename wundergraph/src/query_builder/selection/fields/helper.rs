use crate::helper::tuple::AppendToTuple;
use crate::query_builder::types::{
    AssociatedValue, HasMany, TableField, WundergraphSqlValue, WundergraphValue,
};

/// A helper trait to collect extracted graphql fields which represents a
/// database value
pub trait TableFieldCollector<T> {
    /// List of all collected fields
    ///
    /// Normally a tuple with `FIELD_COUNT` values
    type Out;

    /// Number of collected fields
    const FIELD_COUNT: usize;

    /// Execute the given callback with the converted global index
    /// (inside the complete field list) calculated from the passed local index
    /// (inside this specific field list)
    fn map<F: Fn(usize) -> R, R>(local_index: usize, callback: F) -> Option<R>;
}

/// A helper trait to collect extracted graphql fields which not represent a
/// database value
pub trait NonTableFieldCollector<T> {
    /// List of all collected fields
    ///
    /// Normally a tuple with `FIELD_COUNT` values
    type Out;

    /// Number of collected fields
    const FIELD_COUNT: usize;

    /// Execute the given callback with the converted global index
    /// (inside the complete field list) calculated from the passed local index
    /// (inside this specific field list)
    fn map<F: Fn(usize) -> R, R>(local_index: usize, callback: F) -> Option<R>;
}

/// A helper trati to exctrat graphql fields, that represent database values,
/// from the global field list
pub trait FieldListExtractor {
    /// List of extracted fields
    ///
    /// Normally a tuple with `FIELD_COUNT` values
    type Out;

    /// Number of extracted fields
    const FIELD_COUNT: usize;

    /// Execute the given callback with the converted global index
    /// (inside the complete field list) calculated from the passed local index
    /// (inside this specific field list)
    fn map<F: Fn(usize) -> R, R>(local_index: usize, callback: F) -> Option<R>;
}

/// A helper trati to exctrat graphql fields, which don't represent database values,
/// from the global field list
pub trait NonTableFieldExtractor {
    /// List of extracted fields
    ///
    /// Normally a tuple with `FIELD_COUNT` values
    type Out;

    /// Number of extracted fields
    const FIELD_COUNT: usize;

    /// Execute the given callback with the converted global index
    /// (inside the complete field list) calculated from the passed local index
    /// (inside this specific field list)
    fn map<F: Fn(usize) -> R, R>(local_index: usize, callback: F) -> Option<R>;
}

impl FieldListExtractor for () {
    type Out = ();

    const FIELD_COUNT: usize = 0;

    fn map<F: Fn(usize) -> R, R>(_local_index: usize, _callback: F) -> Option<R> {
        None
    }
}

impl NonTableFieldExtractor for () {
    type Out = ();

    const FIELD_COUNT: usize = 0;

    fn map<F: Fn(usize) -> R, R>(_local_index: usize, _callback: F) -> Option<R> {
        None
    }
}

impl<T> TableFieldCollector<T> for ()
where
    T: WundergraphSqlValue,
{
    type Out = (T,);

    const FIELD_COUNT: usize = 1;

    fn map<F: Fn(usize) -> R, R>(local_index: usize, callback: F) -> Option<R> {
        if local_index == 0 {
            Some(callback(0))
        } else {
            None
        }
    }
}

impl<T, FK> TableFieldCollector<HasMany<T, FK>> for () {
    type Out = ();

    const FIELD_COUNT: usize = 0;

    fn map<F: Fn(usize) -> R, R>(_local_index: usize, _callback: F) -> Option<R> {
        None
    }
}

impl<T> NonTableFieldCollector<T> for ()
where
    T: WundergraphSqlValue,
{
    type Out = ();

    const FIELD_COUNT: usize = 0;

    fn map<F: Fn(usize) -> R, R>(_local_index: usize, _callback: F) -> Option<R> {
        None
    }
}

impl<T, FK> NonTableFieldCollector<HasMany<T, FK>> for () {
    type Out = (HasMany<T, FK>,);

    const FIELD_COUNT: usize = 1;

    fn map<F: Fn(usize) -> R, R>(local_index: usize, callback: F) -> Option<R> {
        if local_index == 0 {
            Some(callback(0))
        } else {
            None
        }
    }
}

macro_rules! wundergraph_add_one_to_index {
    ($idx_head: tt $($idx: tt)+) => {
        wundergraph_add_one_to_index!{$($idx)*}
    };
    ($idx: tt) => {
        $idx + 1
    }
}

macro_rules! wundergraph_impl_field_extractor {
    ($($T: ident,)*) => {
        wundergraph_impl_field_extractor!{
            t = [$($T,)*],
            rest = [],
        }
    };
    (
        t = [$T:ident, $($Ts:ident,)+],
        rest = [$($Other:ident,)*],
    ) => {
        wundergraph_impl_field_extractor!{
            t = [$($Ts,)*],
            rest = [$($Other,)* $T,],
        }
    };
    (
        t = [$T:ident,],
        rest = [$($Other:ident,)*],
    ) => {
        impl<$($Other,)* $T> FieldListExtractor for ($($Other,)* $T,)
        where ($($Other,)*): TableFieldCollector<$T>
        {
            type Out = <($($Other,)*) as TableFieldCollector<$T>>::Out;

            const FIELD_COUNT: usize = <($($Other,)*) as TableFieldCollector<$T>>::FIELD_COUNT;

            fn map<Func: Fn(usize) -> Ret, Ret>(local_index: usize, callback: Func) -> Option<Ret> {
                <($($Other,)*) as TableFieldCollector<$T>>::map(local_index, callback)
            }
        }

        impl<$($Other,)* $T> NonTableFieldExtractor for ($($Other,)* $T,)
        where ($($Other,)*): NonTableFieldCollector<$T>
        {
            type Out = <($($Other,)*) as NonTableFieldCollector<$T>>::Out;

            const FIELD_COUNT: usize = <($($Other,)*) as NonTableFieldCollector<$T>>::FIELD_COUNT;

            fn map<Func: Fn(usize) -> Ret, Ret>(local_index: usize, callback: Func) -> Option<Ret> {
                <($($Other,)*) as NonTableFieldCollector<$T>>::map(local_index, callback)
            }
        }
    };
}

#[allow(missing_debug_implementations)]
pub struct FieldCollectorHelper<T, N, A>(std::marker::PhantomData<(T, N, A)>);

macro_rules! wundergraph_impl_field_extractors {
    ($(
        $Tuple:tt {
            $(($idx:tt) -> $T:ident, $ST: ident, $TT: ident,) +
        }
    )+) => {
        $(
            wundergraph_impl_field_extractor!($($T,)*);

            impl<$($T,)+ Next> TableFieldCollector<Next> for ($($T,)*)
            where
                Next: WundergraphValue,
            FieldCollectorHelper<($($T,)*), Next, Next::ValueType>: TableFieldCollector<Next> {
                type Out = <FieldCollectorHelper<($($T,)*), Next, Next::ValueType> as TableFieldCollector<Next>>::Out;

                const FIELD_COUNT: usize = <FieldCollectorHelper<($($T,)*), Next, Next::ValueType> as TableFieldCollector<Next>>::FIELD_COUNT;

                fn map<Func: Fn(usize) -> Ret, Ret>(local_index: usize, callback: Func) -> Option<Ret>
                {
                    <FieldCollectorHelper<($($T,)*), Next, Next::ValueType> as TableFieldCollector<Next>>::map(local_index, callback)
                }
            }

            impl<$($T,)* Next> TableFieldCollector<Next> for FieldCollectorHelper<($($T,)*), Next, TableField>
            where Next: WundergraphValue<ValueType = TableField>,
                  ($($T,)*): FieldListExtractor,
                  <($($T,)*) as FieldListExtractor>::Out: AppendToTuple<Next>,
            {
                type Out = <<($($T,)*) as FieldListExtractor>::Out as AppendToTuple<Next>>::Out;

                const FIELD_COUNT: usize = <<($($T,)*) as FieldListExtractor>::Out as AppendToTuple<Next>>::LENGHT;

                fn map<Func: Fn(usize) -> Ret, Ret>(local_index: usize, callback: Func) -> Option<Ret> {
                    if local_index == <<($($T,)*) as FieldListExtractor>::Out as AppendToTuple<Next>>::LENGHT - 1 {
                        Some(callback(wundergraph_add_one_to_index!($($idx)*)))
                    } else {
                        <($($T,)*) as FieldListExtractor>::map(local_index, callback)
                    }
                }
            }

            impl<$($T,)* Next> TableFieldCollector<Next> for FieldCollectorHelper<($($T,)*), Next, AssociatedValue>
            where
                Next: WundergraphValue<ValueType = AssociatedValue>,
                ($($T,)*): FieldListExtractor,
            {
                type Out = <($($T,)*) as FieldListExtractor>::Out;

                const FIELD_COUNT: usize = <($($T,)*) as FieldListExtractor>::FIELD_COUNT;

                fn map<Func: Fn(usize) -> Ret, Ret>(local_index: usize, callback: Func) -> Option<Ret> {
                    <($($T,)*) as FieldListExtractor>::map(local_index, callback)
                }
            }

            impl<$($T,)+ Next> NonTableFieldCollector<Next> for ($($T,)*)
            where
                Next: WundergraphValue,
            FieldCollectorHelper<($($T,)*), Next, Next::ValueType>: NonTableFieldCollector<Next> {
                type Out = <FieldCollectorHelper<($($T,)*), Next, Next::ValueType> as NonTableFieldCollector<Next>>::Out;

                const FIELD_COUNT: usize = <FieldCollectorHelper<($($T,)*), Next, Next::ValueType> as NonTableFieldCollector<Next>>::FIELD_COUNT;

                fn map<Func: Fn(usize) -> Ret, Ret>(local_index: usize, callback: Func) -> Option<Ret>
                {
                    <FieldCollectorHelper<($($T,)*), Next, Next::ValueType> as NonTableFieldCollector<Next>>::map(local_index, callback)
                }
            }

            impl<$($T,)* Next> NonTableFieldCollector<Next> for FieldCollectorHelper<($($T,)*), Next, TableField>
            where Next: WundergraphValue<ValueType = TableField>,
                  ($($T,)*): NonTableFieldExtractor,
            {
                type Out = <($($T,)*) as NonTableFieldExtractor>::Out;

                const FIELD_COUNT: usize = <($($T,)*) as NonTableFieldExtractor>::FIELD_COUNT;

                fn map<Func: Fn(usize) -> Ret, Ret>(local_index: usize, callback: Func) -> Option<Ret> {
                    <($($T,)*) as NonTableFieldExtractor>::map(local_index, callback)
                }
            }

            impl<$($T,)* Next> NonTableFieldCollector<Next> for FieldCollectorHelper<($($T,)*), Next, AssociatedValue>
            where
                  Next: WundergraphValue<ValueType = AssociatedValue>,
                  ($($T,)*): NonTableFieldExtractor,
                  <($($T,)*) as NonTableFieldExtractor>::Out: AppendToTuple<Next>,
            {
                type Out = <<($($T,)*) as NonTableFieldExtractor>::Out as AppendToTuple<Next>>::Out;

                const FIELD_COUNT: usize = <<($($T,)*) as NonTableFieldExtractor>::Out as AppendToTuple<Next>>::LENGHT;

                fn map<Func: Fn(usize) -> Ret, Ret>(local_index: usize, callback: Func) -> Option<Ret> {
                    if local_index == <<($($T,)*) as NonTableFieldExtractor>::Out as AppendToTuple<Next>>::LENGHT - 1 {
                        Some(callback(wundergraph_add_one_to_index!($($idx)*)))
                    } else {
                        <($($T,)*) as NonTableFieldExtractor>::map(local_index, callback)
                    }
                }
            }

        )*
    }
}

__diesel_for_each_tuple!(wundergraph_impl_field_extractors);
