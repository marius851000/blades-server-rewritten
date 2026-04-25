use std::sync::Arc;

use actix_web::{
    post,
    web::{self, Json},
};
use blades_lib::user_data::CompleteCharacterWithIdWithoutData;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{AsyncConnection, RunQueryDsl, scoped_futures::ScopedFutureExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    BladeApiError, ServerGlobal, models::CharacterDbEntryCharacterAlone, schema::characters,
    session::SessionLookedUpMaybe,
};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetAvatarRequest {
    avatar_icon_id: Uuid,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetAvatarResponse {
    character: CompleteCharacterWithIdWithoutData,
}

#[post("/blades.bgs.services/api/game/v1/public/characters/{character_id}/avatar")]
pub async fn set_avatar(
    path: web::Path<Uuid>,
    session: SessionLookedUpMaybe,
    app_state: web::Data<Arc<ServerGlobal>>,
    body: web::Json<SetAvatarRequest>,
) -> Result<Json<SetAvatarResponse>, BladeApiError> {
    let session = session.get_session_or_error()?;
    let character_id = path.into_inner();
    let mut conn = app_state.db_pool.get().await.unwrap();

    conn.transaction(|mut conn| {
        async move {
            let mut character = characters::table
                .filter(characters::user_id.eq(session.session.user_id))
                .filter(characters::id.eq(character_id))
                .select(CharacterDbEntryCharacterAlone::as_select())
                .for_no_key_update()
                .load(&mut conn)
                .await?
                .into_iter()
                .next()
                .expect("TODO: proper error handling for not found/not permitted");

            character.character.0.avatar_icon_id = Some(body.avatar_icon_id);

            let result = Json(SetAvatarResponse {
                character: CompleteCharacterWithIdWithoutData {
                    id: character_id,
                    character: character.character.0.clone(),
                },
            });

            diesel::update(characters::table)
                .filter(characters::id.eq(character_id))
                .set(character)
                .execute(&mut conn)
                .await?;

            Ok(result)
        }
        .scope_boxed()
    })
    .await
}
