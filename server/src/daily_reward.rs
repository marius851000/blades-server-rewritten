use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

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
    // providing an "until" from the past cause the client to try getting an updated value an not process any other request until a new up to date one is fetched

    let expire_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|x| x.as_millis())
        .unwrap_or(0)
        + 3600 * 1000;

    Ok(Json(json!( {
        "dailyRewardStatus": {
            "rewardUid": "eefb9db4-0632-49b9-ae35-1da398ca0003",
            "until": expire_time,
            "dailyReward": {
                "stackableItems": {
                    "42d91529-c88b-4c5b-815b-b55508b4e7ef": 2
                }
            },
            "collected": false
        }
    })))
}
