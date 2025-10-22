// Dedicated metadata query service with filtering and sorting

use crate::errors::CoreError;
use crate::persistence::models::WorkflowMetadata;
use crate::persistence::store::db_ops::DatabaseOperations;
use crate::persistence::store::serialization::deserialize;
use chrono::{DateTime, Utc};
use redb::{ReadOnlyTable, ReadableTable};
use std::time::SystemTime;
use tracing::instrument;

use super::query_types::{SortOrder, WorkflowQueryOptions, WorkflowSortField};
use super::types::EXECUTIONS_DEF;

/// Service for querying workflow metadata with advanced filtering and sorting
#[derive(Debug)]
pub struct MetadataQueryService {
    db_ops: DatabaseOperations,
}

impl MetadataQueryService {
    /// Create a new metadata query service
    pub fn new(db_ops: DatabaseOperations) -> Self {
        Self { db_ops }
    }

    /// Execute metadata query with filtering and sorting
    #[instrument(skip(self))]
    pub async fn query_metadata(
        &self,
        options: WorkflowQueryOptions,
    ) -> Result<Vec<WorkflowMetadata>, CoreError> {
        let db_ops = self.db_ops.clone();
        db_ops
            .execute_read(move |db| {
                let read_txn = db.begin_read()?;
                let workflows_table: ReadOnlyTable<&str, &[u8]> =
                    read_txn.open_table(EXECUTIONS_DEF)?;

                let mut metadata_list = Vec::new();
                let mut count = 0;
                let offset = options.offset.unwrap_or(0);

                for item in workflows_table.iter()? {
                    if count >= offset + options.limit.unwrap_or(usize::MAX) {
                        break;
                    }

                    let (key, value) = item?;
                    if key.value().starts_with("workflow:") {
                        let metadata: WorkflowMetadata = deserialize(value.value())?;

                        // Apply filters
                        if !Self::matches_filters(&metadata, &options) {
                            count += 1;
                            continue;
                        }

                        if count >= offset {
                            metadata_list.push(metadata);
                        }
                        count += 1;
                    }
                }

                // Apply sorting
                Self::apply_sorting(&mut metadata_list, &options);

                Ok(metadata_list)
            })
            .await
    }

    /// Check if metadata matches all applied filters
    fn matches_filters(metadata: &WorkflowMetadata, options: &WorkflowQueryOptions) -> bool {
        // Status filter
        if let Some(status) = &options.status_filter {
            if &metadata.status != status {
                return false;
            }
        }

        // Name filter
        if let Some(name_filter) = &options.workflow_name_filter {
            if !metadata.workflow_name.contains(name_filter) {
                return false;
            }
        }

        // Time filters
        if let Some(after) = &options.start_time_after {
            if let Ok(metadata_time) = metadata.start_timestamp.parse::<DateTime<Utc>>() {
                let metadata_system_time: SystemTime = metadata_time.into();
                if metadata_system_time < *after {
                    return false;
                }
            } else {
                return false; // Invalid timestamp format
            }
        }

        if let Some(before) = &options.start_time_before {
            if let Ok(metadata_time) = metadata.start_timestamp.parse::<DateTime<Utc>>() {
                let metadata_system_time: SystemTime = metadata_time.into();
                if metadata_system_time > *before {
                    return false;
                }
            } else {
                return false; // Invalid timestamp format
            }
        }

        true
    }

    /// Apply sorting to the metadata list
    fn apply_sorting(metadata_list: &mut [WorkflowMetadata], options: &WorkflowQueryOptions) {
        match options.sort_by {
            WorkflowSortField::StartTime => {
                metadata_list.sort_by(|a, b| match options.sort_order {
                    SortOrder::Descending => b.start_timestamp.cmp(&a.start_timestamp),
                });
            }
        }
    }
}
