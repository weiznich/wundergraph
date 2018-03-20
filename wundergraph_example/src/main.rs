#![feature(plugin)]
#![plugin(rocket_codegen)]
#![deny(warnings, missing_debug_implementations, missing_copy_implementations)]
// Clippy lints
#![cfg_attr(feature = "clippy", allow(unstable_features))]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy(conf_file = "../../clippy.toml")))]
#![cfg_attr(feature = "clippy",
            allow(option_map_unwrap_or_else, option_map_unwrap_or, match_same_arms,
                  type_complexity))]
#![cfg_attr(feature = "clippy",
            warn(option_unwrap_used, result_unwrap_used, wrong_pub_self_convention, mut_mut,
                 non_ascii_literal, similar_names, unicode_not_nfc, enum_glob_use, if_not_else,
                 items_after_statements, used_underscore_binding))]

#[macro_use]
extern crate diesel;
extern crate diesel_migrations;
#[macro_use]
extern crate juniper;
extern crate juniper_rocket;
extern crate ordermap;
extern crate rocket;
#[macro_use]
extern crate wundergraph;

use rocket::response::content;
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Outcome, Request, State};

use diesel::serialize::{self, ToSql};
use diesel::deserialize::{self, FromSql};
use diesel::backend::Backend;
use diesel::sql_types::SmallInt;
use diesel::associations::BelongsTo;
use diesel::r2d2::{ConnectionManager, Pool};

use std::io::Write;

use wundergraph::query_helper::{HasMany, HasOne};

mod mutations;
use self::mutations::*;

#[derive(Debug, Copy, Clone, AsExpression, FromSqlRow, GraphQLEnum, Hash, Eq, PartialEq, Nameable,
         FilterValue, FromLookAhead)]
#[sql_type = "SmallInt"]
pub enum Episode {
    NEWHOPE = 1,
    EMPIRE = 2,
    JEDI = 3,
}

impl<DB> ToSql<SmallInt, DB> for Episode
where
    DB: Backend,
    i16: ToSql<SmallInt, DB>,
{
    fn to_sql<W: Write>(&self, out: &mut serialize::Output<W, DB>) -> serialize::Result {
        (*self as i16).to_sql(out)
    }
}

impl<DB> FromSql<SmallInt, DB> for Episode
where
    DB: Backend,
    i16: FromSql<SmallInt, DB>,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        let value = i16::from_sql(bytes)?;
        Ok(match value {
            1 => Episode::NEWHOPE,
            2 => Episode::EMPIRE,
            3 => Episode::JEDI,
            _ => unreachable!(),
        })
    }
}

table! {
    heros {
        id -> Integer,
        name -> Text,
        hair_color -> Nullable<Text>,
        species -> Integer,
        home_world -> Nullable<Integer>,
    }
}

table!{
    friends(hero_id, friend_id) {
        hero_id -> Integer,
        friend_id -> Integer,
    }
}

table! {
    species {
        id -> Integer,
        name -> Text,
    }
}

table! {
    home_worlds {
        id -> Integer,
        name -> Text,
    }
}

table! {
    appears_in (hero_id, episode) {
        hero_id -> Integer,
        episode -> SmallInt,
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Identifiable, Queryable, WundergraphEntity,
         WundergraphFilter, Copy)]
#[primary_key(hero_id, episode)]
#[table_name = "appears_in"]
pub struct AppearsIn {
    #[wundergraph(skip)]
    hero_id: i32,
    episode: Episode,
}

impl BelongsTo<Hero> for AppearsIn {
    type ForeignKey = i32;
    type ForeignKeyColumn = appears_in::hero_id;

    fn foreign_key(&self) -> Option<&Self::ForeignKey> {
        Some(&self.hero_id)
    }

    fn foreign_key_column() -> Self::ForeignKeyColumn {
        appears_in::hero_id
    }
}

#[derive(Clone, Debug, Queryable, Eq, PartialEq, Hash, Identifiable, WundergraphEntity,
         WundergraphFilter)]
#[table_name = "friends"]
#[primary_key(hero_id)]
pub struct Friend {
    #[wundergraph(skip)]
    hero_id: i32,
    #[wundergraph(remote_table = "heros")]
    friend_id: HasOne<i32, Hero>,
}

impl BelongsTo<Hero> for Friend {
    type ForeignKey = i32;
    type ForeignKeyColumn = friends::hero_id;

    fn foreign_key(&self) -> Option<&Self::ForeignKey> {
        Some(&self.hero_id)
    }

    fn foreign_key_column() -> Self::ForeignKeyColumn {
        friends::hero_id
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Identifiable, Queryable, WundergraphEntity,
         WundergraphFilter)]
#[table_name = "home_worlds"]
pub struct HomeWorld {
    id: i32,
    name: String,
    #[diesel(default)]
    #[wundergraph(remote_table = "heros", is_nullable_reference = "true",
                  foreign_key = "home_world")]
    heros: HasMany<Hero>,
}

