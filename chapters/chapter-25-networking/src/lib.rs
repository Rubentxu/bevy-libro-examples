// Capítulo 25. Networking — Packet structure, serialization, interpolation
use std::collections::VecDeque;

/// Network packet: a message sent between client and server
#[derive(Clone, Debug, PartialEq)]
pub struct Packet {
    pub sequence: u32,
    pub timestamp: u64,
    pub data: Vec<u8>,
}

impl Packet {
    pub fn new(sequence: u32, timestamp: u64, data: Vec<u8>) -> Self {
        Self { sequence, timestamp, data }
    }

    /// Serialize packet to bytes (simplified)
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.sequence.to_le_bytes());
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes.extend_from_slice(&(self.data.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&self.data);
        bytes
    }

    /// Deserialize packet from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 16 {
            return Err("Packet too short".to_string());
        }
        let sequence = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let timestamp = u64::from_le_bytes(bytes[4..12].try_into().unwrap());
        let data_len = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;

        if bytes.len() < 16 + data_len {
            return Err("Data length mismatch".to_string());
        }
        let data = bytes[16..16 + data_len].to_vec();

        Ok(Self { sequence, timestamp, data })
    }
}

/// Reliable delivery: track ack'd packets and resend if needed
pub struct ReliableChannel {
    next_sequence: u32,
    pending_acks: VecDeque<(u32, Packet)>,
    acked_sequences: Vec<u32>,
    max_pending: usize,
}

impl ReliableChannel {
    pub fn new(max_pending: usize) -> Self {
        Self {
            next_sequence: 0,
            pending_acks: VecDeque::new(),
            acked_sequences: Vec::new(),
            max_pending,
        }
    }

    pub fn send(&mut self, data: Vec<u8>, timestamp: u64) -> Packet {
        let packet = Packet::new(self.next_sequence, timestamp, data);
        self.next_sequence += 1;
        self.pending_acks.push_back((packet.sequence, packet.clone()));
        packet
    }

    pub fn ack(&mut self, sequence: u32) {
        self.pending_acks.retain(|(seq, _)| *seq != sequence);
        self.acked_sequences.push(sequence);
    }

    pub fn pending_count(&self) -> usize {
        self.pending_acks.len()
    }

    pub fn get_pending(&self) -> Vec<&Packet> {
        self.pending_acks.iter().map(|(_, p)| p).collect()
    }

    pub fn can_send(&self) -> bool {
        self.pending_acks.len() < self.max_pending
    }
}

/// Lag compensation: interpolate between received states
pub struct StateInterpolation<T: Clone> {
    states: VecDeque<(u64, T)>,
    max_delay: usize,
}

impl<T: Clone> StateInterpolation<T> {
    pub fn new(max_delay: usize) -> Self {
        Self {
            states: VecDeque::with_capacity(max_delay),
            max_delay,
        }
    }

    pub fn push(&mut self, timestamp: u64, state: T) {
        self.states.push_back((timestamp, state));
        while self.states.len() > self.max_delay {
            self.states.pop_front();
        }
    }

    pub fn get_at(&self, timestamp: u64) -> Option<&T> {
        self.states
            .iter()
            .find(|(t, _)| *t == timestamp)
            .map(|(_, s)| s)
    }

    pub fn latest(&self) -> Option<&T> {
        self.states.back().map(|(_, s)| s)
    }

    pub fn oldest_timestamp(&self) -> Option<u64> {
        self.states.front().map(|(t, _)| *t)
    }
}

/// Ping/RTT calculator
pub struct PingTracker {
    samples: VecDeque<f32>,
    max_samples: usize,
}

