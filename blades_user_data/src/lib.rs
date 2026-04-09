use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;
mod wallet;
pub use wallet::Wallet;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CompleteData {
    pub customization: Value,
}

impl Default for CompleteData {
    fn default() -> Self {
        CompleteData {
            customization: json!({}),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
// May also be sent to the user on initial sync (does not have the id field, see #[serde(flatten)])
pub struct CompleteCharacter {
    pub name: String,
    pub tag_id: String,
    // equippedAbilities
    // abilities
    pub version: u64,
    pub level: u16,
    pub experience: u64,
    // completedQuests
    pub maximum_abyss_level_reached: u16,

    pub current_quest_dungeon: Option<()>, // TODO: figure what is actually stored here
    pub last_jobs_reset_time: u64,
    pub inventory_level: u16,
    pub stamina_attribute_points: u32,
    pub magicka_attribute_points: u32,
    // globalShopOffers
    // challengeSeason
    // loadoutProfiles
    pub last_guild_exchange_request_time: u64,
    pub last_guild_exchange_donation_time: u64,
    pub guild_exchange_donation_count: i64,
    pub pvp_chest_meter: i64,
    pub pvp_winning_streak: i64,
    pub pvp_exception_easier_match_remaining: i64,
    pub pvp_exception_harder_match_remaining: i64,
    pub matchmaking_pvp_trophies: i64,
    pub pvp_trophies: i64,
    pub highest_arena_reached: u64,
    pub highest_level_arena_reached: u64,
    pub number_pvp_match_played: i64,
    pub trophy_count_modified: i64,
    pub pvp_season_id: Uuid,
    pub job_difficulty_cycle_index: i64,
    pub data: CompleteData,
    pub validation_flags: u32,
    pub trasury_level: u32,
    //avatar_icon_id
    pub name_validated: bool,
}

impl Default for CompleteCharacter {
    fn default() -> Self {
        CompleteCharacter {
            name: String::default(),
            tag_id: String::default(),
            version: 0,
            level: 1,
            experience: 1,
            maximum_abyss_level_reached: 0,
            current_quest_dungeon: None,
            last_jobs_reset_time: 0,
            inventory_level: 0,
            stamina_attribute_points: 0,
            magicka_attribute_points: 0,
            last_guild_exchange_request_time: 0,
            last_guild_exchange_donation_time: 0,
            guild_exchange_donation_count: 0,
            pvp_chest_meter: 0,
            pvp_winning_streak: 0,
            pvp_exception_easier_match_remaining: 0,
            pvp_exception_harder_match_remaining: 0,
            matchmaking_pvp_trophies: 0,
            pvp_trophies: 0,
            highest_arena_reached: 1,
            highest_level_arena_reached: 1,
            number_pvp_match_played: 0,
            trophy_count_modified: 0,
            pvp_season_id: Uuid::default(),
            job_difficulty_cycle_index: 0,
            validation_flags: 1,
            trasury_level: 0,
            name_validated: true,
            data: CompleteData::default(),
        }
    }
}
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct CompleteInventory {}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PersistedCharacterData {
    //This is used in query, and should stay constant across the server lifetime
    pub user_id: Uuid,
    pub character: CompleteCharacter,
    pub inventory: CompleteInventory,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UserAccount {
    pub gp_deviceids: HashSet<String>,
    /// The user id that is actually communicated with the client, and should be kept secret
    pub secret_id: Uuid,
}

impl UserAccount {
    pub fn create_new_user() -> Self {
        UserAccount {
            gp_deviceids: HashSet::default(),
            secret_id: Uuid::new_v4(),
        }
    }
}
