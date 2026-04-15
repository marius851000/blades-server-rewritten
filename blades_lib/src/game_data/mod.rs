use std::collections::HashMap;

use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct EmptyStruct {}

#[derive(Deserialize)]
pub struct GameDataItem {
    pub name: String,
    pub r#type: u64,
}

#[derive(Deserialize)]
pub struct GameDataLootTable(pub HashMap<Uuid, EmptyStruct>);

#[derive(Deserialize)]
pub struct GameDataInteractable {
    pub loot_table: HashMap<Uuid, GameDataLootTable>,
}

#[derive(Deserialize, Clone)]
pub struct GameDataItemReward {
    pub count: u64,
    pub template_uuid: Uuid,
}

#[derive(Deserialize, Clone)]
pub struct GameDataReward {
    pub experience: f64,
    pub town_points: u64,
    pub chest_is_none: bool,
    pub items_to_reward: Vec<GameDataItemReward>,
}

#[derive(Deserialize, Clone)]
pub struct GameDataObjective {
    pub description: String,
    pub quota: f64,
    pub rewards: Vec<GameDataReward>,
}

#[derive(Deserialize, Clone)]
pub struct GameDataQuestDungeonInfo {
    pub objectives: HashMap<Uuid, GameDataObjective>,
    pub version: u64,
    pub dungeon_uuid: Uuid,
}

#[derive(Deserialize)]
pub struct GameDataQuest {
    pub dungeon_info: Option<GameDataQuestDungeonInfo>,
}

#[derive(Deserialize)]
pub struct GameDataItemApparitionSettings {
    pub interactable_uuid: Uuid,
    pub weight: u64,
    pub mandatory: u64,
}

#[derive(Deserialize)]
pub struct GameDataItemSpawnInfo {
    pub name: Option<String>,
    pub apparition_settings: Vec<GameDataItemApparitionSettings>,
}

#[derive(Deserialize)]
pub struct GameDataEnemySpawnGroup {
    pub quantity: u64,
}

#[derive(Deserialize)]
pub struct GameDataDungeonSpawnInfo {
    pub chest: HashMap<Uuid, EmptyStruct>,
    pub item: HashMap<Uuid, GameDataItemSpawnInfo>,
    pub enemy_spawn_groups: HashMap<Uuid, GameDataEnemySpawnGroup>,
}

#[derive(Deserialize)]
pub struct GameDataDungeon {
    pub handle: String,
    pub spawn_info: GameDataDungeonSpawnInfo,
}

#[derive(Deserialize)]
pub struct GameData {
    pub items_template: HashMap<Uuid, GameDataItem>,
    pub interactables: HashMap<Uuid, GameDataInteractable>,
    pub quests: HashMap<Uuid, GameDataQuest>,
    pub dungeons: HashMap<Uuid, GameDataDungeon>,
}
