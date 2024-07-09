use std::{collections::HashSet, hash::Hash, sync::Mutex};

use strum::Display;
use thiserror::Error;

#[derive(Debug, Default)]
pub struct IDLocker<T: Hash + Eq + Clone>(Mutex<HashSet<T>>);

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

impl<T: Hash + Eq + Clone + Send> Drop for IDLockGuard<'_, T> {
    fn drop(&mut self) {
        let _ = self.borrowed_unlock();
    }
}

impl<T: Hash + Eq + Clone + Send> IDLockGuard<'_, T> {
    fn borrowed_unlock(&mut self) -> IDLockerResult {
        self.locker
            .0
            .lock()
            .ok()
            .ok_or(IDLockerError::AlreadyUnlocked)?
            .remove(&self.locking)
            .then_some(())
            .ok_or(IDLockerError::AlreadyUnlocked)
    }

    pub fn unlock(mut self) -> IDLockerResult {
        self.borrowed_unlock()
    }
}

impl<T: Hash + Eq + Clone + Send> IDLocker<T> {
    #[must_use]
    pub fn new() -> Self {
        Self(Mutex::new(HashSet::new()))
    }

    pub fn lock(&self, locking: T) -> Result<IDLockGuard<T>, IDLockerError> {
        self.0
            .lock()
            .ok()
            .ok_or(IDLockerError::AlreadyLocked)?
            .insert(locking.clone())
            .then_some(IDLockGuard {
                locking,
                locker: self,
            })
            .ok_or(IDLockerError::AlreadyLocked)
    }
}
