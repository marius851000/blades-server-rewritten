use std::{collections::HashMap, sync::Arc};

use actix_web::{http::StatusCode, post, web};
use blades_user_data::UserAccount;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{BladeApiError, ServerGlobal, session::Session};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AnonLoginInfo {
    user_id: Option<String>,
    device_id: String,
    platform: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionResponse {
    session: SessionResponseInner,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionResponseInner {
    session_id: String,
    user_id: String,
    token: String,
    schema: String,
    feature_status: u64,
    linked_accounts_status: u64,
    token_expiration_seconds: u64,
    denied_features: HashMap<String, DeniedFeatureResponse>,
}

impl SessionResponseInner {
    fn from_session(session_id: Uuid, session: &Session) -> Self {
        let mut denied_features = HashMap::new();
        denied_features.insert(
            "e3_signup_bonus".to_string(),
            DeniedFeatureResponse {
                deny_expired_secs: 0,
                deny_reason_code: 1,
            },
        );

        SessionResponseInner {
            session_id: session_id.to_string(),
            user_id: session.secret_user_id.to_string(),
            token: session.generate_token(&session_id),
            schema: "blades_v1".to_string(),
            feature_status: 7,
            linked_accounts_status: 4,
            token_expiration_seconds: session.expire_unix_timestamp,
            denied_features,
        }
    }
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DeniedFeatureResponse {
    deny_expired_secs: u64,
    deny_reason_code: u64,
}

#[post("/blades.bgs.services/api/authentication/v1/public/auth/anon")]
async fn anon_log_in(
    app_state: web::Data<Arc<ServerGlobal>>,
    info: web::Json<AnonLoginInfo>,
) -> Result<web::Json<SessionResponse>, BladeApiError> {
    if let Some(private_user_id) = info.0.user_id {
        // load pre-existing user
        // http code 404 service 3 error code 101 if not found, apparently
        let db = app_state.db_pool.get().await.unwrap();
        let result = db
            .query(
                "SELECT id, data FROM users WHERE data->>'secret_id' = $1",
                &[&private_user_id],
            )
            .await?;
        let (user_id, user): (Uuid, tokio_postgres::types::Json<UserAccount>) =
            if let Some(row) = result.get(0) {
                (row.get(0), row.get(1))
            } else {
                return Err(BladeApiError::new(StatusCode::NOT_FOUND, 3, 101)); // user not found
            };

        //TODO: some actual form of authentification.
        let session = Arc::new(Session::new(
            user_id,
            user.0.secret_id,
            app_state.session_store.ttl,
        ));
        let session_id = app_state.session_store.store_new_session(session.clone());
        return Ok(web::Json(SessionResponse {
            session: SessionResponseInner::from_session(session_id, session.as_ref()),
        }));
    } else {
        // create a new user
        let mut new_user = UserAccount::create_new_user();
        if info.0.platform == "gp" {
            new_user.gp_deviceids.insert(info.0.device_id);
        } else {
            return Err(BladeApiError::new(StatusCode::BAD_REQUEST, 3, 3)); //INVALID_REQUEST_DEVICE_ID
        }
        let new_user_id = Uuid::new_v4();
        let new_user_secret_id = new_user.secret_id;
        let db = app_state.db_pool.get().await.unwrap();
        db.execute(
            "INSERT INTO users (id, data) VALUES ($1, $2)",
            &[&new_user_id, &tokio_postgres::types::Json(new_user)],
        )
        .await?;

        let session = Arc::new(Session::new(
            new_user_id,
            new_user_secret_id,
            app_state.session_store.ttl,
        ));
        let session_id = app_state.session_store.store_new_session(session.clone());
        return Ok(web::Json(SessionResponse {
            session: SessionResponseInner::from_session(session_id, session.as_ref()),
        }));
    }
}
