use std::sync::Arc;

use crate::{
    models::CharacterDbEntryWallet, schema, util::get_only_single_character_and_check_permission,
};
use actix_web::{
    get,
    web::{self, Json},
};
use blades_lib::user_data::CompleteWallet;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use serde::Serialize;
use uuid::Uuid;

use crate::{BladeApiError, ServerGlobal, session::SessionLookedUpMaybe};

#[derive(Serialize)]
struct GetWalletResponse {
    wallet: CompleteWallet,
}

#[get("/blades.bgs.services/api/game/v1/public/characters/{character_id}/wallets/current")]
pub async fn get_wallet(
    session: SessionLookedUpMaybe,
    app_state: web::Data<Arc<ServerGlobal>>,
    path: web::Path<Uuid>,
) -> Result<Json<GetWalletResponse>, BladeApiError> {
    let session = session.get_session_or_error()?;
    let character_id = path.into_inner();
    let mut conn = app_state.db_pool.get().await.unwrap();
    let data_entry = {
        use schema::characters::dsl::*;
        characters
            .filter(id.eq(character_id))
            .select(CharacterDbEntryWallet::as_select())
            .load(&mut conn)
            .await
            .unwrap()
    };

    let character = get_only_single_character_and_check_permission(data_entry, &session.session)?;

    Ok(Json(GetWalletResponse {
        wallet: character.wallet.0,
    }))
}
