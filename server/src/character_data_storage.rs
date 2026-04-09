//TODO: this need some change. At least rename user to character here. And change the name. It is still too generic.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use anyhow::{Context, bail};
use blades_user_data::PersistedCharacterData;
use deadpool_postgres::{ClientWrapper, Pool};
use tokio::time::sleep;
use tokio_postgres::types::Json;
use uuid::Uuid;

pub struct CharacterFullDataLocalConfig {
    pub write_delay: u32, // in second
    pub db_pool: Arc<Pool>,
}

/// A guard that derefs to the inner T of an LazyEntry<T> assuming the value is already present.
pub struct CharacterFullDataLocalGuard {
    /// Assumed to be the Some variant. Will otherwise panic.
    inner: tokio::sync::OwnedMutexGuard<CharacterFullDataLocalEntry>,
    /// Used for planning persistance
    arc: Arc<tokio::sync::Mutex<CharacterFullDataLocalEntry>>,
}

impl std::ops::Deref for CharacterFullDataLocalGuard {
    type Target = PersistedCharacterData;
    fn deref(&self) -> &Self::Target {
        self.inner
            .value
            .as_ref()
            .expect("Value was not initialised")
    }
}

impl std::ops::DerefMut for CharacterFullDataLocalGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.plan_persistance(&self.arc);
        self.inner
            .value
            .as_mut()
            .expect("Value was not initialised")
    }
}

pub struct CharacterFullDataLocalEntry {
    value: Option<PersistedCharacterData>,
    #[allow(dead_code)]
    acquired_at: Instant,
    first_non_persisted_write_at: Option<Instant>,
    config: Arc<CharacterFullDataLocalConfig>,
}

impl CharacterFullDataLocalEntry {
    pub fn empty(config: Arc<CharacterFullDataLocalConfig>) -> Self {
        Self {
            value: None,
            acquired_at: Instant::now(),
            first_non_persisted_write_at: None,
            config,
        }
    }

    pub fn plan_persistance(
        &mut self,
        self_arc: &Arc<tokio::sync::Mutex<CharacterFullDataLocalEntry>>,
    ) {
        if self.first_non_persisted_write_at.is_none() {
            self.first_non_persisted_write_at = Some(Instant::now());

            let delay = self.config.write_delay;
            let entry_arc = self_arc.clone();

            // Plan persistance in the future
            tokio::spawn(async move {
                sleep(Duration::from_secs(delay as u64)).await;
                let value = {
                    let mut guard = entry_arc.lock().await;
                    guard.first_non_persisted_write_at = None;
                    guard.value.as_ref().expect("value not set").clone()
                };

                // Borrow the value for persistence without taking ownership.
                todo!();
            });
        }
    }
}

pub struct CharacterFullDataLocalStorage {
    storage: Mutex<HashMap<Uuid, Arc<tokio::sync::Mutex<CharacterFullDataLocalEntry>>>>,
    config: Arc<CharacterFullDataLocalConfig>,
}

impl CharacterFullDataLocalStorage {
    pub fn new(config: CharacterFullDataLocalConfig) -> Self {
        Self {
            storage: Mutex::new(HashMap::new()),
            config: Arc::new(config),
        }
    }

    pub async fn get(
        &self,
        id: Uuid,
        client: &mut ClientWrapper,
    ) -> anyhow::Result<CharacterFullDataLocalGuard> {
        let entry = {
            let mut lock = self.storage.lock().unwrap(); // this lock is not await-compatible
            lock.entry(id.clone())
                .or_insert_with(|| {
                    Arc::new(tokio::sync::Mutex::new(CharacterFullDataLocalEntry::empty(
                        self.config.clone(),
                    )))
                })
                .clone()
        };
        // the entry_lock should be locked from acquisition for initialisation up to the data being initialised (to avoid double loading of the data)
        let entry_cloned = entry.clone();
        let mut entry_lock = tokio::sync::Mutex::lock_owned(entry).await;

        if entry_lock.value.is_some() {
            return Ok(CharacterFullDataLocalGuard {
                inner: entry_lock,
                arc: entry_cloned,
            });
        } else {
            entry_lock.value = Some(
                self.load_from_source(client, id.clone())
                    .await
                    .with_context(|| {
                        format!("Trying to load the user {:?} from the database", id)
                    })?,
            );
            return Ok(CharacterFullDataLocalGuard {
                inner: entry_lock,
                arc: entry_cloned,
            });
        }
    }

    async fn load_from_source(
        &self,
        client: &mut ClientWrapper,
        id: Uuid,
    ) -> anyhow::Result<PersistedCharacterData> {
        let result = client
            .query("SELECT data FROM characters WHERE id = $1", &[&id])
            .await
            .context("looking up the character")?;
        if let Some(result) = result.get(0) {
            let result: Json<PersistedCharacterData> = result.get(0);
            Ok(result.0)
        } else {
            bail!("Character {:?} not found", id);
        }
    }

    // this also makes sure the user does not already have an id.
    pub async fn add_new_character(
        &self,
        client: &mut ClientWrapper,
        id: Uuid,
        data: PersistedCharacterData,
    ) -> anyhow::Result<()> {
        let transaction = client.transaction().await.unwrap();
        let already_existing_characters_for_user = transaction
            .query(
                "SELECT id FROM characters WHERE data->>'userId' = $1",
                &[&data.user_id.to_string()],
            )
            .await
            .unwrap();
        if already_existing_characters_for_user.len() > 0 {
            transaction.rollback().await.unwrap();
            bail!("user already have a character");
        }
        transaction
            .execute(
                "INSERT INTO characters (id, data) VALUES ($1, $2)",
                &[&id, &Json(data)],
            )
            .await
            .context("persisting a new user data")?;
        transaction.commit().await.unwrap();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn test_is_send_sync() {
        assert_send_sync::<CharacterFullDataLocalStorage>();
    }
}
