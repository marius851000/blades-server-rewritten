use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum QuestType {
    Normal,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum QuestStatus {
    Active,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ObjectiveStatus {
    pub status: QuestStatus,
    pub progress: f64,
    pub completed: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Quest {
    pub version: u64,
    pub r#type: QuestType,
    pub objectives_statuses: HashMap<Uuid, ObjectiveStatus>,
    pub difficulty_level: i64,
    pub seed: u64,
    pub gld_quest_id: Uuid,
    pub completed: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QuestWithId {
    pub quest_id: Uuid,
    #[serde(flatten)]
    pub quest: Quest,
}
