use std::collections::HashMap;

use thiserror::Error;
use uuid::Uuid;

use crate::{
    game_data::GameData,
    user_data::{
        ChestGeneratedData, DungeonEnemyResult, DungeonGeneratedData, DungeonItemResult,
        ObjectiveStatus, Quest, QuestStatus, QuestType,
    },
};

#[derive(Error, Debug, Clone)]
pub enum GenerateQuestDataError {
    #[error("quest {0} does not exist")]
    QuestNotFound(Uuid),
    #[error("dungeon {0} does not exist")]
    DungeonNotFound(Uuid),
}

pub fn generate_quest_data(
    game_data: &GameData,
    quest_id: Uuid,
) -> Result<(Quest, Option<DungeonGeneratedData>), GenerateQuestDataError> {
    let quest_data = game_data
        .quests
        .get(&quest_id)
        .ok_or(GenerateQuestDataError::QuestNotFound(quest_id))?;

    // Non-dungeon quest need more parsing: they DO have objective, and version is also returned (but is at least the version default of 0?)
    let dungeon_info = quest_data.dungeon_info.as_ref().unwrap();
    let quest = Quest {
        completed: false,
        difficulty_level: -1,
        gld_quest_id: quest_id,
        seed: 1234,
        r#type: QuestType::Normal,
        version: dungeon_info.version,
        objective_statuses: dungeon_info
            .objectives
            .iter()
            .map(|(id, _o)| {
                (
                    *id,
                    ObjectiveStatus {
                        completed: false,
                        progress: 0.0,
                        status: QuestStatus::Active,
                    },
                )
            })
            .collect(),
    };

    let dungeon = game_data.dungeons.get(&dungeon_info.dungeon_uuid).ok_or(
        GenerateQuestDataError::DungeonNotFound(dungeon_info.dungeon_uuid),
    )?;

    let generated_dungeon_data = DungeonGeneratedData {
        enemy_generated_data: dungeon
            .spawn_info
            .enemy_spawn_groups
            .iter()
            .map(|(spawn_group_id, spawn_group)| {
                let mut enemies_info = Vec::new();
                for _ in 0..spawn_group.quantity {
                    enemies_info.push(DungeonEnemyResult {
                        enemy_level: 1,
                        given_xp: 1000,
                        spawn_group_loot: HashMap::default(),
                        loot_table_loot: HashMap::default(),
                    })
                }
                (*spawn_group_id, enemies_info)
            })
            .collect(),
        chest_generated_data: dungeon
            .spawn_info
            .chest
            .iter()
            .map(|(chest_spawn_id, _spawn_info)| {
                (*chest_spawn_id, vec![ChestGeneratedData { tier: 1 }])
            })
            .collect(),
        item_generated_data: dungeon
            .spawn_info
            .item
            .iter()
            .map(|(item_spawn_id, _spawn_info)| {
                (
                    *item_spawn_id,
                    vec![DungeonItemResult {
                        loot_table_loot: HashMap::default(),
                    }],
                )
            })
            .collect(),
        algorithm_version: 1,
        version: dungeon_info.version,
    };

    Ok((quest, Some(generated_dungeon_data)))
}
