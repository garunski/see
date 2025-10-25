//! Database connection and schema management

pub mod connection;
pub mod schema;

// Re-export public types
pub use connection::DatabasePool;
pub use schema::initialize_schema;
