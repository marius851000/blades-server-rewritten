use actix_web::{
    post,
    web::{self, Json},
};
use blades_lib::user_data::CompleteCharacterWithIdWithoutData;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use serde::Serialize;
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

use crate::{
    BladeApiError, ServerGlobal, models::CharacterDbEntryCharacterAlone,
    session::SessionLookedUpMaybe, util::get_only_single_character_and_check_permission,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetChallengesResult {
    character: CompleteCharacterWithIdWithoutData,
    challenge_status: HashMap<u32, ()>,
}

#[post("/blades.bgs.services/api/game/v1/public/characters/{character_id}/challenges")]
pub async fn get_challenges(
    session: SessionLookedUpMaybe,
    app_state: web::Data<Arc<ServerGlobal>>,
    path: web::Path<Uuid>,
) -> Result<Json<GetChallengesResult>, BladeApiError> {
    //TODO: this return placeholder result (no challenges)
    let session = session.get_session_or_error()?;
    let character_id = path.into_inner();
    let mut conn = app_state.db_pool.get().await.unwrap();
    let data_entry = {
        use crate::schema::characters::dsl::*;
        characters
            .filter(id.eq(character_id))
            .select(CharacterDbEntryCharacterAlone::as_select())
            .load(&mut conn)
            .await
            .unwrap()
    };

    let character = get_only_single_character_and_check_permission(data_entry, &session.session)?;

    Ok(Json(GetChallengesResult {
        character: CompleteCharacterWithIdWithoutData {
            id: character_id,
            character: character.character.0,
        },
        challenge_status: HashMap::default(),
    }))
}
