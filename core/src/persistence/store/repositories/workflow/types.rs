// Types and constants for workflow repository operations

use redb::TableDefinition;

// Table names
pub const EXECUTIONS_TABLE: &str = "executions";
pub const EXECUTION_IDS_TABLE: &str = "execution_ids";
pub const TASKS_TABLE: &str = "tasks";

// Table definitions
pub const EXECUTIONS_DEF: TableDefinition<&str, &[u8]> = TableDefinition::new(EXECUTIONS_TABLE);
pub const EXECUTION_IDS_DEF: TableDefinition<&str, &str> =
    TableDefinition::new(EXECUTION_IDS_TABLE);
pub const TASKS_DEF: TableDefinition<&str, &[u8]> = TableDefinition::new(TASKS_TABLE);

/// Helper struct for common table operations
#[derive(Debug)]
pub struct TableContext<'a> {
    pub executions_table: redb::Table<'a, &'static str, &'static [u8]>,
    pub execution_ids_table: redb::Table<'a, &'static str, &'static str>,
    pub tasks_table: redb::Table<'a, &'static str, &'static [u8]>,
}
