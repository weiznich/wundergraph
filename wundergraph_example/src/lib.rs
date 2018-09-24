//#![deny(warnings, missing_debug_implementations, missing_copy_implementations)]
// Clippy lints
#![cfg_attr(feature = "clippy", allow(unstable_features))]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(
    feature = "clippy",
    plugin(clippy(conf_file = "../../clippy.toml"))
)]
#![cfg_attr(
    feature = "clippy",
    allow(
        option_map_unwrap_or_else,
        option_map_unwrap_or,
        match_same_arms,
        type_complexity,
        useless_attribute
    )
)]
#![cfg_attr(
    feature = "clippy",
    warn(
        option_unwrap_used,
        result_unwrap_used,
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
extern crate indexmap;
#[macro_use]
extern crate wundergraph;
extern crate failure;

use diesel::associations::HasTable;
use diesel::backend::{Backend, UsesAnsiSavepointSyntax};
use diesel::connection::AnsiTransactionManager;
use diesel::deserialize::{self, FromSql};
use diesel::query_builder::BoxedSelectStatement;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::serialize::{self, ToSql};
use diesel::sql_types::{Bool, Integer, Nullable, SmallInt, Text};
use diesel::{Connection, Identifiable};

use juniper::LookAheadSelection;

use failure::Error;
use std::io::Write;

use wundergraph::query_helper::{HasMany, HasOne, LazyLoad};
use wundergraph::query_modifier::{BuildQueryModifier, QueryModifier};
use wundergraph::scalar::WundergraphScalarValue;
use wundergraph::WundergraphContext;

pub mod mutations;
use self::mutations::*;

#[derive(
    Debug,
    Copy,
    Clone,
    AsExpression,
    FromSqlRow,
    GraphQLEnum,
    Hash,
    Eq,
    PartialEq,
    Nameable,
    FilterValue,
    FromLookAhead,
)]
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

#[derive(
    Clone,
    Debug,
    Hash,
    Eq,
    PartialEq,
    Identifiable,
    Queryable,
    WundergraphEntity,
    WundergraphFilter,
    Copy,
    Associations,
)]
#[primary_key(hero_id, episode)]
#[belongs_to(Hero)]
#[table_name = "appears_in"]
#[wundergraph(context = "MyContext<Conn>")]
pub struct AppearsIn {
    #[wundergraph(skip)]
    hero_id: i32,
    episode: Episode,
}

#[derive(
    Clone, Debug, Queryable, Eq, PartialEq, Hash, WundergraphEntity, WundergraphFilter, Associations,
)]
#[table_name = "friends"]
#[belongs_to(Hero)]
#[wundergraph(context = "MyContext<Conn>")]
pub struct Friend {
    #[wundergraph(skip)]
    hero_id: i32,
    friend_id: HasOne<i32, Hero>,
}

// TODO: make this two impls deriveable
impl HasTable for Friend {
    type Table = friends::table;

    fn table() -> Self::Table {
        friends::table
    }
}

impl<'a> Identifiable for &'a Friend {
    type Id = (&'a i32, &'a i32);

    fn id(self) -> Self::Id {
        let friend_id = match self.friend_id {
            HasOne::Id(ref id) => id,
            HasOne::Item(ref hero) => &hero.id,
        };
        (&self.hero_id, friend_id)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TestModifier;

impl QueryModifier<<DBConnection as Connection>::Backend> for TestModifier {
    type Entity = HomeWorld;

    fn modify_query<'a>(
        &self,
        final_query: BoxedSelectStatement<
            'a,
            (Integer, Nullable<Text>, Nullable<Bool>),
            home_worlds::table,
            <DBConnection as Connection>::Backend,
        >,
        _selection: &LookAheadSelection<WundergraphScalarValue>,
    ) -> Result<
        BoxedSelectStatement<
            'a,
            (Integer, Nullable<Text>, Nullable<Bool>),
            home_worlds::table,
            <DBConnection as Connection>::Backend,
        >,
        Error,
    > {
        Ok(final_query)
    }
}

impl BuildQueryModifier<HomeWorld> for TestModifier {
    type Context = MyContext<DBConnection>;
    fn from_ctx(_ctx: &Self::Context) -> Result<Self, Error> {
        Ok(TestModifier)
    }
}

#[derive(
    Clone, Debug, Hash, Eq, PartialEq, Identifiable, Queryable, WundergraphEntity, WundergraphFilter,
)]
#[table_name = "home_worlds"]
#[wundergraph(query_modifier = "TestModifier", context = "MyContext<Conn>")]
/// A world where a hero was born
pub struct HomeWorld {
    /// Internal id of a world
    id: i32,
    /// The name of a world
    name: LazyLoad<String>,
    #[wundergraph(is_nullable_reference = "true")]
    /// All heros of a given world
    heros: HasMany<Hero>,
}

#[allow(deprecated)]
mod hero {
    use super::*;
    #[derive(
        Clone,
        Debug,
        Identifiable,
        Hash,
        Eq,
        PartialEq,
        Queryable,
        WundergraphEntity,
        WundergraphFilter,
        Associations,
    )]
    #[table_name = "heros"]
    #[belongs_to(Species, foreign_key = "species")]
    #[belongs_to(HomeWorld, foreign_key = "home_world")]
    #[wundergraph(context = "MyContext<Conn>")]
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
        home_world: HasOne<Option<i32>, Option<HomeWorld>>,
        /// Episodes a hero appears in
        appears_in: HasMany<AppearsIn>,
        /// List of friends of the current hero
        friends: HasMany<Friend>,
    }
}
pub use self::hero::{Hero, HeroFilter};

#[derive(
    Clone, Debug, Identifiable, Hash, Eq, PartialEq, Queryable, WundergraphEntity, WundergraphFilter,
)]
#[table_name = "species"]
#[wundergraph(context = "MyContext<Conn>")]
/// A species
pub struct Species {
    /// Internal id of a species
    id: i32,
    /// The name of a species
    name: String,
    /// A list of heros for a species
    heros: HasMany<Hero>,
}

wundergraph_query_object!{
    Query(context = MyContext<Conn>) {
        Heros(Hero, filter = HeroFilter),
        Species(Species, filter = SpeciesFilter),
        HomeWorlds(HomeWorld, filter = HomeWorldFilter),
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

impl<Conn> WundergraphContext<Conn::Backend> for MyContext<Conn>
where
    Conn: Connection<TransactionManager = AnsiTransactionManager> + 'static,
    Conn::Backend: UsesAnsiSavepointSyntax,
{
    type Connection = diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<Conn>>;

    fn get_connection(&self) -> &Self::Connection {
        &self.conn
    }
}

#[cfg(feature = "postgres")]
pub type DBConnection = ::diesel::PgConnection;

#[cfg(feature = "sqlite")]
pub type DBConnection = ::diesel::SqliteConnection;

pub type Schema<Conn> = juniper::RootNode<
    'static,
    Query<Pool<ConnectionManager<Conn>>>,
    Mutation<Pool<ConnectionManager<Conn>>>,
    WundergraphScalarValue,
>;