impl PingTracker {
    pub fn new(max_samples: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(max_samples),
            max_samples,
        }
    }

    pub fn add_sample(&mut self, rtt_ms: f32) {
        if self.samples.len() >= self.max_samples {
            self.samples.pop_front();
        }
        self.samples.push_back(rtt_ms);
    }

    pub fn average(&self) -> f32 {
        if self.samples.is_empty() { return 0.0; }
        let sum: f32 = self.samples.iter().sum();
        sum / self.samples.len() as f32
    }

    pub fn jitter(&self) -> f32 {
        if self.samples.len() < 2 { return 0.0; }
        let avg = self.average();
        let variance: f32 = self.samples.iter()
            .map(|s| (s - avg).powi(2))
            .sum::<f32>() / self.samples.len() as f32;
        variance.sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packet_serialize_roundtrip() {
        let original = Packet::new(42, 1234567890, vec![1, 2, 3, 4, 5]);
        let bytes = original.to_bytes();
        let restored = Packet::from_bytes(&bytes).unwrap();

        assert_eq!(restored, original);
    }

    #[test]
    fn packet_empty_data() {
        let original = Packet::new(0, 0, vec![]);
        let bytes = original.to_bytes();
        let restored = Packet::from_bytes(&bytes).unwrap();
        assert_eq!(restored, original);
    }

    #[test]
    fn packet_invalid_data() {
        let result = Packet::from_bytes(&[0, 1, 2]);
        assert!(result.is_err());
    }

    #[test]
    fn reliable_channel_send_and_ack() {
        let mut channel = ReliableChannel::new(10);
        let p1 = channel.send(vec![1], 0);
        let p2 = channel.send(vec![2], 1);

        assert_eq!(channel.pending_count(), 2);

        channel.ack(p1.sequence);
        assert_eq!(channel.pending_count(), 1);

        channel.ack(p2.sequence);
        assert_eq!(channel.pending_count(), 0);
    }

    #[test]
    fn reliable_channel_can_send() {
        let mut channel = ReliableChannel::new(3);
        assert!(channel.can_send());

        channel.send(vec![1], 0);
        channel.send(vec![2], 0);
        channel.send(vec![3], 0);
        assert!(!channel.can_send(), "Should be at max pending");
    }

    #[test]
    fn state_interpolation_push_get() {
        let mut interp = StateInterpolation::<u32>::new(10);
        interp.push(100, 42);
        interp.push(200, 84);

        assert_eq!(interp.get_at(100), Some(&42));
        assert_eq!(interp.get_at(200), Some(&84));
        assert_eq!(interp.get_at(999), None);
    }

    #[test]
    fn state_interpolation_latest() {
        let mut interp = StateInterpolation::<u32>::new(10);
        interp.push(100, 1);
        interp.push(200, 2);
        interp.push(300, 3);

        assert_eq!(interp.latest(), Some(&3));
    }

    #[test]
    fn state_interpolation_evicts_old() {
        let mut interp = StateInterpolation::<u32>::new(2);
        interp.push(100, 1);
        interp.push(200, 2);
        interp.push(300, 3);

        assert_eq!(interp.oldest_timestamp(), Some(200), "Should have evicted timestamp 100");
    }

    #[test]
    fn ping_tracker_average() {
        let mut tracker = PingTracker::new(10);
        tracker.add_sample(50.0);
        tracker.add_sample(60.0);
        tracker.add_sample(70.0);

        assert!((tracker.average() - 60.0).abs() < 0.001);
    }

    #[test]
    fn ping_tracker_jitter() {
        let mut tracker = PingTracker::new(10);
        // All same = 0 jitter
        tracker.add_sample(50.0);
        tracker.add_sample(50.0);
        tracker.add_sample(50.0);
        assert!((tracker.jitter() - 0.0).abs() < 0.001);

        // Varying = non-zero jitter
        tracker.add_sample(100.0);
        assert!(tracker.jitter() > 0.0);
    }

    #[test]
    fn ping_tracker_rolling_window() {
        let mut tracker = PingTracker::new(3);
        tracker.add_sample(10.0);
        tracker.add_sample(20.0);
        tracker.add_sample(30.0);
        tracker.add_sample(40.0);

        // Should only have last 3 samples: 20, 30, 40
        assert!((tracker.average() - 30.0).abs() < 0.001);
    }
}
