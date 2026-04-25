use actix_web::{FromRequest, get, http::StatusCode, web};
use log::error;
use serde::Serialize;
use std::{
    collections::BTreeMap,
    future::{self, ready},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    sync::{Mutex, mpsc::UnboundedSender},
    time::Instant,
};
use uuid::Uuid;

use crate::{BladeApiError, ServerGlobal, arena::MatchmakingMessage};

pub struct Session {
    pub user_id: Uuid,
    pub secret_user_id: Uuid,
    pub extra_secret: Uuid, // a UUIDv4 just for added randomness
    pub expire_unix_timestamp: u64,
    // incremented each (connected) request by the middleware
    pub request_count: AtomicU64,
    pub matchmaking_ws: Mutex<Option<UnboundedSender<MatchmakingMessage>>>,
}

impl Session {
    pub fn new(user_id: Uuid, secret_user_id: Uuid, ttl: Duration) -> Self {
        Self {
            user_id,
            secret_user_id,
            expire_unix_timestamp: match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(duration) => (duration + ttl - ttl / 10).as_secs(),
                Err(e) => {
                    error!(
                        "Oh no! In Session, it seems we are before the unix timestamp! Defaulting to ttl to 0. Error is {:?}",
                        e
                    );
                    (ttl - ttl / 10).as_secs()
                }
            },
            extra_secret: Uuid::new_v4(),
            request_count: AtomicU64::new(1),
            matchmaking_ws: Mutex::new(None),
        }
    }

    pub fn generate_token(&self, session_id: &Uuid) -> String {
        format!("{}|{}", session_id, self.extra_secret)
    }
}

//TODO: FromRequest for this SessionLookupUp
pub struct SessionLookedUp {
    #[allow(unused)]
    pub session_id: Uuid,
    pub session: Arc<Session>,
}

// Read the session from the Authorization header
pub struct SessionLookedUpMaybe(Option<SessionLookedUp>);

impl SessionLookedUpMaybe {
    pub fn get_session_or_error(&self) -> Result<&SessionLookedUp, BladeApiError> {
        self.0
            .as_ref()
            .ok_or_else(|| BladeApiError::new(StatusCode::UNAUTHORIZED, 3, 43))
    }
}

impl FromRequest for SessionLookedUpMaybe {
    type Error = actix_web::Error;
    type Future = future::Ready<Result<Self, Self::Error>>;

    //TODO: use BladeApiError instead
    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let authorization = if let Some(authorization) = req.headers().get("Authorization") {
            if let Ok(token) = authorization.to_str() {
                token
            } else {
                return ready(Err(actix_web::error::ErrorBadRequest(
                    "Authorization header can’t be parsed as str",
                )));
            }
        } else {
            return ready(Ok(SessionLookedUpMaybe(None)));
        };

        let token = if let Some(token) = authorization.split('=').skip(1).next() {
            token
        } else {
            return ready(Err(actix_web::error::ErrorBadRequest(
                "Authorization can’t be parsed (no equal sign present)",
            )));
        };

        let mut token_splitted = token.split('|');
        let (session_id, extra_secret) = if let Some(session_id) = token_splitted.next()
            && let Some(extra_secret) = token_splitted.next()
        {
            let session_id = match Uuid::parse_str(session_id) {
                Ok(v) => v,
                Err(_err) => {
                    return ready(Err(actix_web::error::ErrorBadRequest(
                        "can’t parse session id part of the token",
                    )));
                }
            };
            let extra_secret = match Uuid::parse_str(extra_secret) {
                Ok(v) => v,
                Err(_err) => {
                    return ready(Err(actix_web::error::ErrorBadRequest(
                        "can’t parse extra secret part of the token",
                    )));
                }
            };
            (session_id, extra_secret)
        } else {
            return ready(Err(actix_web::error::ErrorBadRequest(
                "Invalid token format (no |)",
            )));
        };

        let global = req
            .app_data::<web::Data<Arc<ServerGlobal>>>()
            .expect("server global not in app_data (for extracting a Session)");
        let session = if let Some(v) = global.session_store.get(session_id) {
            v
        } else {
            // most likely the token expired. Should we return something specific in such case?
            return ready(Ok(SessionLookedUpMaybe(None)));
        };
        if session.extra_secret == extra_secret {
            return ready(Ok(SessionLookedUpMaybe(Some(SessionLookedUp {
                session_id,
                session,
            }))));
        } else {
            return ready(Err(actix_web::error::ErrorUnauthorized(
                "Invalid token (extra secret mismatch)",
            )));
        }
    }
}

pub struct SessionStore {
    //TODO: eventually migrate to a parallel ordered map. A mutex per request seems pretty bad for performance.
    map: std::sync::Mutex<BTreeMap<Uuid, Arc<Session>>>,
    /// TTL should be at least 1h30min, as that is the grace period used by session for its ttl returned to the client.
    pub ttl: Duration,
    time_base: Instant,
}

impl SessionStore {
    pub fn new(ttl: Duration) -> Self {
        Self {
            map: std::sync::Mutex::new(BTreeMap::default()),
            ttl,
            time_base: Instant::now(),
        }
    }

    /// While extremly unlikely, it might generate an already existing key. Another one should be requested in such case.
    /// The UUID encode time since self.time_base in its first 64 bytes (BE-encoded for sorting)
    fn get_uuid_for_instant(&self, future_instant: &Instant) -> Uuid {
        let t = future_instant
            .duration_since(self.time_base)
            .as_secs()
            .to_be_bytes();
        let r: [u8; 8] = rand::random();
        let bytes = [
            t[0], t[1], t[2], t[3], t[4], t[5], t[6], t[7], r[0], r[1], r[2], r[3], r[4], r[5],
            r[6], r[7],
        ];
        Uuid::new_v8(bytes)
    }

    #[allow(unused)]
    pub fn extract_creation_instant(&self, uuid: Uuid) -> Option<Instant> {
        let bytes = uuid.as_bytes();
        let ts_bytes: [u8; 8] = bytes[0..8].try_into().ok()?;
        let secs = u64::from_be_bytes(ts_bytes);
        Some(self.time_base + Duration::from_secs(secs))
    }

    pub fn get(&self, session_id: Uuid) -> Option<Arc<Session>> {
        self.map.lock().unwrap().get(&session_id).cloned()
    }

    pub fn store_new_session(&self, session: Arc<Session>) -> Uuid {
        let now_instant = Instant::now();
        let clear_before_instant = now_instant - self.ttl;
        let uuid_to_clear_before = self.get_uuid_for_instant(&clear_before_instant);

        let mut id = self.get_uuid_for_instant(&now_instant);
        {
            let mut locked = self.map.lock().unwrap();

            while locked.get(&id).is_some() {
                id = self.get_uuid_for_instant(&now_instant);
            }
            locked.insert(id.clone(), session);

            while let Some((k, _v)) = locked.first_key_value()
                && k < &uuid_to_clear_before
            {
                locked.pop_first();
            }
        }
        return id;
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SyncResponse {
    request_index: u64,
}

#[get("/blades.bgs.services/api/game/v1/public/sync")]
async fn sync(session: SessionLookedUpMaybe) -> Result<web::Json<SyncResponse>, BladeApiError> {
    let session = session.get_session_or_error()?;
    Ok(web::Json(SyncResponse {
        request_index: session
            .session
            .request_count
            .load(Ordering::Relaxed)
            .saturating_sub(1), // the counter is incremented before processing the variable. This may cause issue if multiple request from the client are made simulteneously, thought.
    }))
}
