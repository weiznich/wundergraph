#[macro_use] extern crate diesel;

use actix_web::web::Data;
use actix_web::web::Json;
use actix_web::middleware;
use actix_web::web;
use actix_web::App;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use diesel::{conn};
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::r2d2::PooledConnection;
use diesel::r2d2::CustomizeConnection;
use diesel::Connection;
use diesel::connection::SimpleConnection;
use juniper::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use serde::Deserialize;
use serde::Serialize;
use std::sync::Arc;
use wundergraph::scalar::WundergraphScalarValue;

pub mod api;

pub type Schema<Connection> = juniper::RootNode<
    'static,
    self::api::Query<PooledConnection<ConnectionManager<Connection>>>,
    self::api::Mutation<PooledConnection<ConnectionManager<Connection>>>,
    WundergraphScalarValue,
>;

// actix integration stuff
#[derive(Serialize, Deserialize, Debug)]
pub struct GraphQLData(GraphQLRequest<WundergraphScalarValue>);

#[derive(Clone)]
struct AppState {{
    schema: Arc<Schema<{conn}>>,
    pool: Arc<Pool<ConnectionManager<{conn}>>>,
}}

fn graphiql() -> HttpResponse {{
    let html = graphiql_source("/graphql");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}}

fn graphql(
    Json(GraphQLData(data)): Json<GraphQLData>,
    st: Data<AppState>,
) -> Result<HttpResponse, failure::Error> {{
    let ctx = st.get_ref().pool.get()?;
    let res = data.execute(&st.get_ref().schema, &ctx);
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&res)?))
}}

#[derive(Debug)]
struct ConnectionHandler;

impl<E> CustomizeConnection<{conn}, E> for ConnectionHandler {{
    fn on_acquire(&self, conn: &mut {conn}) -> Result<(), E> {{
        Ok(conn.begin_test_transaction().unwrap())
    }}
}}

fn main() {{
    let manager = ConnectionManager::<{conn}>::new("{db_url}");
    let pool = Pool::builder()
        .max_size(1)
        .connection_customizer(Box::new(ConnectionHandler))
        .build(manager)
        .expect("Failed to init pool");
    {{
        let conn = pool.get().unwrap();
        conn.batch_execute("{migrations}").unwrap();
    }}

    let query = self::api::Query::default();
    let mutation = self::api::Mutation::default();
    let schema = Schema::new(query, mutation);

    let schema = Arc::new(schema);
    let pool = Arc::new(pool);
    let data = AppState {{ schema, pool }};

    let url = "{listen_url}";

    println!("Started http server: http://{{}}", url);

    HttpServer::new(move || {{
        App::new()
            .data(data.clone())
            .wrap(middleware::Logger::default())
            .route("/graphql", web::get().to(graphql))
            .route("/graphql", web::post().to(graphql))
            .route("/graphiql", web::get().to(graphiql))
            .default_service(web::route().to(|| {{
                HttpResponse::Found()
                    .header("location", "/graphiql")
                    .finish()
            }}))
    }})
    .bind(&url)
    .expect("Failed to start server")
    .run()
    .unwrap();
}}
