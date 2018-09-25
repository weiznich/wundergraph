#[macro_use]
mod query;
#[macro_use]
mod mutation;

#[doc(hidden)]
#[macro_export]
macro_rules! __wundergraph_debug_log_wrapper {
    ($($t:tt)*) => {$crate::log::debug!($($t)*)}
}
