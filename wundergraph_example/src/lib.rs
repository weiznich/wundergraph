#![feature(trace_macros)]
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
        print_stdout,
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

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate juniper;
extern crate failure;
extern crate indexmap;
extern crate wundergraph;

use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::serialize::{self, ToSql};
use diesel::sql_types::SmallInt;
use diesel::{Connection, Identifiable};
use failure::Error;
use juniper::LookAheadSelection;
use std::io::Write;
use wundergraph::query_helper::{HasMany, HasOne};
use wundergraph::scalar::WundergraphScalarValue;
use wundergraph::WundergraphContext;
use wundergraph::{BoxedQuery, LoadingHandler, QueryModifier, ApplyOffset};
use wundergraph::{WundergraphEntity, WundergraphValue};

pub mod mutations;
use self::mutations::*;

#[derive(Debug, Copy, Clone, AsExpression, FromSqlRow, GraphQLEnum, WundergraphValue)]
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

table! {
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

#[derive(Clone, Debug, Identifiable, Queryable, WundergraphEntity)]
#[primary_key(hero_id)]
#[table_name = "appears_in"]
pub struct AppearsIn {
    hero_id: HasOne<i32, Hero>,
    episode: Episode,
}

#[derive(Clone, Debug, Queryable, Eq, PartialEq, Hash, WundergraphEntity, Identifiable)]
#[table_name = "friends"]
#[primary_key(hero_id)]
pub struct Friend {
    #[wundergraph(skip)]
    hero_id: i32,
    friend_id: HasOne<i32, Hero>,
}

#[derive(Clone, Debug, Identifiable, WundergraphEntity)]
#[table_name = "home_worlds"]
/// A world where a hero was born
pub struct HomeWorld {
    /// Internal id of a world
    id: i32,
    /// The name of a world
    name: String,
    /// All heros of a given world
    heros: HasMany<Hero>,
}

#[allow(deprecated)]
mod hero {
    use super::*;
    #[derive(Clone, Debug, Identifiable, Queryable, WundergraphEntity)]
    #[table_name = "heros"]
    /// A hero from Star Wars
    pub struct Hero {
        /// Internal id of a hero
        pub(super) id: i32,
        /// The name of a hero
        #[wundergraph(graphql_name = "heroName")]
        #[column_name = "name"]
        something: String,
        /// The hair color of a hero
        #[deprecated(note = "Hair color should not be used because of unsafe things")]
        hair_color: Option<String>,
        /// Which species a hero belongs to
        species: HasOne<i32, Species>,
        /// On which world a hero was born
        home_world: Option<HasOne<i32, HomeWorld>>,
        //        /// Episodes a hero appears in
        appears_in: HasMany<AppearsIn>,
        /// List of friends of the current hero
        friends: HasMany<Friend>,
    }
}
pub use self::hero::Hero;

#[derive(Clone, Debug, Identifiable, WundergraphEntity)]
#[table_name = "species"]
/// A species
pub struct Species {
    /// Internal id of a species
    id: i32,
    /// The name of a species
    name: String,
    /// A list of heros for a species
    heros: HasMany<Hero>,
}

wundergraph::query_object! {
    Query {
        Hero,
        Species,
        HomeWorld,
    }
}

#[derive(Debug)]
pub struct MyContext<Conn>
where
    Conn: Connection + 'static,
{
    conn: PooledConnection<ConnectionManager<Conn>>,
}

impl<Conn> MyContext<Conn>
where
    Conn: Connection + 'static,
{
    pub fn new(conn: PooledConnection<ConnectionManager<Conn>>) -> Self {
        Self { conn }
    }
}

impl<T, C, DB> QueryModifier<T, DB> for MyContext<C>
where
    C: Connection<Backend = DB>,
    DB: Backend + ApplyOffset + 'static,
    T: LoadingHandler<DB, Self>,
    Self: WundergraphContext,
    Self::Connection: Connection<Backend = DB>,
{
    fn modify_query<'a>(
        &self,
        _select: &LookAheadSelection<'_, WundergraphScalarValue>,
        query: BoxedQuery<'a, T, DB, Self>,
    ) -> Result<BoxedQuery<'a, T, DB, Self>, Error> {
        dbg!(T::TYPE_NAME);
        match T::TYPE_NAME {
//            "Heros" => Err(Error::from_boxed_compat(String::from("Is user").into())),
            _ => Ok(query),
        }
    }
}

impl WundergraphContext for MyContext<DBConnection> {
    type Connection = diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<DBConnection>>;

    fn get_connection(&self) -> &Self::Connection {
        &self.conn
    }
}

#[cfg(feature = "postgres")]
pub type DBConnection = ::diesel::PgConnection;

#[cfg(feature = "sqlite")]
pub type DBConnection = ::diesel::SqliteConnection;

pub type DbBackend = <DBConnection as Connection>::Backend;

pub type Schema<Ctx> =
    juniper::RootNode<'static, Query<Ctx>, Mutation<Ctx>, WundergraphScalarValue>;
