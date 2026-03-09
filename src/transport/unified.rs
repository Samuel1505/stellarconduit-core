/// Re-exports for convenient access to the unified transport abstraction.
pub use crate::transport::connection::{Connection, ConnectionState, TransportType};
pub use crate::transport::errors::TransportError;

/// A type-erased, heap-allocated connection handle usable across transports.
pub type BoxedConnection = Box<dyn Connection>;