#[derive(Clone, Debug, Identifiable, Hash, Eq, PartialEq, Queryable, WundergraphEntity,
         WundergraphFilter)]
#[table_name = "heros"]
pub struct Hero {
    id: i32,
    name: String,
    hair_color: Option<String>,
    #[wundergraph(remote_table = "species")]
    species: HasOne<i32, Species>,
    #[wundergraph(remote_table = "home_worlds")]
    home_world: HasOne<Option<i32>, Option<HomeWorld>>,
    #[diesel(default)]
    #[wundergraph(remote_table = "appears_in", foreign_key = "hero_id")]
    appears_in: HasMany<AppearsIn>,
    #[diesel(default)]
    #[wundergraph(remote_table = "friends", foreign_key = "hero_id")]
    friends: HasMany<Friend>,
}

impl BelongsTo<Species> for Hero {
    type ForeignKey = i32;
    type ForeignKeyColumn = heros::species;

    fn foreign_key(&self) -> Option<&Self::ForeignKey> {
        match self.species {
            HasOne::Id(ref i) => Some(i),
            HasOne::Item(ref i) => Some(&i.id),
        }
    }

    fn foreign_key_column() -> Self::ForeignKeyColumn {
        heros::species
    }
}

impl BelongsTo<HomeWorld> for Hero {
    type ForeignKey = i32;
    type ForeignKeyColumn = heros::home_world;

    fn foreign_key(&self) -> Option<&Self::ForeignKey> {
        match self.home_world {
            HasOne::Id(ref i) => i.as_ref(),
            HasOne::Item(ref i) => i.as_ref().map(|i| &i.id),
        }
    }

    fn foreign_key_column() -> Self::ForeignKeyColumn {
        heros::home_world
    }
}

#[derive(Clone, Debug, Identifiable, Hash, Eq, PartialEq, Queryable, WundergraphEntity,
         WundergraphFilter)]
#[table_name = "species"]
pub struct Species {
    id: i32,
    name: String,
    #[diesel(default)]
    #[wundergraph(remote_table = "heros", foreign_key = "species")]
    heros: HasMany<Hero>,
}

wundergraph_query_object!{
    Query {
        Heros(Hero, filter = HeroFilter),
        Species(Species, filter = SpeciesFilter),
        HomeWorlds(HomeWorld, filter = HomeWorldFilter),
    }
}

// rocket integration stuff
#[derive(Debug)]
pub struct DbConn<Conn: diesel::Connection + 'static>(
    diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<Conn>>,
);

impl<'a, 'r, Conn: diesel::Connection + 'static> FromRequest<'a, 'r> for DbConn<Conn> {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<DbConn<Conn>, ()> {
        let pool =
            request.guard::<State<diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<Conn>>>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(DbConn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}

impl<Conn> DbConn<Conn>
where
    Conn: diesel::Connection,
{
    fn get_connection(
        &self,
    ) -> &diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<Conn>> {
        &self.0
    }
}

#[get("/")]
fn graphiql() -> content::Html<String> {
    juniper_rocket::graphiql_source("/graphql")
}

//type DBConnection = ::diesel::PgConnection;
type DBConnection = ::diesel::SqliteConnection;

#[get("/graphql?<request>")]
#[cfg_attr(feature = "clippy", allow(needless_pass_by_value))]
fn get_graphql_handler(
    request: juniper_rocket::GraphQLRequest,
    schema: State<Schema<DBConnection>>,
    conn: DbConn<DBConnection>,
) -> juniper_rocket::GraphQLResponse {
    request.execute(&schema, conn.get_connection())
}

#[post("/graphql", data = "<request>")]
#[cfg_attr(feature = "clippy", allow(needless_pass_by_value))]
fn post_graphql_handler(
    request: juniper_rocket::GraphQLRequest,
    schema: State<Schema<DBConnection>>,
    conn: DbConn<DBConnection>,
) -> juniper_rocket::GraphQLResponse {
    request.execute(&schema, conn.get_connection())
}

type Schema<Conn> = juniper::RootNode<
    'static,
    Query<Pool<ConnectionManager<Conn>>>,
    Mutation<Pool<ConnectionManager<Conn>>>,
>;

fn main() {
    let manager = ConnectionManager::<DBConnection>::new(":memory:");
    let pool = Pool::builder()
        .max_size(1)
        .build(manager)
        .expect("Failed to init pool");
    ::diesel_migrations::run_pending_migrations(&pool.get().expect("Failed to get db connection"))
        .expect("Failed to run migrations");

    let query = Query::<Pool<ConnectionManager<DBConnection>>>::default();
    let mutation = Mutation::<Pool<ConnectionManager<DBConnection>>>::default();
    let schema = Schema::new(query, mutation);

    rocket::ignite()
        .manage(schema)
        .manage(pool)
        .mount(
            "/",
            routes![graphiql, get_graphql_handler, post_graphql_handler],
        )
        .launch();
}
