use crate::peer::peer_node::Peer;

pub enum PenaltyReason {
    InvalidSignature,
    DuplicateMessageFlood,
    ConnectionDropped,
}

pub enum RewardReason {
    SuccessfullyRoutedTx,
    ValidNewGossipEnvelope,
}

impl PenaltyReason {
    fn points(&self) -> u32 {
        match self {
            PenaltyReason::InvalidSignature => 20,
            PenaltyReason::DuplicateMessageFlood => 10,
            PenaltyReason::ConnectionDropped => 2,
        }
    }
}

impl RewardReason {
    fn points(&self) -> u32 {
        match self {
            RewardReason::SuccessfullyRoutedTx => 5,
            RewardReason::ValidNewGossipEnvelope => 2,
        }
    }
}

pub fn apply_penalty(peer: &mut Peer, reason: PenaltyReason) {
    let penalty = reason.points();
    peer.reputation = peer.reputation.saturating_sub(penalty);
    if peer.reputation == 0 {
        peer.is_banned = true;
    }
}

pub fn apply_reward(peer: &mut Peer, reason: RewardReason) {
    let reward = reason.points();
    peer.reputation = (peer.reputation + reward).min(100);
}
