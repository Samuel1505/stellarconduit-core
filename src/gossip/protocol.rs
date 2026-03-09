//! Gossip protocol event loop.

use tokio::time::sleep;
use std::time::Duration;

use crate::gossip::round::{
    GossipScheduler, ACTIVE_ROUND_INTERVAL_MS, IDLE_ROUND_INTERVAL_MS,
};

pub async fn run_gossip_loop(mut scheduler: GossipScheduler) {
    loop {
        if scheduler.is_time_for_round() {
            // TODO: select fanout peers and broadcast buffered messages
            log::debug!("Gossip round fired");
            scheduler.round_executed();
        }

        let wait_ms = if scheduler.is_idle() {
            IDLE_ROUND_INTERVAL_MS
        } else {
            ACTIVE_ROUND_INTERVAL_MS
        };

        sleep(Duration::from_millis(wait_ms)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_gossip_loop_starts_without_blocking() {
        let scheduler = GossipScheduler::new();
        let handle = tokio::spawn(run_gossip_loop(scheduler));
        let result = timeout(Duration::from_millis(200), async {
            tokio::time::sleep(Duration::from_millis(100)).await;
        })
        .await;
        assert!(result.is_ok());
        handle.abort();
    }

    #[tokio::test]
    async fn test_gossip_loop_can_be_aborted() {
        let scheduler = GossipScheduler::new();
        let handle = tokio::spawn(run_gossip_loop(scheduler));
        tokio::time::sleep(Duration::from_millis(50)).await;
        handle.abort();
        let _ = handle.await;
    }

    #[tokio::test]
    async fn test_gossip_loop_starts_in_idle_mode() {
        use crate::gossip::round::IDLE_TIMEOUT_SEC;
        let mut scheduler = GossipScheduler::new();
        scheduler.last_active_msg_time =
            std::time::Instant::now() - Duration::from_secs(IDLE_TIMEOUT_SEC + 5);
        assert!(scheduler.is_idle());
        let handle = tokio::spawn(run_gossip_loop(scheduler));
        let result = timeout(Duration::from_millis(150), async {
            tokio::time::sleep(Duration::from_millis(50)).await;
        })
        .await;
        assert!(result.is_ok());
        handle.abort();
    }
}
