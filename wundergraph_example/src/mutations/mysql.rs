use crate::mutations::{AppearsIn, Friend, NewAppearsIn, NewFriend};
use diesel::prelude::*;
use juniper::{ExecutionResult, Executor, Selection, Value};
use wundergraph::query_builder::mutations::{HandleBatchInsert, HandleInsert};
use wundergraph::query_builder::selection::LoadingHandler;
use wundergraph::{QueryModifier, WundergraphContext};

impl<Ctx> HandleInsert<Friend, NewFriend, diesel::mysql::Mysql, Ctx> for super::friends::table
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
            diesel::insert_into(super::friends::table)
                .values((
                    super::friends::hero_id.eq(insertable.hero_id),
                    super::friends::friend_id.eq(insertable.friend_id),
                ))
                .execute(conn)?;

            let query = <Friend as LoadingHandler<diesel::mysql::Mysql, Ctx>>::build_query(
                &[],
                &look_ahead,
            )?
            .filter(
                super::friends::hero_id
                    .eq(insertable.hero_id)
                    .and(super::friends::friend_id.eq(insertable.friend_id)),
            )
            .limit(1);

            let items = Friend::load(&look_ahead, selection, executor, query)?;
            Ok(items.into_iter().next().unwrap_or(Value::Null))
        })
    }
}

impl<Ctx> HandleBatchInsert<Friend, NewFriend, diesel::mysql::Mysql, Ctx> for super::friends::table
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
                            super::friends::hero_id.eq(hero_id),
                            super::friends::friend_id.eq(friend_id),
                        )
                    })
                    .collect::<Vec<_>>();
                diesel::insert_into(super::friends::table)
                    .values(insert_values)
                    .execute(conn)?;
            }

            let mut query = <Friend as LoadingHandler<diesel::mysql::Mysql, Ctx>>::build_query(
                &[],
                &look_ahead,
            )?;

            for NewFriend { hero_id, friend_id } in insertable {
                query = query.or_filter(
                    super::friends::hero_id
                        .eq(hero_id)
                        .and(super::friends::friend_id.eq(friend_id)),
                )
            }

            let items = Friend::load(&look_ahead, selection, executor, query)?;
            Ok(Value::list(items))
        })
    }
}

impl<Ctx> HandleInsert<AppearsIn, NewAppearsIn, diesel::mysql::Mysql, Ctx>
    for super::appears_in::table
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
            diesel::insert_into(super::appears_in::table)
                .values((
                    super::appears_in::hero_id.eq(insertable.hero_id),
                    super::appears_in::episode.eq(insertable.episode),
                ))
                .execute(conn)?;

            let query = <AppearsIn as LoadingHandler<diesel::mysql::Mysql, Ctx>>::build_query(
                &[],
                &look_ahead,
            )?
            .filter(
                super::appears_in::hero_id
                    .eq(insertable.hero_id)
                    .and(super::appears_in::episode.eq(insertable.episode)),
            )
            .limit(1);

            let items = AppearsIn::load(&look_ahead, selection, executor, query)?;
            Ok(items.into_iter().next().unwrap_or(Value::Null))
        })
    }
}

impl<Ctx> HandleBatchInsert<AppearsIn, NewAppearsIn, diesel::mysql::Mysql, Ctx>
    for super::appears_in::table
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
                            super::appears_in::hero_id.eq(hero_id),
                            super::appears_in::episode.eq(episode),
                        )
                    })
                    .collect::<Vec<_>>();
                diesel::insert_into(super::appears_in::table)
                    .values(insert_values)
                    .execute(conn)?;
            }

            let mut query = <AppearsIn as LoadingHandler<diesel::mysql::Mysql, Ctx>>::build_query(
                &[],
                &look_ahead,
            )?;

            for NewAppearsIn { hero_id, episode } in insertable {
                query = query.or_filter(
                    super::appears_in::hero_id
                        .eq(hero_id)
                        .and(super::appears_in::episode.eq(episode)),
                )
            }

            let items = AppearsIn::load(&look_ahead, selection, executor, query)?;
            Ok(Value::list(items))
        })
    }
}
