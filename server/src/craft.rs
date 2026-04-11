use std::collections::HashMap;

use actix_web::{get, web::Json};

#[get("blades.bgs.services/api/game/v1/public/characters/{character_id}/crafts")]
pub async fn get_crafts() -> Json<HashMap<u32, ()>> {
    Json(HashMap::default())
}
