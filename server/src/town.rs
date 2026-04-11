use std::sync::Arc;

use actix_web::{get, web::{self, Json}};
use tokio::{fs::File, io::AsyncReadExt};

use crate::ServerGlobal;

#[get("/blades.bgs.services/api/game/v1/public/characters/{character_id}/towns/current")]
pub async fn get_town(
    app_state: web::Data<Arc<ServerGlobal>>,
) -> Json<serde_json::Value> {
    let path = app_state.static_data_path.join("default_town.json");
    let mut file = File::open(&path).await.unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).await.unwrap();
    Json(serde_json::from_str(&content).unwrap())
}
