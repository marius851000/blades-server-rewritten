use actix_web::{get, web::Json};
use serde_json::{Value, json};

#[get("/blades.bgs.services/api/game/v1/public/characters/{character_id}/announcements")]
pub async fn get_announcements() -> Json<Value> {
    Json(json!({
        "announcements": []
    }))
}
