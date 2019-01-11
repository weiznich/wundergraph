//! Some helper functionality used to implement wundergraph
//!
//! Functionality from this module is only useful if you want to extend
//! wundergraph to support additional types
mod from_lookahead;
mod nameable;
#[doc(hidden)]
pub mod primary_keys;

pub use self::from_lookahead::FromLookAheadValue;
pub use self::nameable::{NameBuilder, Nameable};
