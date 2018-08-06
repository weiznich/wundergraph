#![deny(warnings, missing_debug_implementations, missing_copy_implementations)]
// Clippy lints
#![cfg_attr(feature = "clippy", allow(unstable_features))]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy(conf_file = "../../clippy.toml")))]
#![cfg_attr(
    feature = "clippy",
    allow(option_map_unwrap_or_else, option_map_unwrap_or, match_same_arms, type_complexity)
)]
#![cfg_attr(
    feature = "clippy",
    warn(
        option_unwrap_used, result_unwrap_used, wrong_pub_self_convention, mut_mut,
        non_ascii_literal, similar_names, unicode_not_nfc, enum_glob_use, if_not_else,
        items_after_statements, used_underscore_binding
    )
)]

#[macro_use]
extern crate structopt;
#[macro_use]
extern crate diesel;
//extern crate infer_schema_internals;

use structopt::StructOpt;

mod print_schema;
mod infer_schema_internals;
mod database;

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
