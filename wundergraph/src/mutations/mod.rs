//! This module contains all functionality that is needed to implement mutations
//!
//! In general mutations should just work without any additional work that
//! writing some struct definition and deriving basic diesel for them
//! For special cases a manual implementation of one of the for exported traits
//! is required
//!
//! # Insert
//!
//! The easiest way to provide a single table insert mutation is to crate a struct
//! with all corresponding fields that derive `#[derive(Insertable, GrahpQLInputobject)]`
//! ```rust
//! # #[macro_use]
//! # extern crate diesel;
//! # #[macro_use]
//! # extern crate juniper;
//! # table! {
//! #    heros {
//! #        id -> Integer,
//! #        name -> Text,
//! #        species -> Nullable<Integer>,
//! #        home_world -> Nullable<Integer>,
//! #    }
//! # }
//!
//! #[derive(Insertable, GraphQLInputObject, Clone, Debug)]
//! #[table_name = "heros"]
//! pub struct NewHero {
//!    name: String,
//!    species: i32,
//!    home_world: Option<i32>,
//! }
//! # fn main() {}
//! ```
//!
//! For more complex cases like inserts that involve multiple tables at one
//! implement [`HandleInsert`](trait.HandleInsert.html) and
//! [`InsertHelper`](trait.InsertHelper.html) manually
//!
//! # Update
//!
//! Similar to `Insert` operations the easiest way to provide a single table update
//! mutation is to create a struct with all corresponding fields that derive
//! `#[derive(AsChangeset, GraphqlInputObject, Identifiable)]`
//! ```rust
//! # #[macro_use]
//! # extern crate diesel;
//! # #[macro_use]
//! # extern crate juniper;
//! # table! {
//! #    heros {
//! #        id -> Integer,
//! #        name -> Text,
//! #        species -> Nullable<Integer>,
//! #        home_world -> Nullable<Integer>,
//! #    }
//! # }
//!
//! #[derive(AsChangeset, GraphQLInputObject, Identifiable, Debug)]
//! #[table_name = "heros"]
//! pub struct HeroChangeset {
//!     id: i32,
//!     name: Option<String>,
//!     species: Option<i32>,
//!     home_world: Option<i32>,
//! }
//! # fn main() {}
//! ```

mod delete;
mod insert;
mod update;

pub use self::delete::{handle_delete, DeletedCount, HandleDelete};
pub use self::insert::{handle_batch_insert, handle_insert, HandleBatchInsert, HandleInsert};
pub use self::update::{handle_update, HandleUpdate};
