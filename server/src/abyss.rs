use actix_web::{
    post,
    web::{self, Json},
};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
struct GetAbyssResponse {
    abyss: Option<()>,
}

#[post("/blades.bgs.services/api/game/v1/public/characters/{character_id}/abysses/current")]
pub async fn get_abyss(_path: web::Path<Uuid>) -> Json<GetAbyssResponse> {
    //TODO: implement
    Json(GetAbyssResponse { abyss: None })
}
