use alloc::collections::BTreeMap;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLockReadGuard};

use super::{DbKey, Lock};

/// Read-only db
#[derive(Clone)]
pub struct DbReader<V> {
    pub(super) inner: Lock<V>,
}

impl<V> DbReader<V> {
    pub async fn read(&self) -> DbReadView<'_, V> {
        DbReadView { guard: self.inner.read().await }
    }
}

// can avoid clone here maybe?
// we would need to hold a long-lived reference to the db reader
// which would prevent writes to avoid cloning
// cloning small strings should generally be OK? as long as they aren't leaked
// somehow.
impl<V: Clone> DbReader<V> {
    pub async fn daemon_url(&self) -> Option<V> {
        let read = self.read().await;
        read.get(&super::Keys::DURL).cloned()
    }

    pub async fn ssid(&self) -> Option<V> {
        let read = self.read().await;
        read.get(&super::Keys::SSID).cloned()
    }

    pub async fn wifi_pw(&self) -> Option<V> {
        let read = self.read().await;
        read.get(&super::Keys::WIPW).cloned()
    }
}

/// A view into the db.
pub struct DbReadView<'a, V> {
    pub(super) guard: RwLockReadGuard<'a, CriticalSectionRawMutex, BTreeMap<DbKey, V>>,
}

impl<V> DbReadView<'_, V> {
    pub fn get(&self, key: &DbKey) -> Option<&V> {
        self.guard.get(key)
    }
}
