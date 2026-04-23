use std::sync::Arc;

use actix_web::{
    get, post,
    web::{self, Json},
};
use blades_lib::user_data::{B64EncodedData, DungeonState, DungeonStatus};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, QueryDsl, SelectableHelper, associations::HasTable,
};
use diesel_async::{AsyncConnection, RunQueryDsl, scoped_futures::ScopedFutureExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    BladeApiError, ServerGlobal,
    json_db::JsonDbWrapper,
    models::{QuestDbEntry, QuestDbEntryDungeonStateAndInitialState},
    session::SessionLookedUpMaybe,
    util::check_permission_for_character_and_get_it,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DungeonResponseEntry {
    quest_id: Uuid,
    initial_state: B64EncodedData,
    status: DungeonStatus,
}

#[derive(Serialize)]
pub struct DungeonResponse {
    dungeons: Vec<DungeonResponseEntry>,
}

#[get("/blades.bgs.services/api/game/v1/public/characters/{character_id}/dungeons")]
pub async fn get_dungeons(
    path: web::Path<Uuid>,
    session: SessionLookedUpMaybe,
    app_state: web::Data<Arc<ServerGlobal>>,
) -> Result<Json<DungeonResponse>, BladeApiError> {
    let session = session.get_session_or_error()?;
    let character_id_normal = path.into_inner();
    let mut conn = app_state.db_pool.get().await.unwrap();

    let _ =
        check_permission_for_character_and_get_it(&mut conn, &session.session, character_id_normal)
            .await?;

    let ongoing_quest_query = {
        use crate::schema::quests::dsl::*;

        quests::table()
            .filter(
                character_id
                    .eq(character_id_normal)
                    .and(dungeon_state.is_not_null())
                    .and(initial_state.is_not_null()),
            )
            .select(QuestDbEntryDungeonStateAndInitialState::as_select())
            .load(&mut conn)
            .await?
    };

    let dungeons = ongoing_quest_query
        .into_iter()
        .map(|entry| DungeonResponseEntry {
            quest_id: entry.id,
            // unwrap is fine, as we checked in the SQL query they aren’t null
            initial_state: entry.initial_state.unwrap().0,
            status: entry.dungeon_state.unwrap().0.dungeon_status,
        })
        .collect();

    Ok(Json(DungeonResponse { dungeons }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EnterDungeonRequest {
    dungeon_instance: Option<B64EncodedData>,
    current_state: B64EncodedData,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct EnterDungeonResponse {
    dungeon_status: DungeonStatus,
}

#[post(
    "/blades.bgs.services/api/game/v1/public/characters/{character_id}/quests/{quest_id}/dungeons/current/enter"
)]
pub async fn enter_quest_dungeon(
    path: web::Path<(Uuid, Uuid)>,
    body: Json<EnterDungeonRequest>,
    session: SessionLookedUpMaybe,
    app_state: web::Data<Arc<ServerGlobal>>,
) -> Result<Json<EnterDungeonResponse>, BladeApiError> {
    let session = session.get_session_or_error()?;
    let body = body.0;
    let (character_id, quest_id) = path.into_inner();
    let mut conn = app_state.db_pool.get().await.unwrap();

    let quest = match app_state.game_data.quests.get(&quest_id) {
        Some(v) => v,
        None => todo!("error handling for non-existand quest dungeon enter"),
    };
    let dungeon_info = match quest.dungeon_info.as_ref() {
        Some(v) => v,
        None => todo!("error handling enter dungeon of quest without dungepn"),
    };

    let _ = check_permission_for_character_and_get_it(&mut conn, &session.session, character_id)
        .await?;

    conn.transaction(|mut conn| {
        async move {
            let quest_query = {
                use crate::schema::quests::dsl::*;

                quests::table()
                    .filter(id.eq(&quest_id))
                    .select(QuestDbEntry::as_select())
                    .for_update()
                    .load(&mut conn)
                    .await?
            };

            let quest = match quest_query.into_iter().next() {
                Some(v) => v,
                None => todo!("Error on accept non-started quest"),
            };

            if let Some(dungeon_instance) = body.dungeon_instance {
                // first time entering
                if quest.dungeon_state.is_some() {
                    todo!("Error on enter already entered dungeon (or handle it properly?)")
                }
                let status = DungeonStatus {
                    dungeon_settings_ids: vec![dungeon_info.dungeon_uuid],
                    revive_count: 0,
                    algorithm_version: 1,
                    current_state: body.current_state,
                    seed: 54321,
                    level: 1,
                    version: 1, //TODO: figure out where this version come from.
                };

                {
                    use crate::schema::quests::dsl::*;

                    diesel::update(quests)
                        .filter(id.eq(quest_id))
                        .set((
                            dungeon_state.eq(Some(JsonDbWrapper(DungeonState {
                                dungeon_status: status.clone(),
                            }))),
                            initial_state.eq(Some(JsonDbWrapper(dungeon_instance.clone()))),
                        ))
                        .execute(&mut conn)
                        .await
                        .unwrap();
                }

                Ok(Json(EnterDungeonResponse {
                    dungeon_status: status,
                }))
            } else {
                // we are re-entering the dungeon. Just save the progress
                let mut dungeon_state_actual = if let Some(dungeon_state) = quest.dungeon_state {
                    dungeon_state.0
                } else {
                    todo!("Properly handle dungeon state not existing");
                };
                dungeon_state_actual.dungeon_status.current_state = body.current_state;
                {
                    use crate::schema::quests::dsl::*;

                    diesel::update(quests)
                        .filter(id.eq(quest_id))
                        .set(dungeon_state.eq(Some(JsonDbWrapper(dungeon_state_actual.clone()))))
                        .execute(&mut conn)
                        .await?;
                };
                Ok(Json(EnterDungeonResponse {
                    dungeon_status: dungeon_state_actual.dungeon_status,
                }))
            }
        }
        .scope_boxed()
    })
    .await
}
