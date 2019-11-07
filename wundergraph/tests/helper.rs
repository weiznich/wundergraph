#![allow(dead_code)]
use crate::DbConnection;
use diesel::r2d2::CustomizeConnection;
use diesel::r2d2::*;
use diesel::Connection;
use juniper::*;
use serde_json::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use wundergraph::scalar::WundergraphScalarValue;
use wundergraph_bench::api::{Mutation as BenchMutation, Query as BenchQuery};
use wundergraph_bench::Schema as BenchSchema;
use wundergraph_example::mutations::Mutation as ExampleMutation;
use wundergraph_example::{MyContext, Query as ExampleQuery, Schema as ExampleSchema};

lazy_static! {
    static ref MIGRATION_LOCK: Mutex<()> = Mutex::new(());
}

#[derive(Debug)]
struct TestTransaction;

impl CustomizeConnection<DbConnection, ::diesel::r2d2::Error> for TestTransaction {
    fn on_acquire(
        &self,
        conn: &mut DbConnection,
    ) -> ::std::result::Result<(), ::diesel::r2d2::Error> {
        conn.begin_test_transaction().unwrap();
        Ok(())
    }
}

pub fn get_example_schema() -> (
    ExampleSchema<MyContext<DbConnection>>,
    Pool<ConnectionManager<DbConnection>>,
) {
    let db_url = ::std::env::var("DATABASE_URL")
        .expect("You need to set `DATABASE_URL` as environment variable");
    {
        let _migration_lock = MIGRATION_LOCK.lock();
        let conn = DbConnection::establish(&db_url).unwrap();
        run_migrations(&conn, "wundergraph_example");
    }
    let manager = ConnectionManager::<DbConnection>::new(db_url);
    let pool = Pool::builder()
        .max_size(1)
        .connection_customizer(Box::new(TestTransaction))
        .build(manager)
        .expect("Failed to init pool");

    let query = ExampleQuery::<MyContext<DbConnection>>::default();
    let mutation = ExampleMutation::<MyContext<DbConnection>>::default();
    (ExampleSchema::new(query, mutation), pool)
}

pub fn get_bench_schema() -> (
    BenchSchema<DbConnection>,
    Pool<ConnectionManager<DbConnection>>,
) {
    let db_url = ::std::env::var("DATABASE_URL")
        .expect("You need to set `DATABASE_URL` as environment variable");
    {
        let conn = DbConnection::establish(&db_url).unwrap();
        run_migrations(&conn, "wundergraph_bench");
    }
    let manager = ConnectionManager::<DbConnection>::new(db_url);
    let pool = Pool::builder()
        .max_size(1)
        .connection_customizer(Box::new(TestTransaction))
        .build(manager)
        .expect("Failed to init pool");

    run_migrations(&*pool.get().unwrap(), "wundergraph_bench");
    let query = BenchQuery::default();
    let mutation = BenchMutation::default();
    (BenchSchema::new(query, mutation), pool)
}

fn run_migrations(conn: &DbConnection, which: &str) {
    let mut migration_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    migration_path.push("..");
    migration_path.push(which);
    migration_path.push("migrations");
    if cfg!(feature = "postgres") {
        migration_path.push("pg");
    } else if cfg!(feature = "sqlite") {
        migration_path.push("sqlite");
    }
    let pending_migrations =
        ::diesel_migrations::mark_migrations_in_directory(conn, &migration_path)
            .unwrap()
            .into_iter()
            .filter_map(|(migration, run)| if run { None } else { Some(migration) });

    ::diesel_migrations::run_migrations(conn, pending_migrations, &mut ::std::io::stdout())
        .unwrap();
}

#[derive(Debug)]
pub struct WundergraphResponse<'a>(
    ::std::result::Result<
        (
            ::juniper::Value<WundergraphScalarValue>,
            Vec<ExecutionError<WundergraphScalarValue>>,
        ),
        GraphQLError<'a>,
    >,
);

pub fn execute_query<'a, Q, M, C>(
    schema: &'a RootNode<Q, M, WundergraphScalarValue>,
    ctx: &C,
    query: &'a str,
) -> WundergraphResponse<'a>
where
    Q: GraphQLType<WundergraphScalarValue, Context = C>,
    M: GraphQLType<WundergraphScalarValue, Context = C>,
{
    execute_query_with_variables(schema, ctx, query, &[])
}

pub fn execute_query_with_variables<'a, Q, M, C>(
    schema: &'a RootNode<Q, M, WundergraphScalarValue>,
    ctx: &C,
    query: &'a str,
    vars: &[(&str, ::serde_json::Value)],
) -> WundergraphResponse<'a>
where
    Q: GraphQLType<WundergraphScalarValue, Context = C>,
    M: GraphQLType<WundergraphScalarValue, Context = C>,
{
    let vars = vars
        .into_iter()
        .map(|(ref k, v)| {
            let v = to_string(&v).unwrap();
            let var = from_str(&v).unwrap();
            ((*k).to_owned(), var)
        })
        .collect::<HashMap<String, _>>();

    WundergraphResponse(execute(query, None, schema, &vars, ctx))
}

impl<'a> WundergraphResponse<'a> {
    pub fn is_ok(&self) -> bool {
        self.0.is_ok()
    }

    pub fn is_err(&self) -> bool {
        self.0.is_err()
    }

    pub fn as_json(self) -> ::serde_json::Value {
        ::serde_json::to_value(self.0.unwrap()).unwrap()
    }
}
