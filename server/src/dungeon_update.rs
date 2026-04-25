use std::sync::Arc;

use crate::{
    json_db::JsonDbWrapper,
    models::{CharacterDbEntryCharacterWalletInventory, QuestDbEntryDungeonStateAndGeneratedData},
};
use actix_web::{
    post,
    web::{self, Json},
};
use blades_lib::user_data::{
    B64EncodedData, CompleteCharacterWithIdWithoutData, CompleteInventoryUpdate, DungeonStatus,
    EnemyIndex, EnemyStatus, InventoryChangeTracker,
};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{AsyncConnection, RunQueryDsl, scoped_futures::ScopedFutureExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{BladeApiError, ServerGlobal, session::SessionLookedUpMaybe};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct EnemyKilledUpdate {
    pub spawn_group_id: Uuid,
    pub spawner_index: usize,
    pub enemy_index: usize,
    #[allow(unused)]
    // We use the data stored in the generated data instead of trusting the client
    pub xp_reward: f64,
    pub time: u64,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
enum DungeonUpdateAction {
    EnemyKilled(EnemyKilledUpdate),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DungeonUpdateRequest {
    current_state: B64EncodedData,
    actions: Vec<DungeonUpdateAction>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DungeonUpdateResponse {
    inventory: CompleteInventoryUpdate,
    character: CompleteCharacterWithIdWithoutData,
    dungeon_status: DungeonStatus,
}

#[post(
    "blades.bgs.services/api/game/v1/public/characters/{character_id}/quests/{quest_id}/dungeons/current/update"
)]
pub async fn dungeon_update(
    path: web::Path<(Uuid, Uuid)>,
    session: SessionLookedUpMaybe,
    app_state: web::Data<Arc<ServerGlobal>>,
    body: Json<DungeonUpdateRequest>,
) -> Result<Json<DungeonUpdateResponse>, BladeApiError> {
    let session = session.get_session_or_error()?;
    let (character_id, quest_id) = path.into_inner();
    let mut conn = app_state.db_pool.get().await.unwrap();

    conn.transaction(|mut conn| {
        async move {
            let (quest_data, mut character_data) = {
                use crate::schema::characters;
                use crate::schema::quests;

                quests::table
                    .filter(quests::id.eq(quest_id))
                    .filter(characters::id.eq(character_id))
                    .inner_join(characters::table)
                    .filter(characters::user_id.eq(session.session.user_id))
                    .select((
                        QuestDbEntryDungeonStateAndGeneratedData::as_select(),
                        CharacterDbEntryCharacterWalletInventory::as_select(),
                    ))
                    .for_no_key_update()
                    .load(&mut conn)
                    .await?
                    .into_iter()
                    .next()
                    .expect("TODO: proper error handling")
            };

            let generated_data = quest_data
                .generated_data
                .0
                .expect("TODO: proper error handling");
            let mut dungeon_state = quest_data
                .dungeon_state
                .expect("TODO: proper error handling")
                .0;

            let inventory_modification_tracker = InventoryChangeTracker::default();

            dungeon_state.dungeon_status.current_state = body.current_state.clone();

            for action in &body.actions {
                match action {
                    DungeonUpdateAction::EnemyKilled(enemy_killed) => {
                        let enemy_index = EnemyIndex::new(
                            enemy_killed.spawn_group_id,
                            enemy_killed.spawner_index,
                            enemy_killed.enemy_index,
                        );
                        let enemy_generated_data = generated_data
                            .get_enemy(&enemy_index)
                            .expect("TODO: error handling");
                        if let Some(current_enemy_data) = dungeon_state
                            .dungeon_status
                            .enemy_status
                            .get_mut(&enemy_index)
                        {
                            if current_enemy_data.killed {
                                panic!(
                                    "TODO: properly handle killing already killed enemy (or just early continue?)"
                                );
                            }
                            current_enemy_data.killed = true;
                        } else {
                            dungeon_state.dungeon_status.enemy_status.insert(
                                enemy_index,
                                EnemyStatus {
                                    spawn_group_id: enemy_killed.spawn_group_id,
                                    xp_reward: enemy_generated_data.given_xp,
                                    killed: true,
                                    time: enemy_killed.time,
                                    loot: enemy_generated_data.merged_loot_table(),
                                },
                            );
                        }

                        character_data.character.0.experience += enemy_generated_data.given_xp;
                    }
                }
            }

            // generate the response before we submit data to minimize the amount of cloning needed

            let result = DungeonUpdateResponse {
                dungeon_status: dungeon_state.dungeon_status.clone(),
                character: CompleteCharacterWithIdWithoutData {
                    id: character_id,
                    character: character_data.character.0.clone(),
                },
                inventory: character_data.inventory.0.generate_client_update(&inventory_modification_tracker)
            };

            let quest_data_rebuilt = QuestDbEntryDungeonStateAndGeneratedData {
                id: quest_id,
                dungeon_state: Some(JsonDbWrapper(dungeon_state)),
                generated_data: JsonDbWrapper(Some(generated_data)),
            };

            {
                use crate::schema::quests;
                diesel::update(quests::table)
                    .filter(quests::id.eq(quest_id))
                    .set(quest_data_rebuilt)
                    .execute(&mut conn)
                    .await?;
            }

            {
                use crate::schema::characters;

                diesel::update(characters::table)
                    .filter(characters::id.eq(character_data.id))
                    .set(character_data)
                    .execute(&mut conn)
                    .await?;
            }

            Ok::<_, BladeApiError>(Json(result))
        }
    }.scope_boxed()).await
}
