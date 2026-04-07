//TODO: this need some change. At least rename user to character here. And change the name. It is still too generic.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use anyhow::Context;
use tokio::time::sleep;

use crate::UserId;

pub struct LazyWriterAndCacheConfig {
    write_delay: u32,
}

impl LazyWriterAndCacheConfig {
    pub fn new() -> Self {
        Self { write_delay: 30 }
    }

    pub fn with_write_delay(&mut self, write_delay: u32) -> &mut Self {
        self.write_delay = write_delay;
        self
    }
}

/// A guard that derefs to the inner T of an LazyEntry<T> assuming the value is already present.
pub struct LazyEntryGuard<T: Send + 'static> {
    /// Assumed to be the Some variant. Will otherwise panic.
    inner: tokio::sync::OwnedMutexGuard<LazyEntry<T>>,
    /// Used for planning persistance
    arc: Arc<tokio::sync::Mutex<LazyEntry<T>>>,
}

impl<T: Send + 'static> std::ops::Deref for LazyEntryGuard<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner
            .value
            .as_ref()
            .expect("Value was not initialised")
    }
}

impl<T: Send + 'static> std::ops::DerefMut for LazyEntryGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.plan_persistance(&self.arc);
        self.inner
            .value
            .as_mut()
            .expect("Value was not initialised")
    }
}

pub struct LazyEntry<T: Send + 'static> {
    value: Option<T>,
    #[allow(dead_code)]
    acquired_at: Instant,
    first_non_persisted_write_at: Option<Instant>,
    config: Arc<LazyWriterAndCacheConfig>,
}

impl<T: Send + 'static> LazyEntry<T> {
    pub fn empty(config: Arc<LazyWriterAndCacheConfig>) -> Self {
        Self {
            value: None,
            acquired_at: Instant::now(),
            first_non_persisted_write_at: None,
            config,
        }
    }

    pub fn plan_persistance(&mut self, self_arc: &Arc<tokio::sync::Mutex<LazyEntry<T>>>) {
        if self.first_non_persisted_write_at.is_none() {
            self.first_non_persisted_write_at = Some(Instant::now());

            let delay = self.config.write_delay;
            let entry_arc = self_arc.clone();

            // Plan persistance in the future
            tokio::spawn(async move {
                sleep(Duration::from_secs(delay as u64)).await;
                let guard = entry_arc.lock().await;

                // Borrow the value for persistence without taking ownership.
                let value = guard.value.as_ref().expect("value not set");
                todo!();

                guard.first_non_persisted_write_at = None;
            });
        }
    }
}

pub struct LazyWriterAndCache<T: Send + 'static> {
    storage: Mutex<HashMap<UserId, Arc<tokio::sync::Mutex<LazyEntry<T>>>>>,
    config: Arc<LazyWriterAndCacheConfig>,
}

impl<T: Send> LazyWriterAndCache<T> {
    pub fn new(config: LazyWriterAndCacheConfig) -> Self {
        Self {
            storage: Mutex::new(HashMap::new()),
            config: Arc::new(config),
        }
    }

    pub async fn load_user_data(&self, id: UserId) -> anyhow::Result<LazyEntryGuard<T>> {
        let entry = {
            let mut lock = self.storage.lock().unwrap(); // this lock is not await-compatible
            lock.entry(id.clone())
                .or_insert_with(|| {
                    Arc::new(tokio::sync::Mutex::new(LazyEntry::empty(
                        self.config.clone(),
                    )))
                })
                .clone()
        };
        // the entry_lock should be locked from acquisition for initialisation up to the data being initialised (to avoid double loading of the data)
        let entry_cloned = entry.clone();
        let mut entry_lock = tokio::sync::Mutex::lock_owned(entry).await;

        if entry_lock.value.is_some() {
            return Ok(LazyEntryGuard {
                inner: entry_lock,
                arc: entry_cloned,
            });
        } else {
            entry_lock.value =
                Some(self.load_from_source(id.clone()).await.with_context(|| {
                    format!("Trying to load the user {:?} from the database", id)
                })?);
            return Ok(LazyEntryGuard {
                inner: entry_lock,
                arc: entry_cloned,
            });
        }
    }

    async fn load_from_source(&self, id: UserId) -> anyhow::Result<T> {
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use blades_user_data::UserSavedData;

    use super::*;

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn test_is_send_sync() {
        assert_send_sync::<LazyWriterAndCache<UserSavedData>>();
    }
}
