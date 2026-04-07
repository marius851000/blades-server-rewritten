use log::error;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::time::Instant;
use uuid::Uuid;

pub struct Session {
    pub user_id: Uuid,
    pub secret_user_id: Uuid,
    pub extra_secret: Uuid, // a UUIDv4 just for added randomness
    pub expire_unix_timestamp: u64,
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
        }
    }

    pub fn generate_token(&self, session_id: &Uuid) -> String {
        format!("{}|{}", session_id, self.extra_secret)
    }
}

pub struct SessionStore {
    //TODO: eventually migrate to a parallel ordered map. A mutex per request seems pretty bad for performance.
    map: Mutex<BTreeMap<Uuid, Arc<Session>>>,
    /// TTL should be at least 1h30min, as that is the grace period used by session for its ttl returned to the client.
    pub ttl: Duration,
    time_base: Instant,
}

impl SessionStore {
    pub fn new(ttl: Duration) -> Self {
        Self {
            map: Mutex::new(BTreeMap::default()),
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
