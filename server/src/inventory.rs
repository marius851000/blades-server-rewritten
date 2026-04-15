use std::sync::Arc;

use actix_web::{
    get,
    web::{self, Json},
};
use blades_lib::user_data::CompleteInventory;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    BladeApiError, ServerGlobal, models::CharacterDbEntryInventory, session::SessionLookedUpMaybe,
    util::get_only_single_character_and_check_permission,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct InventoryQuery {
    consumable_stackable_items_only: bool,
    equipped_items_only: bool,
}

#[derive(Serialize, Deserialize)]
struct GetInventoryResponse {
    inventory: CompleteInventory,
}

#[get("/blades.bgs.services/api/game/v1/public/character/{character_id}/inventories/current")]
pub async fn get_inventory(
    session: SessionLookedUpMaybe,
    app_state: web::Data<Arc<ServerGlobal>>,
    path: web::Path<Uuid>,
    query: web::Query<InventoryQuery>,
) -> Result<Json<GetInventoryResponse>, BladeApiError> {
    //TODO: handle query parameters
    assert!(query.consumable_stackable_items_only == false && query.equipped_items_only == false);
    let session = session.get_session_or_error()?;
    let character_id = path.into_inner();
    let mut conn = app_state.db_pool.get().await.unwrap();

    let inventory_result = {
        use crate::schema::characters::dsl::*;
        characters
            .filter(id.eq(character_id))
            .select(CharacterDbEntryInventory::as_select())
            .load(&mut conn)
            .await
            .unwrap()
    };

    let inventory =
        get_only_single_character_and_check_permission(inventory_result, &session.session)?;

    Ok(Json(GetInventoryResponse {
        inventory: inventory.inventory.0,
    }))
}
