use async_trait::async_trait;
use stellarconduit_core::{
    message::types::{ProtocolMessage, TransactionEnvelope},
    peer::identity::PeerIdentity,
    transport::{
        connection::{Connection, ConnectionState, TransportType},
        errors::TransportError,
    },
};

// ──── MockConnection ─────────────────────────────────────────────────────────
// A minimal in-memory implementation of Connection for testing the trait bounds.

struct MockConnection {
    peer: PeerIdentity,
    state: ConnectionState,
    /// Messages queued to be returned on recv()
    inbox: Vec<ProtocolMessage>,
}

impl MockConnection {
    fn new(pubkey: [u8; 32]) -> Self {
        Self {
            peer: PeerIdentity::new(pubkey),
            state: ConnectionState::Disconnected,
            inbox: Vec::new(),
        }
    }

    fn enqueue_message(&mut self, msg: ProtocolMessage) {
        self.inbox.push(msg);
    }
}

#[async_trait]
impl Connection for MockConnection {
    fn remote_peer(&self) -> PeerIdentity {
        self.peer.clone()
    }

    fn transport_type(&self) -> TransportType {
        TransportType::Ble
    }

    fn state(&self) -> ConnectionState {
        self.state
    }

    async fn connect(&mut self) -> Result<(), TransportError> {
        self.state = ConnectionState::Connected;
        Ok(())
    }

    async fn send(&mut self, _msg: ProtocolMessage) -> Result<(), TransportError> {
        if self.state != ConnectionState::Connected {
            return Err(TransportError::NotConnected);
        }
        Ok(())
    }

    async fn recv(&mut self) -> Result<ProtocolMessage, TransportError> {
        if self.state != ConnectionState::Connected {
            return Err(TransportError::NotConnected);
        }
        self.inbox.pop().ok_or(TransportError::BrokenPipe)
    }

    async fn disconnect(&mut self) -> Result<(), TransportError> {
        self.state = ConnectionState::Disconnected;
        Ok(())
    }
}

// Helper to build a minimal TransactionEnvelope for testing
fn mock_tx_envelope() -> TransactionEnvelope {
    TransactionEnvelope {
        message_id: [0u8; 32],
        origin_pubkey: [1u8; 32],
        tx_xdr: "AAAA".to_string(),
        ttl_hops: 5,
        timestamp: 1_000_000,
        signature: [0u8; 64],
    }
}

// ──── State machine tests ────────────────────────────────────────────────────

#[tokio::test]
async fn test_initial_state_is_disconnected() {
    let conn = MockConnection::new([1u8; 32]);
    assert_eq!(conn.state(), ConnectionState::Disconnected);
}

#[tokio::test]
async fn test_connect_transitions_to_connected() {
    let mut conn = MockConnection::new([1u8; 32]);
    conn.connect().await.unwrap();
    assert_eq!(conn.state(), ConnectionState::Connected);
}

#[tokio::test]
async fn test_disconnect_transitions_to_disconnected() {
    let mut conn = MockConnection::new([1u8; 32]);
    conn.connect().await.unwrap();
    conn.disconnect().await.unwrap();
    assert_eq!(conn.state(), ConnectionState::Disconnected);
}

// ──── Send / Recv tests ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_send_succeeds_when_connected() {
    let mut conn = MockConnection::new([2u8; 32]);
    conn.connect().await.unwrap();
    let msg = ProtocolMessage::Transaction(mock_tx_envelope());
    assert!(conn.send(msg).await.is_ok());
}

#[tokio::test]
async fn test_send_fails_when_disconnected() {
    let mut conn = MockConnection::new([2u8; 32]);
    let msg = ProtocolMessage::Transaction(mock_tx_envelope());
    let err = conn.send(msg).await.unwrap_err();
    assert_eq!(err, TransportError::NotConnected);
}

#[tokio::test]
async fn test_recv_returns_queued_message() {
    let mut conn = MockConnection::new([3u8; 32]);
    conn.connect().await.unwrap();
    let msg = ProtocolMessage::Transaction(mock_tx_envelope());
    conn.enqueue_message(msg.clone());
    let received = conn.recv().await.unwrap();
    assert_eq!(received, msg);
}

#[tokio::test]
async fn test_recv_returns_broken_pipe_when_inbox_empty() {
    let mut conn = MockConnection::new([3u8; 32]);
    conn.connect().await.unwrap();
    let err = conn.recv().await.unwrap_err();
    assert_eq!(err, TransportError::BrokenPipe);
}

#[tokio::test]
async fn test_recv_fails_when_disconnected() {
    let mut conn = MockConnection::new([3u8; 32]);
    let err = conn.recv().await.unwrap_err();
    assert_eq!(err, TransportError::NotConnected);
}

// ──── Metadata tests ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_remote_peer_identity_matches_pubkey() {
    let pubkey = [42u8; 32];
    let conn = MockConnection::new(pubkey);
    assert_eq!(conn.remote_peer().pubkey, pubkey);
}

#[tokio::test]
async fn test_transport_type_is_ble() {
    let conn = MockConnection::new([1u8; 32]);
    assert_eq!(conn.transport_type(), TransportType::Ble);
}

// ──── Send + Sync bound test ─────────────────────────────────────────────────

#[test]
fn test_connection_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<MockConnection>();
}

// ──── ConnectionState equality and Failed variant ────────────────────────────

#[test]
fn test_connection_state_failed_variant() {
    let s = ConnectionState::Failed(TransportError::Timeout);
    assert_eq!(s, ConnectionState::Failed(TransportError::Timeout));
    assert_ne!(s, ConnectionState::Failed(TransportError::BrokenPipe));
}
