/// High-performance RakNet sliding-window congestion control.
///
/// Implements slow start, congestion avoidance, dynamic RTT estimation
/// (Jacobson's algorithm), and adaptive retransmission timeouts (RTO).
#[derive(Debug, Clone)]
pub struct CongestionControl {
    pub mtu: u32,
    pub cwnd: f64,
    pub ssthresh: f64,
    pub srtt: f64,
    pub rttvar: f64,
    pub rto: f64,
    pub bytes_in_flight: usize,
    pub total_bytes_sent: u64,
    pub total_bytes_acked: u64,
    pub total_packets_lost: u64,
}

impl CongestionControl {
    pub const DEFAULT_MTU: u32 = 1464;
    pub const MIN_RTO: f64 = 100.0;  // 100 ms minimum RTO
    pub const MAX_RTO: f64 = 3000.0; // 3 seconds maximum RTO

    pub fn new(mtu: u32) -> Self {
        let mtu = mtu.max(576);
        let initial_cwnd = (mtu as f64) * 4.0;
        Self {
            mtu,
            cwnd: initial_cwnd,
            ssthresh: (mtu as f64) * 64.0,
            srtt: 100.0,
            rttvar: 50.0,
            rto: 300.0,
            bytes_in_flight: 0,
            total_bytes_sent: 0,
            total_bytes_acked: 0,
            total_packets_lost: 0,
        }
    }

    /// Whether the congestion window has sufficient budget to transmit `bytes`.
    #[inline]
    pub fn can_send(&self, bytes: usize) -> bool {
        (self.bytes_in_flight + bytes) as f64 <= self.cwnd
    }

    /// Records outgoing bytes sent over the wire.
    #[inline]
    pub fn on_packet_sent(&mut self, bytes: usize) {
        self.bytes_in_flight += bytes;
        self.total_bytes_sent += bytes as u64;
    }

    /// Handles an acknowledgment of `bytes_acked` with an optional RTT sample in ms.
    pub fn on_ack_received(&mut self, bytes_acked: usize, rtt_sample: Option<f64>) {
        if self.bytes_in_flight >= bytes_acked {
            self.bytes_in_flight -= bytes_acked;
        } else {
            self.bytes_in_flight = 0;
        }
        self.total_bytes_acked += bytes_acked as u64;

        // Window increase: Slow start vs Congestion Avoidance
        if self.cwnd < self.ssthresh {
            // Slow Start: exponential growth
            self.cwnd += bytes_acked as f64;
        } else {
            // Congestion Avoidance: linear growth (1 MTU per RTT)
            let increment = ((self.mtu * self.mtu) as f64) / self.cwnd;
            self.cwnd += increment;
        }

        // Update smoothed RTT and variance via RFC 6298 / Jacobson's algorithm
        if let Some(sample) = rtt_sample {
            if sample > 0.0 {
                let err = sample - self.srtt;
                self.srtt += 0.125 * err;
                self.rttvar += 0.25 * (err.abs() - self.rttvar);
                let new_rto = self.srtt + (4.0 * self.rttvar);
                self.rto = new_rto.clamp(Self::MIN_RTO, Self::MAX_RTO);
            }
        }
    }

    /// Handles a packet loss event (NACK or RTO timeout).
    pub fn on_packet_loss(&mut self, lost_bytes: usize) {
        if self.bytes_in_flight >= lost_bytes {
            self.bytes_in_flight -= lost_bytes;
        } else {
            self.bytes_in_flight = 0;
        }
        self.total_packets_lost += 1;

        // Multiplicative decrease
        self.ssthresh = (self.cwnd / 2.0).max((self.mtu as f64) * 2.0);
        self.cwnd = self.ssthresh;

        // Exponential backoff for RTO
        self.rto = (self.rto * 1.5).min(Self::MAX_RTO);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_congestion_control_slow_start_and_avoidance() {
        let mut cc = CongestionControl::new(1400);
        assert!(cc.can_send(1000));

        let initial_cwnd = cc.cwnd;
        cc.on_packet_sent(1400);
        assert_eq!(cc.bytes_in_flight, 1400);

        // ACK arrives: slow start expands window
        cc.on_ack_received(1400, Some(50.0));
        assert_eq!(cc.bytes_in_flight, 0);
        assert!(cc.cwnd > initial_cwnd);
        assert!(cc.srtt > 0.0);

        // Simulate loss
        let prev_cwnd = cc.cwnd;
        cc.on_packet_loss(1400);
        assert!(cc.cwnd < prev_cwnd);
        assert_eq!(cc.total_packets_lost, 1);
    }
}
