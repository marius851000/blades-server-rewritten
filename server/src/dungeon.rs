use std::sync::Arc;

use actix_web::{
    get, post,
    web::{self, Json},
};
use blades_lib::user_data::{B64EncodedData, DungeonStatus};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper, associations::HasTable};
use diesel_async::{AsyncConnection, RunQueryDsl, scoped_futures::ScopedFutureExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    BladeApiError, ServerGlobal, json_db::JsonDbWrapper, models::QuestDbEntry,
    session::SessionLookedUpMaybe, util::check_permission_for_character_and_get_it,
};

#[derive(Serialize)]
pub struct DungeonResponse {
    dungeons: Vec<()>,
}

#[get("/blades.bgs.services/api/game/v1/public/characters/{character_id}/dungeons")]
pub async fn get_dungeons() -> Json<DungeonResponse> {
    //TODO:
    Json(DungeonResponse {
        dungeons: Vec::new(),
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EnterDungeonRequest {
    #[allow(unused)]
    dungeon_instance: B64EncodedData,
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

            let quest = match quest_query.get(0) {
                Some(v) => v,
                None => todo!("Error on accept non-started quest"),
            };

            if quest.dungeon_state.is_some() {
                todo!("Error on enter already entered dungeon (or handle it properly?)")
            }

            let status = DungeonStatus {
                dungeon_settings_ids: vec![dungeon_info.dungeon_uuid],
                revive_count: 0,
                algorithm_version: 1,
                current_state: body.current_state.clone(),
                seed: 54321,
                level: 1,
                version: 1, //TODO: figure out where this version come from.
            };

            {
                use crate::schema::quests::dsl::*;

                diesel::update(quests)
                    .filter(id.eq(quest_id))
                    .set(dungeon_state.eq(Some(JsonDbWrapper(status.clone()))))
                    .execute(&mut conn)
                    .await
                    .unwrap();
            }

            Ok(Json(EnterDungeonResponse {
                dungeon_status: status,
            }))
        }
        .scope_boxed()
    })
    .await
}
