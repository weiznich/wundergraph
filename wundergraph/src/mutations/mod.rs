mod insert;
mod delete;
mod update;

pub use self::insert::{handle_batch_insert, handle_insert, HandleInsert};
pub use self::update::{handle_update, HandleUpdate};
pub use self::delete::{handle_delete, HandleDelete};

pub trait UnRef {
    type UnRefed;
}

impl<'a, A> UnRef for &'a A {
    type UnRefed = A;
}

impl<'a, A> UnRef for (&'a A,) {
    type UnRefed = (A,);
}

impl<'a, A, B> UnRef for (&'a A, &'a B) {
    type UnRefed = (A, B);
}

impl<'a, A, B, C> UnRef for (&'a A, &'a B, &'a C) {
    type UnRefed = (A, B, C);
}

impl<'a, A, B, C, D> UnRef for (&'a A, &'a B, &'a C, &'a D) {
    type UnRefed = (A, B, C, D);
}

impl<'a, A, B, C, D, E> UnRef for (&'a A, &'a B, &'a C, &'a D, &'a E) {
    type UnRefed = (A, B, C, D, E);
}
