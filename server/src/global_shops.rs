use std::collections::HashMap;

use actix_web::{get, web::Json};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetGlobalShopOverrideResponse {
    global_shop_override: HashMap<Uuid, ()>,
}

#[get("/blades.bgs.services/api/game/v1/public/catalogoverrides/globalshop")]
async fn get_override() -> Json<GetGlobalShopOverrideResponse> {
    Json(GetGlobalShopOverrideResponse {
        global_shop_override: HashMap::new(),
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetGlobalShopForCharacterResponse {
    global_shop: HashMap<(), ()>,
}

#[get("/blades.bgs.services/api/game/v1/public/characters/{character_id}/globalshops/current")]
async fn get_global_shop_for_character() -> Json<GetGlobalShopForCharacterResponse> {
    Json(GetGlobalShopForCharacterResponse {
        global_shop: HashMap::new(),
    })
}
