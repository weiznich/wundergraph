#![deny(missing_debug_implementations, missing_copy_implementations)]
#![cfg_attr(feature = "cargo-clippy", allow(renamed_and_removed_lints))]
#![cfg_attr(feature = "cargo-clippy", warn(clippy))]
// Clippy lints
#![cfg_attr(feature = "cargo-clippy", allow(type_complexity))]
#![cfg_attr(
    feature = "cargo-clippy",
    warn(
        wrong_pub_self_convention,
        used_underscore_binding,
        use_self,
        use_debug,
        unseparated_literal_suffix,
        unnecessary_unwrap,
        unimplemented,
        single_match_else,
        shadow_unrelated,
        option_map_unwrap_or_else,
        option_map_unwrap_or,
        needless_continue,
        mutex_integer,
        needless_borrow,
        items_after_statements,
        filter_map,
        expl_impl_clone_on_copy,
        else_if_without_else,
        doc_markdown,
        default_trait_access,
        option_unwrap_used,
        result_unwrap_used,
        wrong_pub_self_convention,
        mut_mut,
        non_ascii_literal,
        similar_names,
        unicode_not_nfc,
        enum_glob_use,
        if_not_else,
        items_after_statements,
        used_underscore_binding
    )
)]

extern crate structopt;
#[macro_use]
extern crate diesel;
//extern crate infer_schema_internals;

use structopt::StructOpt;

mod database;
mod infer_schema_internals;
mod print_schema;

#[derive(StructOpt, Debug)]
#[structopt(name = "wundergraph")]
enum Wundergraph {
    #[structopt(name = "print-schema")]
    PrintSchema {
        database_url: String,
        schema: Option<String>,
    },
}

fn main() {
    match Wundergraph::from_args() {
        Wundergraph::PrintSchema {
            database_url,
            schema,
        } => print_schema::print(&database_url, schema.as_ref().map(|s| s as &str))
            .expect("Failed to infer the schema"),
    }
}
