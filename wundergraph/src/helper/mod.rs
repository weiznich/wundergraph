//! A module containing various helper traits and types mostly useful
//! to work with tuples at compile time

pub(crate) mod primary_keys;
pub(crate) mod tuple;

#[doc(inline)]
pub use self::primary_keys::NamedTable;
#[doc(inline)]
pub use self::primary_keys::PrimaryKeyInputObject;
#[doc(inline)]
pub use self::primary_keys::UnRef;
#[doc(inline)]
pub use self::primary_keys::UnRefClone;
#[doc(hidden)]
pub use self::primary_keys::{PrimaryKeyArgument, PrimaryKeyInfo};

#[doc(inline)]
pub use self::tuple::AppendToTuple;
#[doc(inline)]
pub use self::tuple::ConcatTuples;
#[doc(inline)]
pub use self::tuple::IsPrimaryKeyIndex;
#[doc(inline)]
pub use self::tuple::TupleIndex;
#[doc(inline)]
pub use self::tuple::*;
