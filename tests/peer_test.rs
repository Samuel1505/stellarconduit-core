use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use stellarconduit_core::peer::{
    identity::PeerIdentity,
    peer_node::Peer,
    reputation::{apply_penalty, apply_reward, PenaltyReason, RewardReason},
};

// ──── PeerIdentity tests ────────────────────────────────────────────────────

#[test]
fn test_peer_identity_display_id_is_hex() {
    let pubkey = [0xABu8; 32];
    let identity = PeerIdentity::new(pubkey);
    assert_eq!(
        identity.display_id.len(),
        64,
        "hex string should be 64 chars"
    );
    assert!(
        identity.display_id.chars().all(|c| c.is_ascii_hexdigit()),
        "display_id should be valid hex"
    );
}

#[test]
fn test_peer_identity_verify_valid_signature() {
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let pubkey = signing_key.verifying_key().to_bytes();
    let identity = PeerIdentity::new(pubkey);

    use ed25519_dalek::Signer;
    let message = b"test payload";
    let signature = signing_key.sign(message).to_bytes();

    assert!(identity.verify_signature(message, &signature));
}

#[test]
fn test_peer_identity_reject_bad_signature() {
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let pubkey = signing_key.verifying_key().to_bytes();
    let identity = PeerIdentity::new(pubkey);

    // Corrupted signature
    let bad_signature = [0u8; 64];
    assert!(!identity.verify_signature(b"test payload", &bad_signature));
}

// ──── Peer tests ────────────────────────────────────────────────────────────

#[test]
fn test_peer_initial_state() {
    let peer = Peer::new([1u8; 32]);
    assert_eq!(peer.reputation, 100);
    assert!(!peer.is_banned);
    assert_eq!(peer.bytes_sent, 0);
    assert_eq!(peer.bytes_received, 0);
    assert_eq!(peer.supported_transports, 0);
    assert!(!peer.is_relay_node);
}

#[test]
fn test_peer_transport_bitmask() {
    let mut peer = Peer::new([1u8; 32]);
    peer.supported_transports |= 0x01; // BLE
    peer.supported_transports |= 0x02; // WiFi-Direct
    assert_eq!(peer.supported_transports & 0x01, 0x01, "BLE should be set");
    assert_eq!(
        peer.supported_transports & 0x02,
        0x02,
        "WiFi-Direct should be set"
    );
}

// ──── Reputation tests ───────────────────────────────────────────────────────

#[test]
fn test_penalty_invalid_signature() {
    let mut peer = Peer::new([2u8; 32]);
    apply_penalty(&mut peer, PenaltyReason::InvalidSignature); // -20
    assert_eq!(peer.reputation, 80);
    assert!(!peer.is_banned);
}

#[test]
fn test_penalty_duplicate_message_flood() {
    let mut peer = Peer::new([2u8; 32]);
    apply_penalty(&mut peer, PenaltyReason::DuplicateMessageFlood); // -10
    assert_eq!(peer.reputation, 90);
}

#[test]
fn test_penalty_connection_dropped() {
    let mut peer = Peer::new([2u8; 32]);
    apply_penalty(&mut peer, PenaltyReason::ConnectionDropped); // -2
    assert_eq!(peer.reputation, 98);
}

#[test]
fn test_penalty_triggers_ban_at_zero() {
    let mut peer = Peer::new([3u8; 32]);
    // Drive reputation to 0 with InvalidSignature (-20 each)
    for _ in 0..5 {
        apply_penalty(&mut peer, PenaltyReason::InvalidSignature);
    }
    assert_eq!(peer.reputation, 0);
    assert!(
        peer.is_banned,
        "peer should be banned when reputation hits 0"
    );
}

#[test]
fn test_penalty_saturates_at_zero() {
    let mut peer = Peer::new([4u8; 32]);
    for _ in 0..20 {
        apply_penalty(&mut peer, PenaltyReason::InvalidSignature);
    }
    assert_eq!(peer.reputation, 0, "reputation should not underflow");
}

#[test]
fn test_reward_successfully_routed_tx() {
    let mut peer = Peer::new([5u8; 32]);
    apply_penalty(&mut peer, PenaltyReason::InvalidSignature); // reputation = 80
    apply_reward(&mut peer, RewardReason::SuccessfullyRoutedTx); // +5 → 85
    assert_eq!(peer.reputation, 85);
}

#[test]
fn test_reward_valid_gossip_envelope() {
    let mut peer = Peer::new([5u8; 32]);
    apply_penalty(&mut peer, PenaltyReason::DuplicateMessageFlood); // reputation = 90
    apply_reward(&mut peer, RewardReason::ValidNewGossipEnvelope); // +2 → 92
    assert_eq!(peer.reputation, 92);
}

#[test]
fn test_reward_caps_at_100() {
    let mut peer = Peer::new([6u8; 32]);
    // Already at 100, reward should not overflow
    apply_reward(&mut peer, RewardReason::SuccessfullyRoutedTx);
    apply_reward(&mut peer, RewardReason::ValidNewGossipEnvelope);
    assert_eq!(peer.reputation, 100, "reputation should not exceed 100");
}
