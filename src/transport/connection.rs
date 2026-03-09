use async_trait::async_trait;

use crate::message::types::ProtocolMessage;
use crate::peer::identity::PeerIdentity;
use crate::transport::errors::TransportError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Disconnecting,
    Failed(TransportError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportType {
    Ble,
    WifiDirect,
}

#[async_trait]
pub trait Connection: Send + Sync {
    /// Retrieve the peer on the other side of this connection
    fn remote_peer(&self) -> PeerIdentity;

    /// Which transport is currently being used
    fn transport_type(&self) -> TransportType;

    /// Current connection state
    fn state(&self) -> ConnectionState;

    /// Attempt to establish the physical connection
    async fn connect(&mut self) -> Result<(), TransportError>;

    /// Send a serialized protocol message. Returns an error if not connected or if IO fails.
    async fn send(&mut self, msg: ProtocolMessage) -> Result<(), TransportError>;

    /// Block until a message is received from this peer
    async fn recv(&mut self) -> Result<ProtocolMessage, TransportError>;

    /// Safely close the connection
    async fn disconnect(&mut self) -> Result<(), TransportError>;
}
