mod delete;
mod insert;
mod update;

pub use self::delete::{handle_delete, HandleDelete};
pub use self::insert::{handle_batch_insert, handle_insert, HandleInsert};
pub use self::update::{handle_update, HandleUpdate};
