use actix_web::{get, web::Json};

#[get("/blades.bgs.services/api/analytics/v1/public/events")]
//TODO: is this path even used (with GET as opposed to POST)? Was it an error on my side
pub async fn list_events() -> Json<Option<()>> {
    Json(None)
}
