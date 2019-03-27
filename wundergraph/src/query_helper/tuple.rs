pub trait IsPrimaryKeyIndex {
    fn is_index(v: usize) -> bool;
}

pub trait TupleIndex<N> {
    type Value;

    fn get(&self) -> Self::Value;
}

pub trait ConcatTuples<Other> {
    type Out;
}

macro_rules! name_from_tuple {
    (1, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex0, $($params)*); };
    (2, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex1, $($params)*); };
    (3, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex2, $($params)*); };
    (4, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex3, $($params)*); };
    (5, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex4, $($params)*); };
    (6, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex5, $($params)*); };
    (7, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex6, $($params)*); };
    (8, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex7, $($params)*); };
    (9, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex8, $($params)*); };
    (10, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex9, $($params)*); };
    (11, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex10, $($params)*); };
    (12, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex11, $($params)*); };
    (13, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex12, $($params)*); };
    (14, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex13, $($params)*); };
    (15, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex14, $($params)*); };
    (16, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex15, $($params)*); };
    (17, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex16, $($params)*); };
    (18, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex17, $($params)*); };
    (19, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex18, $($params)*); };
    (20, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex19, $($params)*); };
    (21, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex20, $($params)*); };
    (22, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex21, $($params)*); };
    (23, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex22, $($params)*); };
    (24, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex23, $($params)*); };
    (25, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex24, $($params)*); };
    (26, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex25, $($params)*); };
    (27, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex26, $($params)*); };
    (28, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex27, $($params)*); };
    (29, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex28, $($params)*); };
    (30, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex29, $($params)*); };
    (31, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex30, $($params)*); };
    (32, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex31, $($params)*); };
}

macro_rules! name_from_idx {
    (0, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex0, $($params)*); };
    (1, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex1, $($params)*); };
    (2, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex2, $($params)*); };
    (3, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex3, $($params)*); };
    (4, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex4, $($params)*); };
    (5, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex5, $($params)*); };
    (6, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex6, $($params)*); };
    (7, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex7, $($params)*); };
    (8, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex8, $($params)*); };
    (9, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex9, $($params)*); };
    (10, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex10, $($params)*); };
    (11, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex11, $($params)*); };
    (12, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex12, $($params)*); };
    (13, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex13, $($params)*); };
    (14, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex14, $($params)*); };
    (15, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex15, $($params)*); };
    (16, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex16, $($params)*); };
    (17, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex17, $($params)*); };
    (18, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex18, $($params)*); };
    (19, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex19, $($params)*); };
    (20, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex20, $($params)*); };
    (21, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex21, $($params)*); };
    (22, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex22, $($params)*); };
    (23, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex23, $($params)*); };
    (24, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex24, $($params)*); };
    (25, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex25, $($params)*); };
    (26, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex26, $($params)*); };
    (27, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex27, $($params)*); };
    (28, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex28, $($params)*); };
    (29, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex29, $($params)*); };
    (30, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex30, $($params)*); };
    (31, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex31, $($params)*); };
    (32, $callback: ident, $($params: tt)*) => { $callback!(TupleIndex32, $($params)*); };
}

macro_rules! get_type {
    (0, $T: ident, $($rest: tt)*) => {$T};
    (1, $T: ident, $($rest: tt)*) => { get_type!(0, $($rest)*) };
    (2, $T: ident, $($rest: tt)*) => { get_type!(1, $($rest)*) };
    (3, $T: ident, $($rest: tt)*) => { get_type!(2, $($rest)*) };
    (4, $T: ident, $($rest: tt)*) => { get_type!(3, $($rest)*) };
    (5, $T: ident, $($rest: tt)*) => { get_type!(4, $($rest)*) };
    (6, $T: ident, $($rest: tt)*) => { get_type!(5, $($rest)*) };
    (7, $T: ident, $($rest: tt)*) => { get_type!(6, $($rest)*) };
    (8, $T: ident, $($rest: tt)*) => { get_type!(7, $($rest)*) };
    (9, $T: ident, $($rest: tt)*) => { get_type!(8, $($rest)*) };
    (10, $T: ident, $($rest: tt)*) => { get_type!(9, $($rest)*) };
    (11, $T: ident, $($rest: tt)*) => { get_type!(10, $($rest)*) };
    (12, $T: ident, $($rest: tt)*) => { get_type!(11, $($rest)*) };
    (13, $T: ident, $($rest: tt)*) => { get_type!(12, $($rest)*) };
    (14, $T: ident, $($rest: tt)*) => { get_type!(13, $($rest)*) };
    (15, $T: ident, $($rest: tt)*) => { get_type!(14, $($rest)*) };
    (16, $T: ident, $($rest: tt)*) => { get_type!(15, $($rest)*) };
    (17, $T: ident, $($rest: tt)*) => { get_type!(16, $($rest)*) };
    (18, $T: ident, $($rest: tt)*) => { get_type!(17, $($rest)*) };
    (19, $T: ident, $($rest: tt)*) => { get_type!(18, $($rest)*) };
    (20, $T: ident, $($rest: tt)*) => { get_type!(19, $($rest)*) };
    (21, $T: ident, $($rest: tt)*) => { get_type!(20, $($rest)*) };
    (22, $T: ident, $($rest: tt)*) => { get_type!(21, $($rest)*) };
    (23, $T: ident, $($rest: tt)*) => { get_type!(22, $($rest)*) };
    (24, $T: ident, $($rest: tt)*) => { get_type!(23, $($rest)*) };
    (25, $T: ident, $($rest: tt)*) => { get_type!(24, $($rest)*) };
    (26, $T: ident, $($rest: tt)*) => { get_type!(25, $($rest)*) };
    (27, $T: ident, $($rest: tt)*) => { get_type!(26, $($rest)*) };
    (28, $T: ident, $($rest: tt)*) => { get_type!(27, $($rest)*) };
    (29, $T: ident, $($rest: tt)*) => { get_type!(28, $($rest)*) };
    (30, $T: ident, $($rest: tt)*) => { get_type!(29, $($rest)*) };
    (31, $T: ident, $($rest: tt)*) => { get_type!(30, $($rest)*) };
    (32, $T: ident, $($rest: tt)*) => { get_type!(31, $($rest)*) };
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
        #[derive(Default, Debug, Clone, Copy)]
        pub struct $name;

        impl IsPrimaryKeyIndex for $name {
            fn is_index(v: usize) -> bool {
                $tuple_idx - 1 == v
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

macro_rules! impl_tuple_macro_wrapper {
    ($(
        $Tuple:tt {
            $(($idx:tt) -> $T:ident, $ST: ident, $TT: ident,) +
        }
    )+) => {
        $(
            name_from_tuple!($Tuple, create_tuple_index, $Tuple, $($T,)*, $($idx)*);
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
        )*

        expand_concat_tuple!(pairs = [$({first = [$($T,)*], second = [$($ST,)*]},)*]);
    }
}

__diesel_for_each_tuple!(impl_tuple_macro_wrapper);
