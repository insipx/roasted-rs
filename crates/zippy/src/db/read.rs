use super::{Lock, DbKey};
use embassy_sync::rwlock::RwLockReadGuard;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use alloc::collections::BTreeMap;

/// Read-only db
pub struct DbReader<V> {
    pub(super) inner: Lock<V>,
}

impl<V> DbReader<V> {
    pub async fn read(&self) -> DbReadView<'_, V> {
        DbReadView { guard: self.inner.read().await }
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
