#![allow(clippy::result_large_err)]
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("redb error: {0}")]
    Redb(#[from] redb::Error),

    #[error("redb database error: {0}")]
    RedbDatabase(#[from] redb::DatabaseError),

    #[error("redb transaction error: {0}")]
    RedbTransaction(#[from] redb::TransactionError),

    #[error("redb table error: {0}")]
    RedbTable(#[from] redb::TableError),

    #[error("redb storage error: {0}")]
    RedbStorage(#[from] redb::StorageError),

    #[error("redb commit error: {0}")]
    RedbCommit(#[from] redb::CommitError),

    #[error("dataflow error: {0}")]
    Dataflow(String),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("execution context error: {0}")]
    ExecutionContext(String),

    #[error("mutex lock error: {0}")]
    MutexLock(String),

    #[error("workflow execution error: {0}")]
    WorkflowExecution(String),

    #[error("task execution error: {0}")]
    TaskExecution(String),

    #[error("command execution error: {0}")]
    CommandExecution(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("serialization error: {0}")]
    Serialization(String),
}

impl From<Box<CoreError>> for CoreError {
    fn from(err: Box<CoreError>) -> Self {
        *err
    }
}
