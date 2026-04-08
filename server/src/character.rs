use std::sync::Arc;

use actix_web::{
    get,
    http::StatusCode,
    post,
    web::{self, Json},
};
use blades_user_data::{CompleteCharacter, CompleteInventory, PersistedCharacterData};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{BladeApiError, ServerGlobal, session::SessionLookedUpMaybe};

#[derive(Serialize)]
struct CompleteCharacterWithId {
    id: Uuid,
    #[serde(flatten)]
    character: CompleteCharacter,
}

#[derive(Serialize)]
struct CharacterListResponse {
    characters: Vec<CompleteCharacterWithId>,
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
            "SELECT id FROM characters WHERE data->>'userId' = $1",
            &[&session.session.user_id.to_string()],
        )
        .await?;
    let mut result = Vec::with_capacity(characters_result.len());
    let mut client = app_state.db_pool.get().await.unwrap();
    for character_entry in characters_result.iter() {
        //TODO: convert error
        let character_id = character_entry.get(0);
        let character_guard = app_state
            .character_storage
            .get(character_id, &mut client)
            .await
            .unwrap();
        result.push(CompleteCharacterWithId {
            id: character_id,
            character: character_guard.character.clone(),
        });
    }
    Ok(web::Json(CharacterListResponse { characters: result }))
}

#[derive(Serialize)]
struct CompleteCharacterWithIdOnly {
    character: CompleteCharacterWithId,
}

#[get("/blades.bgs.services/api/game/v1/public/characters/{character_id}")]
async fn get_character(
    session: SessionLookedUpMaybe,
    app_state: web::Data<Arc<ServerGlobal>>,
    path: web::Path<Uuid>,
) -> Result<Json<CompleteCharacterWithIdOnly>, BladeApiError> {
    let session = session.get_session_or_error()?;
    let character_id = path.into_inner();
    let mut client = app_state.db_pool.get().await.unwrap();
    //TODO: do not unwrap if character does not exist
    let character = app_state
        .character_storage
        .get(character_id, &mut client)
        .await
        .unwrap();
    if character.user_id != session.session.user_id {
        Err(BladeApiError::unauthorized())
    } else {
        Ok(Json(CompleteCharacterWithIdOnly {
            character: CompleteCharacterWithId {
                id: character_id,
                character: character.character.clone(),
            },
        }))
    }
}

#[derive(Deserialize)]
struct DataOnlyCustomization {
    customization: serde_json::Value,
}
#[derive(Deserialize)]
struct CharacterCreationRequest {
    name: String,
    data: DataOnlyCustomization,
}

#[derive(Serialize)]
struct CharacterCreationResponse {
    character: CompleteCharacterWithId,
    inventory: CompleteInventory,
}

#[post("/blades.bgs.services/api/game/v1/public/characters")]
async fn create_characters(
    session: SessionLookedUpMaybe,
    app_state: web::Data<Arc<ServerGlobal>>,
    body: web::Json<CharacterCreationRequest>,
) -> Result<web::Json<CharacterCreationResponse>, BladeApiError> {
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
    if characters_result.len() > 0 {
        // a character already exist
        return Err(BladeApiError::new(StatusCode::FORBIDDEN, 1101, 3));
    }
    //TODO: make sure the user name, or at least the tag id, is unique. Good luck getting it to work with the current (lack of) transaction model. An extra unique key in the table?
    let mut new_complete_data = PersistedCharacterData::default();
    new_complete_data.user_id = session.session.user_id;
    new_complete_data.character.name = body.name.clone();
    new_complete_data.character.data.customization = body.0.data.customization;
    //TODO: default inventory
    let character_uuid = Uuid::new_v4();
    let mut client = app_state.db_pool.get().await.unwrap();
    //TODO: convert error
    app_state
        .character_storage
        .add_new_character(client.as_mut(), character_uuid, new_complete_data)
        .await
        .unwrap();
    //TODO: convert error
    let character = app_state
        .character_storage
        .get(character_uuid, client.as_mut())
        .await
        .unwrap();
    Ok(web::Json(CharacterCreationResponse {
        character: CompleteCharacterWithId {
            id: character_uuid,
            character: character.character.clone(),
        },
        inventory: character.inventory.clone(),
    }))
}
