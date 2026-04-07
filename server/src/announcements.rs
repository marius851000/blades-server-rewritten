use actix_web::{get, web};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StatusResponse {
    ttl: u64,
    systems: Vec<StatusEntryResponse>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StatusEntryResponse {
    name: &'static str,
    status: &'static str,
}

#[get("announcements.blades.bgs.services/status/status.json")]
async fn check_status() -> web::Json<StatusResponse> {
    web::Json(StatusResponse {
        ttl: 300,
        systems: vec![
            StatusEntryResponse {
                name: "authentication",
                status: "online",
            },
            StatusEntryResponse {
                name: "game",
                status: "online",
            },
            StatusEntryResponse {
                name: "pvp",
                status: "online",
            },
            StatusEntryResponse {
                name: "guilds",
                status: "online",
            },
            StatusEntryResponse {
                name: "events",
                status: "online",
            },
            StatusEntryResponse {
                name: "social",
                status: "online",
            },
            StatusEntryResponse {
                name: "quests",
                status: "online",
            },
            StatusEntryResponse {
                name: "challenges",
                status: "online",
            },
            StatusEntryResponse {
                name: "shops",
                status: "online",
            },
        ],
    })
}
