extern crate criterion;
extern crate diesel;
extern crate diesel_migrations;
extern crate juniper;
extern crate serde_json;
extern crate wundergraph;
extern crate wundergraph_bench;
extern crate wundergraph_example;
#[macro_use]
extern crate lazy_static;

#[cfg(feature = "postgres")]
type DbConnection = diesel::pg::PgConnection;

#[cfg(feature = "sqlite")]
type DbConnection = diesel::sqlite::SqliteConnection;

#[cfg(not(any(feature = "postgres", feature = "sqlite")))]
compile_error!("At least one feature of \"sqlite\" or \"postgres\" needs to be enabled");

#[path = "../tests/helper.rs"]
mod helper;

use criterion::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use wundergraph_bench::Schema;

const QUERIES: &[&str] = &[
    r#"query albums_tracks_genre_all {
  Albums {
    id
    title
    tracks {
      id
      name
      genre_id {
        name
      }
    }
  }
}"#,
    r#"query albums_tracks_genre_some {
  Albums(filter: {artist_id: {id: {eq: 127}}}) {
    id
    title
    tracks {
      id
      name
      genre_id {
        name
      }
    }
  }
}"#,
    r#"query tracks_media_all {
  Tracks {
    id
    name
    media_type_id {
      name
    }
  }
}"#,
    r#"query tracks_media_some {
  Tracks (filter: {composer: {eq: "Kurt Cobain"}}){
    id
    name
    album_id {
      id
      title
    }
    media_type_id {
      name
    }
  }
}"#,
//    r#"query artists_collaboration {
//  Artists(filter: {albums: {tracks: {composer: {eq: "Ludwig van Beethoven"}}}})
//  {
//    id
//    name
//  }
//}"#,
    r#"query artistByArtistId {
  Artists(filter: {id: {eq:3}}) {
    id
    name
  }
}"#,
];

fn query(
    query: &str,
    schema: &Schema<DbConnection>,
    ctx: &PooledConnection<ConnectionManager<DbConnection>>,
) {
    let res = helper::execute_query(&schema, &ctx, query);

    assert!(res.is_ok());
}

fn bench(c: &mut Criterion) {
    let (schema, pool) = helper::get_bench_schema();
    let ctx = pool.get().unwrap();

    c.bench_function_over_inputs(
        "query",
        move |b, &&query_string| {
            b.iter(|| query(query_string, &schema, &ctx));
        },
        QUERIES,
    );
}

criterion_group!(benches, bench);
criterion_main!(benches);
