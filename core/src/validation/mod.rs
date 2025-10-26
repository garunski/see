//! JSON Schema validation module for workflow definitions
//!
//! This module provides comprehensive validation of workflow JSON structures
//! using a manually-written JSON Schema (draft-07) with full function type constraints.

pub mod schema;
pub mod types;
pub mod validator;

pub use schema::{get_schema_version, load_workflow_schema};
pub use types::{ValidationError, ValidationErrors};
pub use validator::{validate_workflow_json, validate_workflow_json_simple};
