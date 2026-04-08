use actix_web::{get, web::Json};

#[get("/blades.bgs.services/api/analytics/v1/public/events")]
pub async fn list_events() -> Json<Option<()>> {
    Json(None)
}
