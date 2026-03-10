//! Operations, transactions, undo/redo for s1engine.
//!
//! All document mutations flow through this crate. Direct tree manipulation
//! is never allowed — this is the foundation for undo/redo and future CRDT support.
//!
//! # Architecture
//!
//! ```text
//! Operation         — atomic unit of change
//! Transaction       — groups operations into one undo step
//! History           — undo/redo stacks of transactions
//! Position/Selection — cursor and selection representation
//! ```
//!
//! Every `Operation::apply()` returns its inverse, enabling trivial undo.

pub mod cursor;
pub mod history;
pub mod operation;
pub mod transaction;

// Re-export primary types at crate root.
pub use cursor::{Position, Selection};
pub use history::History;
pub use operation::{apply, validate, Operation, OperationError};
pub use transaction::{apply_transaction, Transaction, TransactionBuilder};
