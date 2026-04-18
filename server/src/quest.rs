use std::sync::Arc;

use actix_web::{
    post,
    web::{self, Json},
};
use blades_lib::{
    user_data::{CompleteCharacterWithIdWithoutData, DungeonGeneratedDataWithId, QuestWithId},
    util::quest::generate_quest_data,
};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper, associations::HasTable, insert_into};
use diesel_async::{AsyncConnection, RunQueryDsl, scoped_futures::ScopedFutureExt};
use serde::Serialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    BladeApiError, ServerGlobal,
    json_db::JsonDbWrapper,
    models::{CharacterDbEntryCharacterAlone, QuestDbEntry},
    session::SessionLookedUpMaybe,
    util::{self, check_permission_for_character_and_get_it},
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetQuestsResponse {
    quests: Vec<QuestWithId>,
    dungeon_generated_data_list: Vec<DungeonGeneratedDataWithId>,
    jobs: Vec<()>, //TODO:
    character: CompleteCharacterWithIdWithoutData,
    job_pools: Value,                      //TODO: this one is critical
    game_event_quests: Vec<()>,            //TODO:
    game_event_quests_in_warning: Vec<()>, //TODO,
    game_event_quests_finished: Vec<()>,   //TODO
}

#[post("/blades.bgs.services/api/game/v1/public/characters/{character_id}/quests")]
pub async fn get_quests(
    session: SessionLookedUpMaybe,
    request: Json<Option<()>>,
    app_state: web::Data<Arc<ServerGlobal>>,
    path: web::Path<Uuid>,
) -> Result<Json<GetQuestsResponse>, BladeApiError> {
    assert!(request.is_none());
    let session = session.get_session_or_error()?;

    let character_id_var = path.into_inner();
    let mut conn = app_state.db_pool.get().await.unwrap();
    conn.transaction(|mut conn| {
        async move {
            let character = {
                use crate::schema::characters::dsl::*;

                characters::table()
                    .filter(id.eq(&character_id_var))
                    .select(CharacterDbEntryCharacterAlone::as_select())
                    .load(&mut conn)
                    .await?
            };
            let character =
                util::get_only_single_character_and_check_permission(character, &session.session)?;

            // we could have done an inner join to check the get the user id, but the user has already been checked previously.
            let quests = {
                use crate::schema::quests::dsl::*;
                // take care! that line above import a character_id thing
                quests::table()
                    .filter(character_id.eq(&character_id_var))
                    .select(QuestDbEntry::as_select())
                    .load(&mut conn)
                    .await?
            };

            let mut result_quests = Vec::new();
            let mut result_generated_data = Vec::new();

            for quest in quests {
                result_quests.push(QuestWithId {
                    quest_id: quest.id,
                    quest: quest.info.0,
                });
                if let Some(generated_data) = quest.generated_data.0 {
                    result_generated_data.push(DungeonGeneratedDataWithId {
                        quest_id: quest.id,
                        inner: generated_data,
                    });
                };
            }

            Ok(Json(GetQuestsResponse {
                quests: result_quests,
                dungeon_generated_data_list: result_generated_data,
                character: CompleteCharacterWithIdWithoutData {
                    id: character_id_var,
                    character: character.character.0,
                },
                jobs: Vec::new(),
                game_event_quests: Vec::new(),
                game_event_quests_finished: Vec::new(),
                game_event_quests_in_warning: Vec::new(),
                job_pools: json! {
                    [
                        {
                            "id": "4956c6ab-1832-4edd-8bee-561b79f83ee2",
                            "endTime": 1774760400,
                            "nextStartTime": 1774760400
                        },
                        {
                            "id": "717b3cf5-21d8-4f0c-a7a9-603fe37b8766",
                            "endTime": 1774760400,
                            "nextStartTime": 1774760400
                        },
                        {
                            "id": "361da91e-6860-4c31-a447-4010cbaad1dd",
                            "endTime": 1774846800,
                            "nextStartTime": 1774846800
                        },
                        {
                            "id": "9d94baeb-96d4-49e9-bdf6-9f939be836d3",
                            "endTime": 0,
                            "nextStartTime": 1774760400
                        },
                        {
                            "id": "c5efa81d-18d9-47f3-a0ac-e108c0a50605",
                            "endTime": 0,
                            "nextStartTime": 1774846800
                        },
                        {
                            "id": "6b2a5baa-f64f-4cfe-8b03-fe7d632ea2f1",
                            "endTime": 0,
                            "nextStartTime": 1774933200
                        },
                        {
                            "id": "9fcbb01c-13bf-4cd9-916f-25d5faf5314e",
                            "endTime": 0,
                            "nextStartTime": 1775019600
                        },
                        {
                            "id": "df666a07-3539-426a-916e-ccdba580cb1d",
                            "endTime": 0,
                            "nextStartTime": 1775106000
                        },
                        {
                            "id": "a4e76931-02bf-4bfb-a472-286e968a03e1",
                            "endTime": 0,
                            "nextStartTime": 1775192400
                        },
                        {
                            "id": "8501a030-5009-4c73-a864-69c3d7fe6ae5",
                            "endTime": 1774760400,
                            "nextStartTime": 1775278800
                        }
                    ]
                },
            }))
        }
        .scope_boxed()
    })
    .await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AcceptQuestResponse {
    quest: QuestWithId,
    dungeon_generated_data: Option<DungeonGeneratedDataWithId>,
}

#[post(
    "/blades.bgs.services/api/game/v1/public/characters/{character_id}/quests/{quest_id}/accept"
)]
async fn accept_quest(
    session: SessionLookedUpMaybe,
    request: Json<Option<()>>,
    app_state: web::Data<Arc<ServerGlobal>>,
    path: web::Path<(Uuid, Uuid)>,
) -> Result<Json<AcceptQuestResponse>, BladeApiError> {
    assert!(request.is_none());
    let session = session.get_session_or_error()?;
    let (character_id, quest_id) = path.into_inner();
    let mut conn = app_state.db_pool.get().await.unwrap();

    // check permission
    let _ = check_permission_for_character_and_get_it(&mut conn, &session.session, character_id)
        .await?;

    // actually add quest

    let (quest, dungeon_generated_data) = generate_quest_data(&app_state.game_data, quest_id)?;
    //TODO: specifically handle the case the quest already exist (primary key is character id + quest id)

    let to_insert = QuestDbEntry {
        id: quest_id,
        character_id,
        info: JsonDbWrapper(quest.clone()),
        generated_data: JsonDbWrapper(dungeon_generated_data.clone()),
        dungeon_state: None,
    };

    {
        use crate::schema::quests::dsl::*;

        insert_into(quests::table())
            .values(&to_insert)
            .execute(&mut conn)
            .await?;
    }

    Ok(Json(AcceptQuestResponse {
        quest: QuestWithId {
            quest_id: quest_id,
            quest,
        },
        dungeon_generated_data: dungeon_generated_data.map(|v| DungeonGeneratedDataWithId {
            quest_id: quest_id,
            inner: v,
        }),
    }))
}
