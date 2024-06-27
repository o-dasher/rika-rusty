use std::{collections::HashSet, hash::Hash, sync::Arc};

use poise::serenity_prelude::futures::lock::Mutex;
use strum::Display;
use thiserror::Error;

#[derive(Debug, Default)]
pub struct IDLocker<T: Hash + Eq + Clone>(Arc<Mutex<HashSet<T>>>);

#[derive(Error, Debug, Display)]
pub enum IDLockerError {
    AlreadyLocked,
    AlreadyUnlocked,
}

pub type IDLockerResult = Result<(), IDLockerError>;

#[derive(Debug)]
pub struct IDLockGuard<'a, T: Hash + Eq + Clone + Send> {
    locker: &'a IDLocker<T>,
    locking: T,
}

impl<T: Hash + Eq + Clone + Send> IDLockGuard<'_, T> {
    pub async fn unlock(self) -> IDLockerResult {
        self.locker
            .0
            .lock()
            .await
            .remove(&self.locking)
            .then_some(())
            .ok_or(IDLockerError::AlreadyUnlocked)
    }
}

impl<T: Hash + Eq + Clone + Send> IDLocker<T> {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(HashSet::new())))
    }

    pub async fn lock(&self, locking: T) -> Result<IDLockGuard<T>, IDLockerError> {
        self.0
            .lock()
            .await
            .insert(locking.clone())
            .then_some(IDLockGuard {
                locking,
                locker: self,
            })
            .ok_or(IDLockerError::AlreadyLocked)
    }
}
