use std::collections::HashMap;

use actix_web::{
    get,
    web::{self, Json},
};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct LeaderboardResult {
    leaderboard: InnerLeaderboardResult,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct InnerLeaderboardResult {
    total_entries: u64,
    current_page: u64,
    total_pages: u64,
    entries: Vec<()>,
}

#[get("/blades.bgs.services/api/game/v1/public/characters/{character_id}/leaderboards/{unk}")]
pub async fn get_leaderboard(
    _path: web::Path<(Uuid, Uuid)>,
    _query: web::Query<HashMap<String, String>>,
) -> Json<LeaderboardResult> {
    Json(LeaderboardResult {
        leaderboard: InnerLeaderboardResult {
            total_entries: 0,
            current_page: 0,
            total_pages: 0,
            entries: Vec::new(),
        },
    })
}
