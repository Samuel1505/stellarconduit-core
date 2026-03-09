use rand::random;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub const CHUNK_FRAME_HEADER_SIZE: usize = 14;
pub const MAX_MESSAGE_SIZE_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, PartialEq)]
pub struct ChunkFrame {
    pub message_id: u32,
    pub total_length: u32,
    pub offset: u32,
    pub payload_size: u16,
    pub payload: Vec<u8>,
}

pub struct MessageChunker {
    pub mtu: usize,
}

impl MessageChunker {
    pub fn chunk(&self, message_bytes: &[u8]) -> Vec<ChunkFrame> {
        if message_bytes.is_empty() {
            return Vec::new();
        }

        if self.mtu <= CHUNK_FRAME_HEADER_SIZE {
            return Vec::new();
        }

        if message_bytes.len() > MAX_MESSAGE_SIZE_BYTES {
            return Vec::new();
        }

        let payload_capacity = self.mtu - CHUNK_FRAME_HEADER_SIZE;
        let payload_capacity = payload_capacity.min(u16::MAX as usize);
        if payload_capacity == 0 {
            return Vec::new();
        }

        let message_id = random::<u32>();
        let total_length = message_bytes.len() as u32;
        let mut frames = Vec::new();
        let mut offset = 0usize;

        while offset < message_bytes.len() {
            let end = (offset + payload_capacity).min(message_bytes.len());
            let payload = message_bytes[offset..end].to_vec();

            frames.push(ChunkFrame {
                message_id,
                total_length,
                offset: offset as u32,
                payload_size: payload.len() as u16,
                payload,
            });

            offset = end;
        }

        frames
    }
}

struct PartialMessageBuffer {
    total_length: usize,
    data: Vec<u8>,
    received_map: Vec<bool>,
    received_bytes: usize,
    last_updated: Instant,
}

pub struct MessageReassembler {
    buffers: HashMap<u32, PartialMessageBuffer>,
}

impl MessageReassembler {
    pub fn new() -> Self {
        Self {
            buffers: HashMap::new(),
        }
    }

    pub fn receive_chunk(&mut self, chunk: ChunkFrame) -> Option<Vec<u8>> {
        let total_length = chunk.total_length as usize;
        if total_length == 0 || total_length > MAX_MESSAGE_SIZE_BYTES {
            return None;
        }

        if usize::from(chunk.payload_size) != chunk.payload.len() {
            return None;
        }

        let start = chunk.offset as usize;
        let end = start.checked_add(chunk.payload.len())?;

        if start >= total_length || end > total_length {
            return None;
        }

        let buffer = self
            .buffers
            .entry(chunk.message_id)
            .or_insert_with(|| PartialMessageBuffer {
                total_length,
                data: vec![0u8; total_length],
                received_map: vec![false; total_length],
                received_bytes: 0,
                last_updated: Instant::now(),
            });

        if buffer.total_length != total_length {
            return None;
        }

        for (idx, byte) in (start..end).zip(chunk.payload.iter().copied()) {
            if !buffer.received_map[idx] {
                buffer.received_map[idx] = true;
                buffer.received_bytes += 1;
            }
            buffer.data[idx] = byte;
        }

        buffer.last_updated = Instant::now();

        if buffer.received_bytes == buffer.total_length {
            if let Some(completed) = self.buffers.remove(&chunk.message_id) {
                return Some(completed.data);
            }
        }

        None
    }

    pub fn cleanup_stale_buffers(&mut self, timeout_ms: u64) {
        let timeout = Duration::from_millis(timeout_ms);
        let now = Instant::now();
        self.buffers
            .retain(|_, buffer| now.duration_since(buffer.last_updated) <= timeout);
    }

    pub fn in_flight_buffer_count(&self) -> usize {
        self.buffers.len()
    }
}

impl Default for MessageReassembler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn chunker_slices_respecting_mtu() {
        let mtu = 32usize;
        let chunker = MessageChunker { mtu };
        let message: Vec<u8> = (0..100u8).collect();

        let chunks = chunker.chunk(&message);
        assert!(!chunks.is_empty());

        for chunk in &chunks {
            let frame_size = CHUNK_FRAME_HEADER_SIZE + chunk.payload.len();
            assert!(frame_size <= mtu);
            assert_eq!(usize::from(chunk.payload_size), chunk.payload.len());
            assert_eq!(chunk.total_length as usize, message.len());
        }

        let mut reassembler = MessageReassembler::new();
        let mut rebuilt = None;
        for chunk in chunks {
            if let Some(bytes) = reassembler.receive_chunk(chunk) {
                rebuilt = Some(bytes);
            }
        }

        assert_eq!(rebuilt, Some(message));
    }

    #[test]
    fn reassembler_handles_out_of_order_chunks() {
        let chunker = MessageChunker { mtu: 40 };
        let message: Vec<u8> = (0..200u16).map(|v| (v % 251) as u8).collect();
        let mut chunks = chunker.chunk(&message);

        assert!(chunks.len() >= 3);
        chunks.swap(0, 1);
        let len = chunks.len();
        chunks.swap(len - 1, len - 2);

        let mut reassembler = MessageReassembler::new();
        let mut rebuilt = None;

        for chunk in chunks {
            if let Some(bytes) = reassembler.receive_chunk(chunk) {
                rebuilt = Some(bytes);
            }
        }

        assert_eq!(rebuilt, Some(message));
    }

    #[test]
    fn stale_buffers_are_cleaned_up() {
        let mut reassembler = MessageReassembler::new();
        let chunk = ChunkFrame {
            message_id: 7,
            total_length: 10,
            offset: 0,
            payload_size: 4,
            payload: vec![1, 2, 3, 4],
        };

        assert_eq!(reassembler.receive_chunk(chunk), None);
        assert_eq!(reassembler.in_flight_buffer_count(), 1);

        thread::sleep(Duration::from_millis(20));
        reassembler.cleanup_stale_buffers(5);
        assert_eq!(reassembler.in_flight_buffer_count(), 0);
    }

    #[test]
    fn oversized_message_is_rejected() {
        let mut reassembler = MessageReassembler::new();
        let chunk = ChunkFrame {
            message_id: 1,
            total_length: (MAX_MESSAGE_SIZE_BYTES + 1) as u32,
            offset: 0,
            payload_size: 1,
            payload: vec![1],
        };

        assert_eq!(reassembler.receive_chunk(chunk), None);
        assert_eq!(reassembler.in_flight_buffer_count(), 0);
    }
}
