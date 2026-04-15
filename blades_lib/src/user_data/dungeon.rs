use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::user_data::Items;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LootTableResult {
    pub stackable_items: HashMap<Uuid, u64>,
    pub currencies: HashMap<Uuid, u64>,
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
    pub enemy_generated_data: HashMap<Uuid, Vec<DungeonEnemyResult>>,
    pub item_generated_data: HashMap<Uuid, Vec<DungeonItemResult>>,
    pub chest_generated_data: HashMap<Uuid, Vec<ChestGeneratedData>>,
    pub algorithm_version: u64,
    pub version: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DungeonGeneratedDataWithId {
    pub quest_id: Uuid,
    #[serde(flatten)]
    pub inner: DungeonGeneratedData,
}
