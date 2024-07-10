use std::{collections::HashSet, hash::Hash, sync::Mutex};

use strum::Display;
use thiserror::Error;

pub trait IDLockable: Send + Clone + Eq + Hash {}
impl<T: Send + Clone + Eq + Hash> IDLockable for T {}

#[derive(Debug, Default)]
pub struct IDLocker<T: IDLockable>(Mutex<HashSet<T>>);

#[derive(Error, Debug, Display)]
pub enum IDLockerError {
    AlreadyLocked,
    AlreadyUnlocked,
}

pub type IDLockerResult = Result<(), IDLockerError>;

#[derive(Debug)]
pub struct IDLockGuard<'a, T: IDLockable> {
    locker: &'a IDLocker<T>,
    locking: T,
}

impl<T: IDLockable> Drop for IDLockGuard<'_, T> {
    fn drop(&mut self) {
        let _ = self.unchecked_unlock();
    }
}

impl<T: IDLockable> IDLockGuard<'_, T> {
    fn unchecked_unlock(&self) -> IDLockerResult {
        self.locker
            .0
            .lock()
            .ok()
            .ok_or(IDLockerError::AlreadyUnlocked)?
            .remove(&self.locking)
            .then_some(())
            .ok_or(IDLockerError::AlreadyUnlocked)
    }

    pub fn unlock(self) -> IDLockerResult {
        self.unchecked_unlock()
    }
}

impl<T: IDLockable> IDLocker<T> {
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
