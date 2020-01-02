#![deny(missing_debug_implementations, missing_copy_implementations)]
#![warn(
    clippy::option_unwrap_used,
    clippy::result_unwrap_used,
    clippy::wrong_pub_self_convention,
    clippy::mut_mut,
    clippy::non_ascii_literal,
    clippy::similar_names,
    clippy::unicode_not_nfc,
    clippy::enum_glob_use,
    clippy::if_not_else,
    clippy::items_after_statements,
    clippy::used_underscore_binding,
    clippy::cargo_common_metadata,
    clippy::dbg_macro,
    clippy::doc_markdown,
    clippy::filter_map,
    clippy::map_flatten,
    clippy::match_same_arms,
    clippy::needless_borrow,
    clippy::needless_pass_by_value,
    clippy::option_map_unwrap_or,
    clippy::option_map_unwrap_or_else,
    clippy::redundant_clone,
    clippy::result_map_unwrap_or_else,
    clippy::unnecessary_unwrap,
    clippy::unseparated_literal_suffix,
    clippy::wildcard_dependencies
)]

#[macro_use]
extern crate diesel;

use structopt::StructOpt;

mod database;
mod infer_schema_internals;
mod print_schema;

use crate::database::InferConnection;

#[derive(StructOpt, Debug)]
#[structopt(name = "wundergraph")]
#[allow(clippy::result_unwrap_used)]
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
        } => {
            let conn = InferConnection::establish(&database_url).unwrap_or_else(|_| {
                panic!("Unable to connect to database with url: {}", database_url)
            });
            print_schema::print(
                &conn,
                schema.as_ref().map(|s| s as &str),
                &mut std::io::stdout(),
            )
        }
        .expect("Failed to infer the schema"),
    }
}
