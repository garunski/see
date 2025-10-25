//! Simple in-memory store for now

use crate::errors::CoreError;
use std::collections::HashMap;
use std::sync::Mutex;

pub struct SimpleStore {
    data: Mutex<HashMap<String, String>>,
}

impl SimpleStore {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(HashMap::new()),
        }
    }

    pub fn save(&self, key: String, value: String) -> Result<(), CoreError> {
        let mut data = self.data.lock()
            .map_err(|e| CoreError::MutexLock(format!("Failed to lock store: {}", e)))?;
        data.insert(key, value);
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Option<String>, CoreError> {
        let data = self.data.lock()
            .map_err(|e| CoreError::MutexLock(format!("Failed to lock store: {}", e)))?;
        Ok(data.get(key).cloned())
    }
}