/// A marker trait that says a given type could be used
/// as type level index into a tuple
pub trait IsPrimaryKeyIndex {
    /// Check if this type represents the index with the given value
    fn is_index(v: usize) -> bool;
}

/// A trait to have a type level index into a tuple
///
/// `Self` represents the tuple to index into, `N` the type level index
///
/// ```
/// # use wundergraph::helper::{TupleIndex, TupleIndex1};
/// #
/// let _a: <(i32, &str, f64) as TupleIndex<TupleIndex1>>::Value = "foo";
/// ```
pub trait TupleIndex<N> {

    /// The type of the indexed tuple value
    type Value;

    /// Get the actual value at the given index
    fn get(&self) -> Self::Value;
}

/// A type level helper to concat two tuples
///
/// `Self` represents the existing tuple, `Other` the tuple that should be
/// appended
pub trait ConcatTuples<Other> {

    /// The resulting tuple type
    type Out;
}

/// A type level helper trait to append another element to a tuple
///
/// `Self` is the existing tuple, `T` the element that is appended
pub trait AppendToTuple<T> {
    /// The resulting tuple type
    type Out;

    /// The length of the resulting tuple
    const LENGHT: usize;
}

impl<T> AppendToTuple<T> for () {
    type Out = (T,);

    const LENGHT: usize = 1;
}

/// A type level marker to be used as index into tuples
///
/// Basically equivalent to `tuple.0`, but at type system level
#[derive(Default, Debug, Clone, Copy)]
pub struct TupleIndex0;

impl IsPrimaryKeyIndex for TupleIndex0 {
    fn is_index(v: usize) -> bool {
        0 == v
    }
}

macro_rules! name_from_idx {
    ($id:expr, $callback:ident, $($params:tt)*) => {
        $crate::paste::item!{
            $callback! {
                [<TupleIndex $id>],
                $($params)*
            }
        }
    }
}

macro_rules! with_dollar_sign {
    ($($body:tt)*) => {
        macro_rules! __with_dollar_sign { $($body)* }
        __with_dollar_sign!($);
    }
}

macro_rules! impl_multiple_tuple_index {
    (
     @impl
            index = {$($T: ident,)*},
            tuple_var = {$($ST: ident,)*},
            tuple = {$tuple: tt},
    ) => {
        impl<$($T,)* $($ST,)*> TupleIndex<($($T,)*)> for ($($ST,)*)
        where
            $($ST: 'static,)*
            $(
                $tuple: TupleIndex<$T>,
            )*
        {
            type Value = (
                $(<$tuple as TupleIndex<$T>>::Value,)*
            );

            fn get(&self) -> Self::Value {
                ($(
                    <$tuple as TupleIndex<$T>>::get(self),
                )*)
            }
        }
    };

    (
        index = {$T: ident, $($TT: ident,)*},
        tuple = {$($ST: ident,)*},
    ) => {
        impl_multiple_tuple_index!{
            @impl
                index = {$T, $($TT,)*},
                tuple_var = {$($ST,)*},
                tuple = {($($ST,)*)},
        }
        impl_multiple_tuple_index!{
            index = {$($TT,)*},
            tuple = {$($ST,)*},
        }
    };
    (
        index = {},
        $($rest: tt)*
    ) => {}
}

macro_rules! impl_tuple_index{
    ($name: ident, $tuple_idx: tt, $($T: ident,)*) => {
        impl<'a, $($T:'static,)*> TupleIndex<$name> for ($($T,)*)
            where get_type!($tuple_idx, $($T,)*): Clone,
        {
            type Value = get_type!($tuple_idx, $($T,)*);

            fn get(&self) -> Self::Value {
                self.$tuple_idx.clone()
            }
        }
    };
}

macro_rules! create_tuple_index {
    (@call_tuple [$($idx:tt)*] @ $args:tt) => {
        $(create_tuple_index!(@call [$idx] $args);)*
    };
    (@call [$idx:tt] ($tuple_idx: tt, $($T: ident,)*)) => {
        name_from_idx!($idx, impl_tuple_index, $idx, $($T,)*);
    };
    ($name: ident, $tuple_idx: tt, $($T: ident,)*, $($idx: tt)*) => {
        /// A type level marker to be used as index into tuples
        #[derive(Default, Debug, Clone, Copy)]
        pub struct $name;

        impl IsPrimaryKeyIndex for $name {
            fn is_index(v: usize) -> bool {
                $tuple_idx == v
            }
        }
        create_tuple_index!(@call_tuple [$($idx)*] @  ($tuple_idx, $($T,)*));
    }
}

macro_rules! expand_concat_tuple{
    (@impl first = [($($T:ident,)*)], second = [($($ST:ident,)*)]) => {
        impl<$($T,)* $($ST,)*> ConcatTuples<($($ST,)*)> for ($($T,)*) {
            type Out = ($($T,)* $($ST,)*);
        }
    };
    (@decouple2 first = [$T:tt], second = [($({$ST:tt},)*)]) => {
        $(
            expand_concat_tuple!(
                @impl
                    first = [$T],
                second = [$ST]
            );
        )*
    };
    (@decouple first = [$({$T:tt},)*], second = [$ST:tt]) => {
        $(
            expand_concat_tuple!(
                @decouple2
                first = [$T],
                second = [$ST]

            );
        )*
    };
    (pairs = [$({first = [$($T: ident,)*], second =[$($ST: ident,)*]},)*]) => {
        expand_concat_tuple!(
            @decouple
                first = [$({($($T,)*)},)*],
            second = [($({($($ST,)*)},)*)]
        );
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

macro_rules! impl_tuple_macro_wrapper {
    ($(
        $Tuple:tt {
            $(($idx:tt) -> $T:ident, $ST: ident, $TT: ident,) +
        }
    )+) => {
        with_dollar_sign! {
            ($d:tt) => {
                macro_rules! get_type {
                    (0, $d O: ident, $d ($d rest: ident,)*) => {$d O};
                    $(($Tuple, $($d $T: ident,)* $d Target: ident, $d ($d rest:ident,)*) => {
                        $d Target
                    };)*
                }
            }
        }
        $(
            name_from_idx!($Tuple, create_tuple_index, $Tuple, $($T,)*, $($idx)*);
            impl_multiple_tuple_index!{
                index = {$($T,)*},
                tuple = {$( $ST,)*},
            }

            impl<$($T,)*> IsPrimaryKeyIndex for ($($T,)*)
            where $($T: IsPrimaryKeyIndex,)*
            {
                fn is_index(v: usize) -> bool {
                    $(
                        <$T as IsPrimaryKeyIndex>::is_index(v) ||
                    )*
                    false
                }
            }

            impl<$($T,)*> ConcatTuples<()> for ($($T,)*)
            {
                type Out = Self;
            }

            impl<$($T,)*> ConcatTuples<($($T,)*)> for () {
                type Out = ($($T,)*);

            }

            impl<$($T,)* New> AppendToTuple<New> for ($($T,)*) {
                type Out = ($($T,)* New);
                const LENGHT: usize = wundergraph_add_one_to_index!($($idx)*) + 1;
            }

        )*

        expand_concat_tuple!(pairs = [$({first = [$($T,)*], second = [$($ST,)*]},)*]);
    }
}

__diesel_for_each_tuple!(impl_tuple_macro_wrapper);
