use std::sync::Arc;

use actix_web::{get, web};
use serde::Serialize;

use crate::{BladeApiError, ServerGlobal, session::SessionLookedUpMaybe};

#[derive(Serialize)]
struct CharacterListResponse {
    characters: Vec<()>, // placeholder, cause apparently infallible is not serializable
}

#[get("/blades.bgs.services/api/game/v1/public/characters")]
async fn list_characters(
    session: SessionLookedUpMaybe,
    app_state: web::Data<Arc<ServerGlobal>>,
) -> Result<web::Json<CharacterListResponse>, BladeApiError> {
    let session = session.get_session_or_error()?;
    let characters_result = app_state
        .db_pool
        .get()
        .await
        .unwrap()
        .query(
            "SELECT id FROM characters WHERE data->>'user_id' = $1",
            &[&session.session.user_id.to_string()],
        )
        .await?;
    let result = Vec::with_capacity(characters_result.len());
    for _character_entry in characters_result.iter() {
        todo!("load back existing characters");
    }
    Ok(web::Json(CharacterListResponse { characters: result }))
}
