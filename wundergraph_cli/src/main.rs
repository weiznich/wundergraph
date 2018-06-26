#[macro_use]
extern crate structopt;
extern crate infer_schema_internals;
extern crate diesel;

use structopt::StructOpt;

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
        } => print_schema::print(
            &database_url,
            schema.as_ref().map(|s| s as &str),
        ).unwrap(),
    }
}
