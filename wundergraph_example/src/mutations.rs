use super::heros;
use super::species;
use super::home_worlds;
use super::friends;
use super::appears_in;
use super::Hero;
use super::Species;
use super::HomeWorld;
use super::Friend;
use super::AppearsIn;
use super::Episode;

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

wundergraph_mutation_object! {
    Mutation {
        Hero(key = i32, insert = NewHero, update = HeroChangeset),
        Species(key = i32, insert = NewSpecies, update = SpeciesChangeset),
        HomeWorld(key = i32, insert = NewHomeWorld, update = HomeWorldChangeset),
        Friend(key = (i32, i32 ), insert = NewFriend),
        AppearsIn(key = (i32, Episode), insert = NewAppearsIn),
    }
}
