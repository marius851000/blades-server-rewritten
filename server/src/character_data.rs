use std::sync::Arc;

use actix_web::{
    http::StatusCode,
    post,
    web::{Data, Json, Path},
};
use diesel::{
    ExpressionMethods, QueryDsl,
    dsl::jsonb_set_create_if_missing,
    sql_types::{Array, Text},
};
use diesel_async::{AsyncConnection, RunQueryDsl, scoped_futures::ScopedFutureExt};
use serde::Deserialize;
use serde_json::Value;
use serde_json::json;
use uuid::Uuid;

use crate::{BladeApiError, ServerGlobal, json_db::JsonDbWrapper, session::SessionLookedUpMaybe};

#[derive(Deserialize)]
struct DataUpdateRequest {
    data: DataUpdateRequestInner,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DataUpdateRequestInner {
    dialog: Option<Value>,
    customization: Option<()>, // not sure customization updateis sent this way
    #[serde(rename = "new-flags")]
    new_flags: Option<Value>,
}

#[post("/blades.bgs.services/api/game/v1/public/characters/{character_id}/data")]
async fn update_data(
    session: SessionLookedUpMaybe,
    app_state: Data<Arc<ServerGlobal>>,
    body: Json<DataUpdateRequest>,
    path: Path<Uuid>,
) -> Result<Json<Value>, BladeApiError> {
    if body.data.dialog.is_none() && body.data.new_flags.is_none() {
        return Ok(Json(json!(null)));
    }
    assert!(body.data.customization.is_none());
    let character_id = path.into_inner();
    let session = session.get_session_or_error()?;
    let mut conn = app_state.db_pool.get().await.unwrap();

    conn.transaction(|mut conn| {
        async move {
            use crate::schema::characters::dsl::*;

            // I can’t figure how to do that in a single request. And I can’t put a for_update in an update statement...

            // lock the row fo update (that would have been avoidable if I could put everything in a single transaction)
            characters
                .filter(id.eq(character_id))
                .filter(user_id.eq(session.session.user_id))
                .for_update()
                .execute(&mut conn)
                .await
                .unwrap();

            if let Some(new_flags) = body.0.data.new_flags {
                let new_data_updated_row = diesel::update(characters)
                    .filter(id.eq(character_id))
                    .filter(user_id.eq(session.session.user_id))
                    .set(
                        data.eq(jsonb_set_create_if_missing::<_, Array<Text>, _, _, _, _>(
                            data,
                            vec!["new-flags"],
                            JsonDbWrapper(new_flags),
                            true,
                        )),
                    )
                    .execute(&mut conn)
                    .await?;

                if new_data_updated_row == 0 {
                    return Err(BladeApiError::new(StatusCode::BAD_REQUEST, 1003, 2));
                }
            };

            if let Some(dialog) = body.0.data.dialog {
                let dialog_updated_row = diesel::update(characters)
                    .filter(id.eq(character_id))
                    .filter(user_id.eq(session.session.user_id))
                    .set(
                        data.eq(jsonb_set_create_if_missing::<_, Array<Text>, _, _, _, _>(
                            data,
                            vec!["dialog"],
                            JsonDbWrapper(dialog),
                            true,
                        )),
                    )
                    .execute(&mut conn)
                    .await?;

                if dialog_updated_row == 0 {
                    return Err(BladeApiError::new(StatusCode::BAD_REQUEST, 1003, 2));
                }
            }

            Ok(Json(json!(null)))
        }
        .scope_boxed()
    })
    .await
}
