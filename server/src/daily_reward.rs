use std::sync::Arc;

use actix_web::{
    post,
    web::{self, Json},
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{BladeApiError, ServerGlobal, session::SessionLookedUpMaybe};

#[post(
    "/blades.bgs.services/api/game/v1/public/characters/{character_id}/towns/current/rewards/current"
)]
async fn get_daily_reward(
    _session: SessionLookedUpMaybe,
    _app_state: web::Data<Arc<ServerGlobal>>,
    _path: web::Path<Uuid>,
) -> Result<Json<Value>, BladeApiError> {
    //TODO: de-placeholder-ify this function
    Ok(Json(json!( {
        "dailyRewardStatus": {
            "rewardUid": "eefb9db4-0632-49b9-ae35-1da398ca0003",
            "until": 1774760493056_i64,
            "dailyReward": {
                "stackableItems": {
                    "42d91529-c88b-4c5b-815b-b55508b4e7ef": 2
                }
            },
            "collected": false
        }
    })))
}
