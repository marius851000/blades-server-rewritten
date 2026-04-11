use actix_web::{get, web::Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct DungeonResponse {
    dungeons: Vec<()>,
}

#[get("/blades.bgs.services/api/game/v1/public/characters/{character_id}/dungeons")]
pub async fn get_dungeons() -> Json<DungeonResponse> {
    Json(DungeonResponse {
        dungeons: Vec::new(),
    })
}
