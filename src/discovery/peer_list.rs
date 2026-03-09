use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::discovery::events::DiscoveryEvent;
use crate::peer::peer_node::Peer;

pub struct PeerList {
    /// Maps public key to the Peer struct
    peers: HashMap<[u8; 32], Peer>,
    /// How many seconds before a peer is considered offline
    expiry_seconds: u64,
}

fn now_unix_sec() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

impl PeerList {
    pub fn new(expiry_seconds: u64) -> Self {
        Self {
            peers: HashMap::new(),
            expiry_seconds,
        }
    }

    /// Insert a new peer or update the last-seen timestamp for an existing one.
    /// Returns `PeerDiscovered` for first contact, `PeerUpdated` for subsequent contacts.
    pub fn insert_or_update(
        &mut self,
        pubkey: [u8; 32],
        signal_strength: u8,
    ) -> Option<DiscoveryEvent> {
        let now = now_unix_sec();

        if let Some(peer) = self.peers.get_mut(&pubkey) {
            peer.last_seen_unix_sec = now;
            Some(DiscoveryEvent::PeerUpdated(
                peer.identity.clone(),
                signal_strength,
            ))
        } else {
            let mut peer = Peer::new(pubkey);
            peer.last_seen_unix_sec = now;
            let identity = peer.identity.clone();
            self.peers.insert(pubkey, peer);
            Some(DiscoveryEvent::PeerDiscovered(identity))
        }
    }

    /// Returns only peers whose last-seen timestamp is within the expiry window.
    pub fn get_active_peers(&self) -> Vec<&Peer> {
        let now = now_unix_sec();
        self.peers
            .values()
            .filter(|p| now.saturating_sub(p.last_seen_unix_sec) <= self.expiry_seconds)
            .collect()
    }

    /// Removes stale peers (beyond expiry window) and returns a `PeerLost` event for each.
    pub fn prune_stale_peers(&mut self) -> Vec<DiscoveryEvent> {
        let now = now_unix_sec();
        let expiry = self.expiry_seconds;

        let stale_keys: Vec<[u8; 32]> = self
            .peers
            .iter()
            .filter(|(_, p)| now.saturating_sub(p.last_seen_unix_sec) > expiry)
            .map(|(k, _)| *k)
            .collect();

        stale_keys
            .into_iter()
            .filter_map(|key| {
                self.peers
                    .remove(&key)
                    .map(|p| DiscoveryEvent::PeerLost(p.identity))
            })
            .collect()
    }

    /// Returns total number of tracked peers (including stale ones not yet pruned).
    pub fn len(&self) -> usize {
        self.peers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.peers.is_empty()
    }

    /// Test helper: directly set last_seen_unix_sec for a peer by pubkey.
    pub fn set_last_seen(&mut self, pubkey: &[u8; 32], ts: u64) {
        if let Some(p) = self.peers.get_mut(pubkey) {
            p.last_seen_unix_sec = ts;
        }
    }
}

/// Background pruning stub — call this on a Tokio task to auto-prune every `interval_secs`.
/// The caller is responsible for wrapping `peer_list` in an `Arc<tokio::sync::Mutex<PeerList>>`.
pub async fn background_pruning_loop(
    peer_list: std::sync::Arc<tokio::sync::Mutex<PeerList>>,
    interval_secs: u64,
) {
    let interval = std::time::Duration::from_secs(interval_secs);
    loop {
        tokio::time::sleep(interval).await;
        let mut list = peer_list.lock().await;
        let lost = list.prune_stale_peers();
        if !lost.is_empty() {
            log::debug!("Pruned {} stale peer(s) from PeerList", lost.len());
        }
    }
}
