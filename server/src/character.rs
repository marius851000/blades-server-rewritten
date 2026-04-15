use std::{collections::HashMap, str::FromStr, sync::Arc};

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
use blades_lib::user_data::{
    Backpack, CompleteCharacter, CompleteCharacterData, CompleteCharacterWithIdAndData,
    CompleteInventory, CompleteWallet, EquippedItems, Item, ItemPropertiesAll, Loadout,
    SingleEquippedItem, Treasury,
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

    let mut new_data = CompleteCharacterData::default();
    new_data.customization = body.0.data.customization;

    let character_uuid = Uuid::new_v4();

    let mut equipped_items = HashMap::new();
    let item1_slot_uuid = Uuid::from_str("417e79de-c810-42f8-8273-f9759df6ae25").unwrap();
    equipped_items.insert(
        item1_slot_uuid,
        SingleEquippedItem {
            id: Uuid::new_v4(),
            slot: item1_slot_uuid,
            item: Item {
                item_template_id: Uuid::from_str("606c8bf6-9dc7-4c5f-b44b-36eb02306c96").unwrap(),
                durability: 75.0,
                tempering_level: 0,
                properties: ItemPropertiesAll::default(),
            },
        },
    );

    let item2_slot_uuid = Uuid::from_str("862605de-c67f-4bce-b527-4e5fb6f25162").unwrap();
    equipped_items.insert(
        item2_slot_uuid,
        SingleEquippedItem {
            id: Uuid::new_v4(),
            slot: item2_slot_uuid,
            item: Item {
                item_template_id: Uuid::from_str("c6f7fab4-eadc-4e8c-bf7f-e0ea095a3acf").unwrap(),
                tempering_level: 0,
                durability: 100.0,
                properties: ItemPropertiesAll::default(),
            },
        },
    );

    let item3_slot_uuid = Uuid::from_str("897a600c-91d6-4449-af09-173da88a907e").unwrap();
    equipped_items.insert(
        item3_slot_uuid,
        SingleEquippedItem {
            id: Uuid::new_v4(),
            slot: item3_slot_uuid,
            item: Item {
                item_template_id: Uuid::from_str("42b6fad8-5ac9-4215-aeff-133715c4c22e").unwrap(),
                durability: 0.0,
                tempering_level: 0,
                properties: ItemPropertiesAll::default(),
            },
        },
    );

    let item4_slot_uuid = Uuid::from_str("e273a4d7-fb87-4f7e-8f1e-398be59afbcb").unwrap();
    equipped_items.insert(
        item4_slot_uuid,
        SingleEquippedItem {
            id: Uuid::new_v4(),
            slot: item4_slot_uuid,
            item: Item {
                item_template_id: Uuid::from_str("2571f818-6ae4-4355-b89a-4a6253089e6c").unwrap(),
                tempering_level: 0,
                durability: 0.0,
                properties: ItemPropertiesAll::default(),
            },
        },
    );

    let inventory = CompleteInventory {
        backpack: Backpack::default(),
        loadout: Loadout {
            equipped_items: EquippedItems(equipped_items),
        },
        treasury: Treasury::default(),
        overflow_treasury: Treasury::default(),
        backpack_version: 1,
        treasury_version: 0,
    };

    let to_insert = CharacterDbEntry {
        id: character_uuid,
        user_id: session.session.user_id,
        character: JsonDbWrapper(new_character),
        data: JsonDbWrapper(new_data),
        wallet: JsonDbWrapper(CompleteWallet::default()),
        inventory: JsonDbWrapper(inventory.clone()),
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
        inventory,
    }))
}
