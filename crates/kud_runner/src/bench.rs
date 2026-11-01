//! Frame-time statistics for the benchmark harness.
//!
//! Percentiles are reported instead of a mean because a mean hides the hitches
//! that actually break a frame budget.

use serde_json::{json, Value};

/// Summary of per-frame timing samples, in microseconds.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TimingStats {
    /// Number of samples.
    pub count: usize,
    /// Fastest frame.
    pub min: u64,
    /// Median frame.
    pub p50: u64,
    /// 95th percentile frame.
    pub p95: u64,
    /// 99th percentile frame.
    pub p99: u64,
    /// Slowest frame.
    pub max: u64,
}

impl TimingStats {
    /// Sorts samples in place and computes percentiles by nearest rank.
    pub fn from_samples(samples: &mut [u64]) -> Self {
        if samples.is_empty() {
            return Self::default();
        }
        samples.sort_unstable();
        Self {
            count: samples.len(),
            min: samples[0],
            p50: percentile(samples, 50.0),
            p95: percentile(samples, 95.0),
            p99: percentile(samples, 99.0),
            max: samples[samples.len() - 1],
        }
    }

    /// Returns the stats as a JSON object for machine-readable reports.
    pub fn as_json(self) -> Value {
        json!({
            "count": self.count,
            "min": self.min,
            "p50": self.p50,
            "p95": self.p95,
            "p99": self.p99,
            "max": self.max,
        })
    }
}

/// Nearest-rank percentile over an already sorted slice.
fn percentile(sorted: &[u64], percent: f64) -> u64 {
    debug_assert!(!sorted.is_empty());
    let rank = (percent / 100.0 * sorted.len() as f64).ceil() as usize;
    sorted[rank.saturating_sub(1).min(sorted.len() - 1)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_samples_produce_zeroed_stats() {
        assert_eq!(TimingStats::from_samples(&mut []), TimingStats::default());
    }

    #[test]
    fn single_sample_is_every_percentile() {
        let stats = TimingStats::from_samples(&mut [42]);
        assert_eq!(stats.count, 1);
        assert_eq!(stats.min, 42);
        assert_eq!(stats.p50, 42);
        assert_eq!(stats.p99, 42);
        assert_eq!(stats.max, 42);
    }

    #[test]
    fn percentiles_use_nearest_rank() {
        let mut samples: Vec<u64> = (1..=100).collect();
        let stats = TimingStats::from_samples(&mut samples);
        assert_eq!(stats.p50, 50);
        assert_eq!(stats.p95, 95);
        assert_eq!(stats.p99, 99);
        assert_eq!(stats.max, 100);
    }

    #[test]
    fn unsorted_input_is_sorted_before_summary() {
        let stats = TimingStats::from_samples(&mut [9, 1, 5, 3, 7]);
        assert_eq!(stats.min, 1);
        assert_eq!(stats.max, 9);
        assert_eq!(stats.p50, 5);
    }

    #[test]
    fn percentile_never_indexes_out_of_bounds() {
        let mut samples = [5_u64, 6];
        let stats = TimingStats::from_samples(&mut samples);
        assert_eq!(stats.p99, 6);
    }
}
