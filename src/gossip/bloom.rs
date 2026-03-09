use bloomfilter::Bloom;

pub struct MessageFilter {
    filter: Bloom<[u8; 32]>,
}

impl MessageFilter {
    /// Create a new filter optimized for `capacity` items with `false_positive_rate`
    pub fn new(capacity: usize, false_positive_rate: f64) -> Self {
        Self {
            filter: Bloom::new_for_fp_rate(capacity, false_positive_rate),
        }
    }

    /// Returns true if the message is PROBABLY already seen.
    /// Returns false if the message is DEFINITELY new.
    pub fn check_and_add(&mut self, message_id: &[u8; 32]) -> bool {
        if self.filter.check(message_id) {
            true
        } else {
            self.filter.set(message_id);
            false
        }
    }
}

pub struct SlidingBloomFilter {
    current: Bloom<[u8; 32]>,
    previous: Bloom<[u8; 32]>,
    capacity: usize,
    fp_rate: f64,
    insert_count: usize,
}

impl SlidingBloomFilter {
    pub fn new(capacity_per_window: usize, fp_rate: f64) -> Self {
        Self {
            current: Bloom::new_for_fp_rate(capacity_per_window, fp_rate),
            previous: Bloom::new_for_fp_rate(capacity_per_window, fp_rate),
            capacity: capacity_per_window,
            fp_rate,
            insert_count: 0,
        }
    }

    pub fn check_and_add(&mut self, message_id: &[u8; 32]) -> bool {
        if self.current.check(message_id) || self.previous.check(message_id) {
            true
        } else {
            self.rotate_if_full();
            self.current.set(message_id);
            self.insert_count += 1;
            false
        }
    }

    fn rotate_if_full(&mut self) {
        if self.insert_count >= self.capacity {
            let new_filter = Bloom::new_for_fp_rate(self.capacity, self.fp_rate);
            self.previous = std::mem::replace(&mut self.current, new_filter);
            self.insert_count = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_filter() {
        let mut filter = MessageFilter::new(100, 0.01);
        let msg1 = [1u8; 32];
        let msg2 = [2u8; 32];

        assert!(!filter.check_and_add(&msg1));
        assert!(filter.check_and_add(&msg1));
        assert!(!filter.check_and_add(&msg2));
        assert!(filter.check_and_add(&msg2));
    }

    #[test]
    fn test_sliding_bloom_filter_rotation() {
        let mut filter = SlidingBloomFilter::new(10, 0.01);

        // Add 10 items to fill current filter
        for i in 0..10 {
            let mut msg = [0u8; 32];
            msg[0] = i as u8;
            assert!(!filter.check_and_add(&msg));
        }

        // Next item triggers rotation — don't assert false (bloom filters are probabilistic)
        let mut msg11 = [0u8; 32];
        msg11[0] = 11;
        filter.check_and_add(&msg11);

        // Old items should still be recognized (they are now in previous)
        let mut msg0 = [0u8; 32];
        msg0[0] = 0;
        assert!(filter.check_and_add(&msg0));

        // Add 10 more items to cause another rotation (12..22 = items 12 to 21 inclusive)
        // Note: we don't assert false on each since bloom filters have a small false-positive
        // rate. We just drive the rotation logic.
        for i in 12..22 {
            let mut msg = [0u8; 32];
            msg[0] = i as u8;
            filter.check_and_add(&msg);
        }

        // After the second rotation, insert_count resets to 1 (the last item of the batch
        // that triggered the rotate).
        assert_eq!(filter.insert_count, 1);
    }

    #[test]
    fn test_false_positive_rate() {
        let mut filter = SlidingBloomFilter::new(1000, 0.05);
        let mut false_positives = 0;

        for i in 0..1000u32 {
            let mut msg = [0u8; 32];
            let bytes = i.to_le_bytes();
            msg[0..4].copy_from_slice(&bytes);
            filter.check_and_add(&msg);
        }

        for i in 1000..2000u32 {
            let mut msg = [0u8; 32];
            let bytes = i.to_le_bytes();
            msg[0..4].copy_from_slice(&bytes);
            if filter.check_and_add(&msg) {
                // Was incorrectly marked as seen (false positive) since it's the first check for this ID
                false_positives += 1;
            }
        }

        let fp_rate = false_positives as f64 / 1000.0;
        assert!(fp_rate <= 0.10, "False positive rate too high: {}", fp_rate);
    }
}
