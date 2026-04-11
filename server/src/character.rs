use std::sync::Arc;

use crate::{
    json_db::JsonDbWrapper,
    models::{CharacterDbEntry, CharacterDbEntryCharacterAndData},
    schema::{self, characters},
    util::get_only_single_character_and_check_permission,
};
use actix_web::{
    get, post,
    web::{self, Json},
};
use blades_user_data::{
    CompleteCharacter, CompleteCharacterWithIdAndData, CompleteData, CompleteInventory,
    CompleteWallet,
};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper, insert_into};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{BladeApiError, ServerGlobal, session::SessionLookedUpMaybe};

#[derive(Serialize)]
struct CharacterListResponse {
    characters: Vec<CompleteCharacterWithIdAndData>,
}

#[get("/blades.bgs.services/api/game/v1/public/characters")]
async fn list_characters(
    session: SessionLookedUpMaybe,
    app_state: web::Data<Arc<ServerGlobal>>,
) -> Result<web::Json<CharacterListResponse>, BladeApiError> {
    let session = session.get_session_or_error()?;
    let mut conn = app_state.db_pool.get().await.unwrap();
    let query_result = {
        use schema::characters::dsl::*;
        characters
            .filter(user_id.eq(session.session.user_id))
            .select(CharacterDbEntryCharacterAndData::as_select())
            .load(&mut conn)
            .await
            .unwrap()
    };

    let mut result = Vec::with_capacity(query_result.len());
    for character in query_result.iter() {
        result.push(CompleteCharacterWithIdAndData {
            id: character.id,
            character: character.character.0.clone(),
            data: character.data.0.clone(),
        });
    }
    Ok(web::Json(CharacterListResponse { characters: result }))
}

#[derive(Serialize)]
struct CompleteCharacterWithIdAndDataContainer {
    character: CompleteCharacterWithIdAndData,
}

#[get("/blades.bgs.services/api/game/v1/public/characters/{character_id}")]
async fn get_character(
    session: SessionLookedUpMaybe,
    app_state: web::Data<Arc<ServerGlobal>>,
    path: web::Path<Uuid>,
) -> Result<Json<CompleteCharacterWithIdAndDataContainer>, BladeApiError> {
    let session = session.get_session_or_error()?;
    let character_id = path.into_inner();
    let mut conn = app_state.db_pool.get().await.unwrap();
    let character_entries = {
        use schema::characters::dsl::*;
        characters
            .filter(id.eq(character_id))
            .select(CharacterDbEntryCharacterAndData::as_select())
            .load(&mut conn)
            .await
            .unwrap()
    };

    let character =
        get_only_single_character_and_check_permission(character_entries, &session.session)?;

    Ok(Json(CompleteCharacterWithIdAndDataContainer {
        character: CompleteCharacterWithIdAndData {
            id: character_id,
            character: character.character.0.clone(),
            data: character.data.0.clone(),
        },
    }))
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
    character: CompleteCharacterWithIdAndData,
    inventory: CompleteInventory,
}

#[post("/blades.bgs.services/api/game/v1/public/characters")]
async fn create_characters(
    session: SessionLookedUpMaybe,
    app_state: web::Data<Arc<ServerGlobal>>,
    body: web::Json<CharacterCreationRequest>,
) -> Result<web::Json<CharacterCreationResponse>, BladeApiError> {
    let session = session.get_session_or_error()?;

    //TODO: make sure the user name, or at least the tag id, is unique. Good luck getting it to work with the current (lack of) transaction model. An extra unique key in the table?
    let mut new_character = CompleteCharacter::default();
    new_character.name = body.name.clone();

    let mut new_data = CompleteData::default();
    new_data.customization = body.0.data.customization;

    let character_uuid = Uuid::new_v4();

    let to_insert = CharacterDbEntry {
        id: character_uuid,
        user_id: session.session.user_id,
        character: JsonDbWrapper(new_character),
        data: JsonDbWrapper(new_data),
        wallet: JsonDbWrapper(CompleteWallet::default()),
    };

    let mut conn = app_state.db_pool.get().await.unwrap();
    //TODO: convert error
    //TODO: explicit no async commit (start a new transaction)
    insert_into(characters::table)
        .values(&to_insert)
        .execute(&mut conn)
        .await
        .unwrap();

    Ok(web::Json(CharacterCreationResponse {
        character: CompleteCharacterWithIdAndData {
            id: character_uuid,
            character: to_insert.character.0,
            data: to_insert.data.0,
        },
        //TODO: actually handle the inventory (including the default loadout)
        inventory: CompleteInventory::default(),
    }))
}
