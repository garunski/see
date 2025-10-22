// Clean query types for workflow operations

use crate::persistence::models::WorkflowStatus;
use std::time::SystemTime;

/// Query options for workflow metadata with modern, focused API
#[derive(Debug, Clone)]
pub struct WorkflowQueryOptions {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub status_filter: Option<WorkflowStatus>,
    pub workflow_name_filter: Option<String>,
    pub start_time_after: Option<SystemTime>,
    pub start_time_before: Option<SystemTime>,
    pub sort_by: WorkflowSortField,
    pub sort_order: SortOrder,
}

impl Default for WorkflowQueryOptions {
    fn default() -> Self {
        Self {
            limit: Some(50),
            offset: Some(0),
            status_filter: None,
            workflow_name_filter: None,
            start_time_after: None,
            start_time_before: None,
            sort_by: WorkflowSortField::StartTime,
            sort_order: SortOrder::Descending,
        }
    }
}

/// Fields available for sorting workflow metadata
#[derive(Debug, Clone, PartialEq)]
pub enum WorkflowSortField {
    StartTime,
}

/// Sort order options
#[derive(Debug, Clone, PartialEq)]
pub enum SortOrder {
    Descending,
}

/// Builder pattern for creating query options
impl WorkflowQueryOptions {
    /// Create a new query options builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the limit for results
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the offset for pagination
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Set sort field and order
    pub fn with_sort(mut self, field: WorkflowSortField, order: SortOrder) -> Self {
        self.sort_by = field;
        self.sort_order = order;
        self
    }
}
