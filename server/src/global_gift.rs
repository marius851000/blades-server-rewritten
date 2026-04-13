use std::collections::HashMap;

use actix_web::{get, web::Json};

#[get("/blades.bgs.services/api/game/v1/public/characters/{character_id}/globalgifts")]
async fn get_global_gifts() -> Json<HashMap<(), ()>> {
    Json(HashMap::new())
}
