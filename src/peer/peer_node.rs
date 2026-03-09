use crate::peer::identity::PeerIdentity;

pub struct Peer {
    pub identity: PeerIdentity,
    /// Reputation score: 0–100. Drops to 0 triggers a ban.
    pub reputation: u32,
    pub is_banned: bool,
    /// Unix timestamp of the last observed activity from this peer
    pub last_seen_unix_sec: u64,
    /// Bitmask: 0x01 = BLE, 0x02 = WiFi-Direct
    pub supported_transports: u8,
    pub is_relay_node: bool,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

impl Peer {
    pub fn new(pubkey: [u8; 32]) -> Self {
        Self {
            identity: PeerIdentity::new(pubkey),
            reputation: 100,
            is_banned: false,
            last_seen_unix_sec: 0,
            supported_transports: 0,
            is_relay_node: false,
            bytes_sent: 0,
            bytes_received: 0,
        }
    }
}
