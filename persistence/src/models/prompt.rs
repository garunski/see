use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Prompt {
    pub id: String,
    pub name: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl Default for Prompt {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::new(),
            content: String::new(),
            created_at: now,
        }
    }
}

impl Prompt {
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Prompt ID cannot be empty".to_string());
        }
        if self.name.is_empty() {
            return Err("Prompt name cannot be empty".to_string());
        }
        if self.content.is_empty() {
            return Err("Prompt content cannot be empty".to_string());
        }

        Ok(())
    }

    pub fn update_content(&mut self, content: String) {
        self.content = content;
    }

    pub fn update_name(&mut self, name: String) {
        self.name = name;
    }
}
