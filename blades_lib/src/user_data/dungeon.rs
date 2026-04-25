use std::{collections::HashMap, fmt};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
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

impl LootTableResult {
    pub fn merge(&mut self, other: LootTableResult) {
        for (uuid, amount) in other.stackable_items {
            self.stackable_items.insert(
                uuid,
                self.stackable_items.get(&uuid).map(|x| *x).unwrap_or(0) + amount,
            );
        }
        for (uuid, amount) in other.currencies {
            self.currencies.insert(
                uuid,
                self.currencies.get(&uuid).map(|x| *x).unwrap_or(0) + amount,
            );
        }
        self.item.0.extend(other.item.0);
    }
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

impl DungeonEnemyResult {
    pub fn merged_loot_table(&self) -> LootTableResult {
        let mut result = LootTableResult::default();
        for loot_table in self
            .spawn_group_loot
            .values()
            .chain(self.loot_table_loot.values())
        {
            result.merge(loot_table.clone());
        }
        result
    }
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
    //TODO: figure what the two level of depth are used for (one is named "spawner"(id) and the second "enemy"(id))
    pub enemy_generated_data: HashMap<Uuid, Vec<Vec<DungeonEnemyResult>>>,
    pub item_generated_data: HashMap<Uuid, Vec<DungeonItemResult>>,
    pub chest_generated_data: HashMap<Uuid, Vec<ChestGeneratedData>>,
    pub algorithm_version: u64,
    pub version: u64,
}

impl DungeonGeneratedData {
    pub fn get_enemy(&self, index: &EnemyIndex) -> Option<&DungeonEnemyResult> {
        self.enemy_generated_data
            .get(&index.spawner_uuid)
            .and_then(|spawner_data| spawner_data.get(index.spawner_index))
            .and_then(|enemy_data| enemy_data.get(index.enemy_index))
    }
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
pub struct EnemyStatus {
    pub spawn_group_id: Uuid,
    pub xp_reward: u64,
    pub killed: bool,
    pub time: u64,
    pub loot: LootTableResult,
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
    #[serde(default)]
    pub enemy_status: HashMap<EnemyIndex, EnemyStatus>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DungeonState {
    pub dungeon_status: DungeonStatus,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EnemyIndex {
    pub spawner_uuid: Uuid,
    pub spawner_index: usize,
    pub enemy_index: usize,
}

impl EnemyIndex {
    pub fn new(spawner_uuid: Uuid, spawner_index: usize, enemy_index: usize) -> Self {
        Self {
            spawner_uuid,
            spawner_index,
            enemy_index,
        }
    }
}

impl fmt::Display for EnemyIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}-{}-{}",
            self.spawner_uuid, self.spawner_index, self.enemy_index
        )
    }
}

// Serialize as a single string “uuid-index-index”
impl Serialize for EnemyIndex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

// Deserialize from that string format
impl<'de> Deserialize<'de> for EnemyIndex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        // Split from the right so that UUID (which may contain dashes) stays intact.
        let parts: Vec<&str> = s.rsplitn(3, '-').collect();
        if parts.len() != 3 {
            return Err(serde::de::Error::custom("Invalid EnemyIndex format"));
        }
        let enemy_index = parts[0]
            .parse::<usize>()
            .map_err(serde::de::Error::custom)?;
        let spawner_index = parts[1]
            .parse::<usize>()
            .map_err(serde::de::Error::custom)?;
        let spawner_uuid = Uuid::parse_str(parts[2])
            .map_err(serde::de::Error::custom)?;
        Ok(EnemyIndex {
            spawner_uuid,
            spawner_index,
            enemy_index,
        })
    }
}
