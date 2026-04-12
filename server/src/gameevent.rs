use actix_web::{post, web::Json};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetGameEventsResponse {
    game_events: Vec<()>,
}

#[post("/blades.bgs.services/api/game/v1/public/characters/{character_id}/gameevents")]
pub async fn get_game_events() -> Json<GetGameEventsResponse> {
    //TODO: placeholder
    Json(GetGameEventsResponse {
        game_events: Vec::new(),
    })
}
