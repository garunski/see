// Settings repository - app settings operations

use crate::errors::CoreError;
use crate::persistence::models::AppSettings;
use crate::persistence::store::db_ops::DatabaseOperations;
use crate::persistence::store::serialization::{deserialize, serialize};
use redb::{ReadOnlyTable, Table, TableDefinition};

const SETTINGS_TABLE: &str = "settings";
const SETTINGS_DEF: TableDefinition<&str, &[u8]> = TableDefinition::new(SETTINGS_TABLE);

/// Repository for app settings operations
#[derive(Debug)]
pub struct SettingsRepository {
    db_ops: DatabaseOperations,
}

impl SettingsRepository {
    /// Create a new SettingsRepository
    pub fn new(db_ops: DatabaseOperations) -> Self {
        Self { db_ops }
    }

    /// Load app settings
    pub async fn load(&self) -> Result<Option<AppSettings>, CoreError> {
        self.db_ops
            .execute_read(|db| {
                let read_txn = db.begin_read()?;
                let settings_table: ReadOnlyTable<&str, &[u8]> =
                    read_txn.open_table(SETTINGS_DEF)?;
                if let Some(serialized) = settings_table.get("app_settings")? {
                    let settings: AppSettings = deserialize(serialized.value())?;
                    Ok(Some(settings))
                } else {
                    Ok(None)
                }
            })
            .await
    }

    /// Save app settings with retry logic
    pub async fn save(&self, settings: &AppSettings) -> Result<(), CoreError> {
        let settings = settings.clone();
        let db = self.db_ops.database().clone();

        self.db_ops
            .execute_write_with_retry(move || {
                tokio::task::block_in_place(|| {
                    let write_txn = db.begin_write()?;
                    {
                        let mut settings_table: Table<&str, &[u8]> =
                            write_txn.open_table(SETTINGS_DEF)?;
                        let serialized = serialize(&settings)?;
                        settings_table.insert("app_settings", serialized.as_slice())?;
                    }
                    write_txn.commit()?;
                    Ok(())
                })
            })
            .await
    }
}
