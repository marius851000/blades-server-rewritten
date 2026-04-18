use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::user_data::{B64EncodedData, Items};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LootTableResult {
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    pub stackable_items: HashMap<Uuid, u64>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    pub currencies: HashMap<Uuid, u64>,
    #[serde(skip_serializing_if = "Items::is_empty")]
    #[serde(default)]
    pub item: Items,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DungeonEnemyResult {
    pub enemy_level: i64,
    #[serde(rename = "givenXP")]
    pub given_xp: u64,
    //TODO: need to find a filled spawn_group_loot to verify it really is that.
    pub spawn_group_loot: HashMap<Uuid, LootTableResult>,
    pub loot_table_loot: HashMap<Uuid, LootTableResult>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DungeonItemResult {
    pub loot_table_loot: HashMap<Uuid, LootTableResult>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChestGeneratedData {
    pub tier: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DungeonGeneratedData {
    //TODO: figure what the two level of depth are used for
    pub enemy_generated_data: HashMap<Uuid, Vec<Vec<DungeonEnemyResult>>>,
    pub item_generated_data: HashMap<Uuid, Vec<DungeonItemResult>>,
    pub chest_generated_data: HashMap<Uuid, Vec<ChestGeneratedData>>,
    pub algorithm_version: u64,
    pub version: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DungeonGeneratedDataWithId {
    pub quest_id: Uuid,
    #[serde(flatten)]
    pub inner: DungeonGeneratedData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DungeonStatus {
    pub dungeon_settings_ids: Vec<Uuid>,
    pub revive_count: u64,
    pub level: u64,
    pub seed: i64,
    pub current_state: B64EncodedData,
    pub algorithm_version: i64,
    pub version: i64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DungeonState {
    pub dungeon_status: DungeonStatus,
}
