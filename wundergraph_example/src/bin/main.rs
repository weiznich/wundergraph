#![deny(missing_debug_implementations, missing_copy_implementations)]
#![warn(
    clippy::option_unwrap_used,
    clippy::result_unwrap_used,
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
    clippy::needless_pass_by_value,
    clippy::option_map_unwrap_or,
    clippy::option_map_unwrap_or_else,
    clippy::redundant_clone,
    clippy::result_map_unwrap_or_else,
    clippy::unnecessary_unwrap,
    clippy::unseparated_literal_suffix,
    clippy::wildcard_dependencies
)]

use actix;

#[macro_use]
extern crate serde;
use env_logger;

use serde_json;
use structopt;

use actix::prelude::*;
use actix_web::{
    http, middleware, server, App, AsyncResponder, FutureResponse, HttpRequest, HttpResponse, Json,
    State,
};
use futures::future::Future;

use diesel::r2d2::{ConnectionManager, Pool};
use juniper::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;

use failure::Error;
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

impl Message for GraphQLData {
    type Result = Result<String, Error>;
}

#[allow(missing_debug_implementations)]
pub struct GraphQLExecutor {
    schema: Arc<Schema<MyContext<DBConnection>>>,
    pool: Arc<Pool<ConnectionManager<DBConnection>>>,
}

impl GraphQLExecutor {
    fn new(
        schema: Arc<Schema<MyContext<DBConnection>>>,
        pool: Arc<Pool<ConnectionManager<DBConnection>>>,
    ) -> Self {
        Self { schema, pool }
    }
}

impl Actor for GraphQLExecutor {
    type Context = SyncContext<Self>;
}

impl Handler<GraphQLData> for GraphQLExecutor {
    type Result = Result<String, Error>;

    fn handle(&mut self, msg: GraphQLData, _: &mut Self::Context) -> Self::Result {
        let ctx = MyContext::new(self.pool.get()?);
        let res = msg.0.execute(&self.schema, &ctx);
        let res_text = serde_json::to_string(&res)?;
        Ok(res_text)
    }
}

struct AppState {
    executor: Addr<GraphQLExecutor>,
}

#[cfg_attr(feature = "clippy", allow(needless_pass_by_value))]
fn graphiql(_req: &HttpRequest<AppState>) -> Result<HttpResponse, Error> {
    let html = graphiql_source("/graphql");
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

#[cfg_attr(feature = "clippy", allow(needless_pass_by_value))]
fn graphql((st, data): (State<AppState>, Json<GraphQLData>)) -> FutureResponse<HttpResponse> {
    st.executor
        .send(data.0)
        .from_err()
        .and_then(|res| match res {
            Ok(user) => Ok(HttpResponse::Ok()
                .content_type("application/json")
                .body(user)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
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
    ::diesel_migrations::run_pending_migrations(&pool.get().expect("Failed to get db connection"))
           .expect("Failed to run migrations");

    let query = Query::<MyContext<DBConnection>>::default();
    //    let mutation = juniper::EmptyMutation::new();
    let mutation = Mutation::<MyContext<DBConnection>>::default();
    let schema = Schema::new(query, mutation);

    let sys = actix::System::new("wundergraph-example");

    let schema = Arc::new(schema);
    let pool = Arc::new(pool);
    let addr = SyncArbiter::start(3, move || {
        GraphQLExecutor::new(schema.clone(), pool.clone())
    });

    let url = opt.socket;

    // Start http server
    server::new(move || {
        App::with_state(AppState {
            executor: addr.clone(),
        })
        // enable logger
        .middleware(middleware::Logger::default())
        .resource("/graphql", |r| r.method(http::Method::POST).with(graphql))
        .resource("/graphql", |r| r.method(http::Method::GET).with(graphql))
        .resource("/graphiql", |r| r.method(http::Method::GET).h(graphiql))
        .default_resource(|r| {
            r.get().f(|_| {
                HttpResponse::Found()
                    .header("location", "/graphiql")
                    .finish()
            })
        })
    })
    .bind(&url)
    .expect("Failed to start server")
    .start();

    println!("Started http server: http://{}", url);
    let _ = sys.run();
}
