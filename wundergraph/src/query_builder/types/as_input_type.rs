pub trait AsInputType: Sized {
    type InputType;
}

impl AsInputType for i16 {
    type InputType = Self;
}

impl AsInputType for i32 {
    type InputType = Self;
}

impl AsInputType for i64 {
    type InputType = Self;
}

impl AsInputType for f32 {
    type InputType = Self;
}

impl AsInputType for f64 {
    type InputType = Self;
}

impl AsInputType for bool {
    type InputType = Self;
}

impl AsInputType for String {
    type InputType = Self;
}

impl<T> AsInputType for Vec<T>
where
    T: AsInputType,
    Vec<T::InputType>: Into<Self>,
{
    type InputType = Vec<T::InputType>;
}

impl<T> AsInputType for Option<T>
where
    T: AsInputType,
    Option<T::InputType>: Into<Self>,
{
    type InputType = Option<T::InputType>;
}

macro_rules! impl_tuple_macro_wrapper {
    ($(
        $Tuple:tt {
            $(($idx:tt) -> $T:ident, $ST: ident, $TT: ident,) +
        }
    )+) => {
        $(
            impl<$($T,)*> AsInputType for ($($T,)*)
            where
                $($T: AsInputType,)*
            {
                type InputType = ($($T::InputType,)*);
            }
        )*
    }
}

__diesel_for_each_tuple!(impl_tuple_macro_wrapper);
