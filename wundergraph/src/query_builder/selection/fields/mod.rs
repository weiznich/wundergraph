//! This module contains helper types to work with fields of wundergraph entities

mod associations;
mod field_list;
mod helper;

#[doc(inline)]
pub use self::helper::{
    FieldListExtractor, NonTableFieldCollector, NonTableFieldExtractor, TableFieldCollector,
};

#[doc(inline)]
pub use self::associations::WundergraphBelongsTo;
#[doc(inline)]
pub use self::field_list::WundergraphFieldList;
#[doc(inline)]
pub use wundergraph_derive::WundergraphBelongsTo;

pub(crate) use self::associations::WundergraphResolveAssociations;
