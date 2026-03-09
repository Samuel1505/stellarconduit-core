//! In-memory stub for `MeshDatabase` (Issue #26 placeholder).
//!
//! This stub satisfies `StatePruner`'s interface without requiring SQLite.
//! When Issue #26 (feat/persistence-sqlite) is merged, this file should be
//! replaced by the real SQLite-backed implementation — no other files need
//! to change, since the public API surface is identical.

use std::sync::Arc;
use tokio::sync::Mutex;

/// A minimal in-memory stub for the persistent mesh database.
///
/// Replace this with the real SQLite implementation from Issue #26.
pub type PendingMessageRecord = ([u8; 32], u64);

pub struct MeshDatabase {
    /// Tracks pubkeys marked offline (for test assertions).
    offline_peers: Arc<Mutex<Vec<[u8; 32]>>>,
    /// Pending messages: (message_id, unix_sec_timestamp).
    pending_messages: Arc<Mutex<Vec<PendingMessageRecord>>>,
}

impl MeshDatabase {
    /// Create a new in-memory stub (no SQLite required).
    pub fn new_stub() -> Self {
        Self {
            offline_peers: Arc::new(Mutex::new(Vec::new())),
            pending_messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Mark a peer as offline in the database.
    pub async fn mark_peer_offline(&self, pubkey: [u8; 32]) {
        self.offline_peers.lock().await.push(pubkey);
    }

    /// Insert a pending message with a unix-second timestamp (for tests).
    pub async fn insert_pending_message(&self, message_id: [u8; 32], unix_sec: u64) {
        self.pending_messages
            .lock()
            .await
            .push((message_id, unix_sec));
    }

    /// Delete all pending messages older than `cutoff_unix_sec`.
    /// Returns the number of messages deleted.
    pub async fn delete_messages_older_than(&self, cutoff_unix_sec: u64) -> usize {
        let mut msgs = self.pending_messages.lock().await;
        let before = msgs.len();
        msgs.retain(|(_, ts)| *ts >= cutoff_unix_sec);
        before - msgs.len()
    }

    /// Total number of pending messages currently tracked.
    pub async fn pending_message_count(&self) -> usize {
        self.pending_messages.lock().await.len()
    }

    /// Peers marked offline (for test assertions).
    pub async fn offline_peer_count(&self) -> usize {
        self.offline_peers.lock().await.len()
    }
}
