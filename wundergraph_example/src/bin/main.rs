#![deny(missing_debug_implementations, missing_copy_implementations)]
#![warn(
    clippy::print_stdout,
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
    clippy::option_map_unwrap_or,
    clippy::option_map_unwrap_or_else,
    clippy::redundant_clone,
    clippy::result_map_unwrap_or_else,
    clippy::unnecessary_unwrap,
    clippy::unseparated_literal_suffix,
    clippy::wildcard_dependencies
)]

use actix_web::web::{Data, Json};
use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use diesel::r2d2::{ConnectionManager, Pool};
use juniper::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use structopt::StructOpt;
use wundergraph::scalar::WundergraphScalarValue;
use wundergraph_example::mutations::Mutation;
use wundergraph_example::*;

#[derive(Debug, StructOpt)]
#[structopt(name = "wundergraph_example")]
struct Opt {
    #[structopt(short = "u", long = "db-url")]
    database_url: String,
    #[structopt(short = "s", long = "socket", default_value = "127.0.0.1:8000")]
    socket: String,
}

// actix integration stuff
#[derive(Serialize, Deserialize, Debug)]
pub struct GraphQLData(GraphQLRequest<WundergraphScalarValue>);

#[derive(Clone)]
struct AppState {
    schema: Arc<Schema<MyContext<DBConnection>>>,
    pool: Arc<Pool<ConnectionManager<DBConnection>>>,
}

fn graphiql() -> HttpResponse {
    let html = graphiql_source("/graphql");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

fn graphql(
    Json(GraphQLData(data)): Json<GraphQLData>,
    st: Data<AppState>,
) -> Result<HttpResponse, failure::Error> {
    let ctx = MyContext::new(st.get_ref().pool.get()?);
    let res = data.execute(&st.get_ref().schema, &ctx);
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&res)?))
}

fn run_migrations(conn: &DBConnection) {
    let mut migration_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    migration_path.push("migrations");
    if cfg!(feature = "postgres") {
        migration_path.push("pg");
    } else if cfg!(feature = "sqlite") {
        migration_path.push("sqlite");
    } else if cfg!(feature = "mysql") {
        migration_path.push("mysql");
    }
    let pending_migrations =
        ::diesel_migrations::mark_migrations_in_directory(conn, &migration_path)
            .unwrap()
            .into_iter()
            .filter_map(|(migration, run)| if run { None } else { Some(migration) });

    ::diesel_migrations::run_migrations(conn, pending_migrations, &mut ::std::io::stdout())
        .expect("Failed to run migrations");
}

#[allow(clippy::print_stdout)]
fn main() {
    let opt = Opt::from_args();
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let manager = ConnectionManager::<DBConnection>::new(opt.database_url);
    let pool = Pool::builder()
        .max_size(1)
        .build(manager)
        .expect("Failed to init pool");

    run_migrations(&pool.get().expect("Failed to get db connection"));

    let query = Query::<MyContext<DBConnection>>::default();
    let mutation = Mutation::<MyContext<DBConnection>>::default();
    let schema = Schema::new(query, mutation);

    let schema = Arc::new(schema);
    let pool = Arc::new(pool);
    let data = AppState { schema, pool };

    let url = opt.socket;

    println!("Started http server: http://{}", url);

    HttpServer::new(move || {
        App::new()
            .data(data.clone())
            .wrap(middleware::Logger::default())
            .route("/graphql", web::get().to(graphql))
            .route("/graphql", web::post().to(graphql))
            .route("/graphiql", web::get().to(graphiql))
            .default_service(web::route().to(|| {
                HttpResponse::Found()
                    .header("location", "/graphiql")
                    .finish()
            }))
    })
    .bind(&url)
    .expect("Failed to start server")
    .run()
    .unwrap();
}
