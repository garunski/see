#![allow(clippy::result_large_err)]
use crate::errors::CoreError;
use std::sync::{Mutex, MutexGuard};

pub struct SafeMutex<T> {
    inner: Mutex<T>,
}

impl<T> SafeMutex<T> {
    pub fn new(data: T) -> Self {
        Self {
            inner: Mutex::new(data),
        }
    }

    pub fn lock(&self) -> Result<MutexGuard<'_, T>, CoreError> {
        self.inner
            .lock()
            .map_err(|e| CoreError::MutexLock(format!("Lock failed: {}", e)))
    }
}

impl<T> From<Mutex<T>> for SafeMutex<T> {
    fn from(mutex: Mutex<T>) -> Self {
        Self { inner: mutex }
    }
}
