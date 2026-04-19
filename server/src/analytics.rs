use std::collections::HashMap;

use actix_web::{
    get, post,
    web::{self, Json},
};
use serde::Serialize;
use uuid::Uuid;

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
pub async fn swrve_submit_device_info(
    _query: web::Query<HashMap<String, String>>,
) -> Json<HashMap<(), ()>> {
    Json(HashMap::new())
}

#[derive(Serializls e, Debug)]
struct SwrveIdentifyResponse {
    status: &'static str,
    swrve_id: Uuid,
}

#[post("/{server_id}.identity.swrve.com/identify")]
pub async fn swrve_identity_identify() -> Json<SwrveIdentifyResponse> {
    Json(SwrveIdentifyResponse {
        status: "new_external_id",
        swrve_id: Uuid::new_v4(),
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppcenterLogResponse {
    status: &'static str,
    valid_diagnostics_ids: Vec<Uuid>,
    throttled_diagnostics_ids: Vec<Uuid>,
    correlation_id: Uuid,
}

#[post("/in.appcenter.ms/logs")]
pub async fn appcenter_log(
    _query: web::Query<HashMap<String, String>>,
) -> Json<AppcenterLogResponse> {
    Json(AppcenterLogResponse {
        status: "Success",
        valid_diagnostics_ids: Vec::new(),
        throttled_diagnostics_ids: Vec::new(),
        correlation_id: Uuid::new_v4(),
    })
}
