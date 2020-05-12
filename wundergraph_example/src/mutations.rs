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
use diesel::prelude::*;
use juniper::*;
use wundergraph::query_builder::mutations::{HandleBatchInsert, HandleInsert};
use wundergraph::query_builder::selection::LoadingHandler;
use wundergraph::{QueryModifier, WundergraphContext};

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

#[cfg_attr(not(feature = "mysql"), derive(Insertable))]
#[derive(GraphQLInputObject, Debug, Copy, Clone)]
#[cfg_attr(not(feature = "mysql"), table_name = "friends")]
pub struct NewFriend {
    hero_id: i32,
    friend_id: i32,
}

impl<Ctx> HandleInsert<Friend, NewFriend, diesel::mysql::Mysql, Ctx> for friends::table
where
    Ctx: WundergraphContext + QueryModifier<Friend, diesel::mysql::Mysql> + 'static,
    Ctx::Connection: Connection<Backend = diesel::mysql::Mysql>,
{
    fn handle_insert(
        selection: Option<&'_ [Selection<'_, wundergraph::scalar::WundergraphScalarValue>]>,
        executor: &Executor<'_, Ctx, wundergraph::scalar::WundergraphScalarValue>,
        insertable: NewFriend,
    ) -> ExecutionResult<wundergraph::scalar::WundergraphScalarValue> {
        let ctx = executor.context();
        let conn = ctx.get_connection();
        let look_ahead = executor.look_ahead();

        conn.transaction(|| {
            diesel::insert_into(friends::table)
                .values((
                    friends::hero_id.eq(insertable.hero_id),
                    friends::friend_id.eq(insertable.friend_id),
                ))
                .execute(conn)?;

            let query = <Friend as LoadingHandler<diesel::mysql::Mysql, Ctx>>::build_query(
                &[],
                &look_ahead,
            )?
            .filter(
                friends::hero_id
                    .eq(insertable.hero_id)
                    .and(friends::friend_id.eq(insertable.friend_id)),
            )
            .limit(1);

            let items = Friend::load(&look_ahead, selection, executor, query)?;
            Ok(items.into_iter().next().unwrap_or(Value::Null))
        })
    }
}

impl<Ctx> HandleBatchInsert<Friend, NewFriend, diesel::mysql::Mysql, Ctx> for friends::table
where
    Ctx: WundergraphContext + QueryModifier<Friend, diesel::mysql::Mysql> + 'static,
    Ctx::Connection: Connection<Backend = diesel::mysql::Mysql>,
{
    fn handle_batch_insert(
        selection: Option<&'_ [Selection<'_, wundergraph::scalar::WundergraphScalarValue>]>,
        executor: &Executor<'_, Ctx, wundergraph::scalar::WundergraphScalarValue>,
        insertable: Vec<NewFriend>,
    ) -> ExecutionResult<wundergraph::scalar::WundergraphScalarValue> {
        let ctx = executor.context();
        let conn = ctx.get_connection();
        let look_ahead = executor.look_ahead();

        conn.transaction(|| {
            {
                let insert_values = insertable
                    .iter()
                    .map(|NewFriend { hero_id, friend_id }| {
                        (
                            friends::hero_id.eq(hero_id),
                            friends::friend_id.eq(friend_id),
                        )
                    })
                    .collect::<Vec<_>>();
                diesel::insert_into(friends::table)
                    .values(insert_values)
                    .execute(conn)?;
            }

            let mut query = <Friend as LoadingHandler<diesel::mysql::Mysql, Ctx>>::build_query(
                &[],
                &look_ahead,
            )?;

            for NewFriend { hero_id, friend_id } in insertable {
                query = query.or_filter(
                    friends::hero_id
                        .eq(hero_id)
                        .and(friends::friend_id.eq(friend_id)),
                )
            }

            let items = Friend::load(&look_ahead, selection, executor, query)?;
            Ok(Value::list(items))
        })
    }
}

#[cfg_attr(not(feature = "mysql"), derive(Insertable))]
#[derive(GraphQLInputObject, Debug, Copy, Clone)]
#[cfg_attr(not(feature = "mysql"), table_name = "appears_in")]
pub struct NewAppearsIn {
    hero_id: i32,
    episode: Episode,
}

impl<Ctx> HandleInsert<AppearsIn, NewAppearsIn, diesel::mysql::Mysql, Ctx> for appears_in::table
where
    Ctx: WundergraphContext + QueryModifier<AppearsIn, diesel::mysql::Mysql> + 'static,
    Ctx::Connection: Connection<Backend = diesel::mysql::Mysql>,
{
    fn handle_insert(
        selection: Option<&'_ [Selection<'_, wundergraph::scalar::WundergraphScalarValue>]>,
        executor: &Executor<'_, Ctx, wundergraph::scalar::WundergraphScalarValue>,
        insertable: NewAppearsIn,
    ) -> ExecutionResult<wundergraph::scalar::WundergraphScalarValue> {
        let ctx = executor.context();
        let conn = ctx.get_connection();
        let look_ahead = executor.look_ahead();

        conn.transaction(|| {
            diesel::insert_into(appears_in::table)
                .values((
                    appears_in::hero_id.eq(insertable.hero_id),
                    appears_in::episode.eq(insertable.episode),
                ))
                .execute(conn)?;

            let query = <AppearsIn as LoadingHandler<diesel::mysql::Mysql, Ctx>>::build_query(
                &[],
                &look_ahead,
            )?
            .filter(
                appears_in::hero_id
                    .eq(insertable.hero_id)
                    .and(appears_in::episode.eq(insertable.episode)),
            )
            .limit(1);

            let items = AppearsIn::load(&look_ahead, selection, executor, query)?;
            Ok(items.into_iter().next().unwrap_or(Value::Null))
        })
    }
}

impl<Ctx> HandleBatchInsert<AppearsIn, NewAppearsIn, diesel::mysql::Mysql, Ctx>
    for appears_in::table
where
    Ctx: WundergraphContext + QueryModifier<AppearsIn, diesel::mysql::Mysql> + 'static,
    Ctx::Connection: Connection<Backend = diesel::mysql::Mysql>,
{
    fn handle_batch_insert(
        selection: Option<&'_ [Selection<'_, wundergraph::scalar::WundergraphScalarValue>]>,
        executor: &Executor<'_, Ctx, wundergraph::scalar::WundergraphScalarValue>,
        insertable: Vec<NewAppearsIn>,
    ) -> ExecutionResult<wundergraph::scalar::WundergraphScalarValue> {
        let ctx = executor.context();
        let conn = ctx.get_connection();
        let look_ahead = executor.look_ahead();

        conn.transaction(|| {
            {
                let insert_values = insertable
                    .iter()
                    .map(|NewAppearsIn { hero_id, episode }| {
                        (
                            appears_in::hero_id.eq(hero_id),
                            appears_in::episode.eq(episode),
                        )
                    })
                    .collect::<Vec<_>>();
                diesel::insert_into(appears_in::table)
                    .values(insert_values)
                    .execute(conn)?;
            }

            let mut query = <AppearsIn as LoadingHandler<diesel::mysql::Mysql, Ctx>>::build_query(
                &[],
                &look_ahead,
            )?;

            for NewAppearsIn { hero_id, episode } in insertable {
                query = query.or_filter(
                    appears_in::hero_id
                        .eq(hero_id)
                        .and(appears_in::episode.eq(episode)),
                )
            }

            let items = AppearsIn::load(&look_ahead, selection, executor, query)?;
            Ok(Value::list(items))
        })
    }
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
