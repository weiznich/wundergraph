use super::appears_in;
use super::friends;
use super::heros;
use super::home_worlds;
use super::species;
use super::AppearsIn;
use super::Episode;
use super::Friend;
use super::Hero;
use super::HomeWorld;
use super::Species;
use juniper::*;

#[derive(Insertable, GraphQLInputObject, Clone, Debug)]
#[table_name = "heros"]
pub struct NewHero {
    name: String,
    hair_color: Option<String>,
    species: i32,
    home_world: Option<i32>,
}

#[derive(AsChangeset, GraphQLInputObject, Identifiable, Debug)]
#[table_name = "heros"]
pub struct HeroChangeset {
    id: i32,
    name: Option<String>,
    hair_color: Option<String>,
    species: Option<i32>,
    home_world: Option<i32>,
}

#[derive(Insertable, GraphQLInputObject, Clone, Debug)]
#[table_name = "species"]
pub struct NewSpecies {
    name: String,
}

#[derive(AsChangeset, GraphQLInputObject, Identifiable, Debug)]
#[table_name = "species"]
pub struct SpeciesChangeset {
    id: i32,
    name: Option<String>,
}

#[derive(Insertable, GraphQLInputObject, Debug)]
#[table_name = "home_worlds"]
pub struct NewHomeWorld {
    name: String,
}

#[derive(AsChangeset, GraphQLInputObject, Identifiable, Debug)]
#[table_name = "home_worlds"]
pub struct HomeWorldChangeset {
    id: i32,
    name: Option<String>,
}

#[derive(Insertable, GraphQLInputObject, Debug, Copy, Clone)]
#[table_name = "friends"]
pub struct NewFriend {
    hero_id: i32,
    friend_id: i32,
}

#[derive(Insertable, GraphQLInputObject, Debug, Copy, Clone)]
#[table_name = "appears_in"]
pub struct NewAppearsIn {
    hero_id: i32,
    episode: Episode,
}

wundergraph::mutation_object! {
    /// Global mutation object for the schema
    Mutation {
        Hero(insert = NewHero, update = HeroChangeset,),
        Species(insert = NewSpecies, update = SpeciesChangeset,),
        HomeWorld(insert = NewHomeWorld, update = HomeWorldChangeset,),
        Friend( insert = NewFriend, update = false),
        AppearsIn(insert = NewAppearsIn, ),
    }
}
