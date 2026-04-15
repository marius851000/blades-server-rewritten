use std::collections::HashMap;

use actix_web::{get, post, web::Json};

#[post("/blades.bgs.services/api/analytics/v1/public/stats/client")]
pub async fn blades_bgs_stat_analytics() -> Json<Option<()>> {
    return Json(None);
}

#[post("/blades.bgs.services/api/analytics/v1/public/events")]
pub async fn blades_bgs_event_analytics() -> Json<Option<()>> {
    return Json(None);
}

#[post("/{server_id}.api.swrve.com/1/batch")]
pub async fn swrve_batch_submit() -> &'static str {
    ""
}

#[get("/{server_id}.content.swrve.com/api/1/user_resources_and_campaigns")]
pub async fn swrve_submit_device_info() -> Json<HashMap<(), ()>> {
    Json(HashMap::new())
}
