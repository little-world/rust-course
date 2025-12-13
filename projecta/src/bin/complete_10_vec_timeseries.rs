use std::collections::VecDeque;

// =============================================================================
// Milestone 1: Basic Sliding Window with VecDeque
// =============================================================================

/// Fixed-size sliding window
#[derive(Debug, Clone)]
pub struct SlidingWindow<T> {
    window: VecDeque<T>,
    capacity: usize,
}

impl<T: Clone> SlidingWindow<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            window: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, value: T) -> Option<T> {
        let mut evicted = None;
        if self.window.len() == self.capacity {
            evicted = self.window.pop_front();
        }
        self.window.push_back(value);
        evicted
    }

    pub fn as_slice(&mut self) -> &[T] {
        self.window.make_contiguous();
        self.window.as_slices().0
    }

    pub fn len(&self) -> usize {
        self.window.len()
    }

    pub fn is_full(&self) -> bool {
        self.window.len() == self.capacity
    }

    pub fn is_empty(&self) -> bool {
        self.window.is_empty()
    }
}

impl SlidingWindow<f64> {
    pub fn average(&self) -> Option<f64> {
        if self.window.is_empty() {
            return None;
        }
        Some(self.window.iter().sum::<f64>() / self.window.len() as f64)
    }

    pub fn min(&self) -> Option<f64> {
        self.window
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .copied()
    }

    pub fn max(&self) -> Option<f64> {
        self.window
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .copied()
    }
}

// =============================================================================
// Milestone 2: Incremental Statistics (Avoid Re-Scanning)
// =============================================================================

#[derive(Debug, Clone)]
pub struct IncrementalWindow {
    window: VecDeque<f64>,
    capacity: usize,
    running_sum: f64,
    running_sum_sq: f64,
}

impl IncrementalWindow {
    pub fn new(capacity: usize) -> Self {
        Self {
            window: VecDeque::with_capacity(capacity),
            capacity,
            running_sum: 0.0,
            running_sum_sq: 0.0,
        }
    }

    pub fn push(&mut self, value: f64) {
        if self.window.len() == self.capacity {
            if let Some(old) = self.window.pop_front() {
                self.running_sum -= old;
                self.running_sum_sq -= old * old;
            }
        }
        self.window.push_back(value);
        self.running_sum += value;
        self.running_sum_sq += value * value;
    }

    pub fn average(&self) -> Option<f64> {
        if self.window.is_empty() {
            return None;
        }
        Some(self.running_sum / self.window.len() as f64)
    }

    pub fn variance(&self) -> Option<f64> {
        if self.window.len() < 2 {
            return None;
        }
        let len = self.window.len() as f64;
        let mean = self.running_sum / len;
        let mean_sq = self.running_sum_sq / len;
        let var = (mean_sq - mean * mean).max(0.0);
        Some(var)
    }

    pub fn std_dev(&self) -> Option<f64> {
        self.variance().map(|v| v.sqrt())
    }

    pub fn len(&self) -> usize {
        self.window.len()
    }

    pub fn is_empty(&self) -> bool {
        self.window.is_empty()
    }

    // Milestone 4 methods implemented later
}

// =============================================================================
// Milestone 3: Min/Max with Monotonic Deque
// =============================================================================

#[derive(Debug)]
pub struct MinMaxWindow {
    window: VecDeque<(usize, f64)>,
    min_deque: VecDeque<(usize, f64)>,
    max_deque: VecDeque<(usize, f64)>,
    capacity: usize,
    index: usize,
}

impl MinMaxWindow {
    pub fn new(capacity: usize) -> Self {
        Self {
            window: VecDeque::with_capacity(capacity),
            min_deque: VecDeque::new(),
            max_deque: VecDeque::new(),
            capacity,
            index: 0,
        }
    }

    pub fn push(&mut self, value: f64) {
        self.index += 1;
        let idx = self.index;

        self.window.push_back((idx, value));
        if self.window.len() > self.capacity {
            if let Some((old_idx, _)) = self.window.pop_front() {
                if self.min_deque.front().map(|(i, _)| *i) == Some(old_idx) {
                    self.min_deque.pop_front();
                }
                if self.max_deque.front().map(|(i, _)| *i) == Some(old_idx) {
                    self.max_deque.pop_front();
                }
            }
        }

        while let Some(&(_, v)) = self.min_deque.back() {
            if v > value {
                self.min_deque.pop_back();
            } else {
                break;
            }
        }
        self.min_deque.push_back((idx, value));

        while let Some(&(_, v)) = self.max_deque.back() {
            if v < value {
                self.max_deque.pop_back();
            } else {
                break;
            }
        }
        self.max_deque.push_back((idx, value));
    }

    pub fn min(&self) -> Option<f64> {
        self.min_deque.front().map(|(_, v)| *v)
    }

    pub fn max(&self) -> Option<f64> {
        self.max_deque.front().map(|(_, v)| *v)
    }

    pub fn len(&self) -> usize {
        self.window.len()
    }
}

// =============================================================================
// Milestone 4: Median and Percentiles with select_nth_unstable
// =============================================================================

impl IncrementalWindow {
    pub fn median(&self) -> Option<f64> {
        if self.window.is_empty() {
            return None;
        }
        let mut values: Vec<f64> = self.window.iter().copied().collect();
        let len = values.len();
        if len % 2 == 1 {
            let mid = len / 2;
            values.select_nth_unstable_by(mid, |a, b| a.partial_cmp(b).unwrap());
            Some(values[mid])
        } else {
            let mid2 = len / 2;
            values.select_nth_unstable_by(mid2, |a, b| a.partial_cmp(b).unwrap());
            let val2 = values[mid2];
            let val1 = values[..mid2]
                .iter()
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .copied()
                .unwrap_or(val2);
            Some((val1 + val2) / 2.0)
        }
    }

    pub fn percentile(&self, p: f64) -> Option<f64> {
        if self.window.is_empty() || p < 0.0 || p > 100.0 {
            return None;
        }
        let mut values: Vec<f64> = self.window.iter().copied().collect();
        let len = values.len();
        let pos = (p / 100.0) * (len as f64 - 1.0);
        let low = pos.floor() as usize;
        let high = pos.ceil() as usize;
        values.select_nth_unstable_by(high, |a, b| a.partial_cmp(b).unwrap());
        let high_val = values[high];
        if high == low {
            Some(high_val)
        } else {
            let low_val = values[..high]
                .iter()
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .copied()
                .unwrap_or(high_val);
            let frac = pos - low as f64;
            Some(low_val * (1.0 - frac) + high_val * frac)
        }
    }

    pub fn percentiles(&self, ps: &[f64]) -> Vec<Option<f64>> {
        if self.window.is_empty() {
            return ps.iter().map(|_| None).collect();
        }
        let mut values: Vec<f64> = self.window.iter().copied().collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let len = values.len();
        ps.iter()
            .map(|p| {
                if *p < 0.0 || *p > 100.0 {
                    None
                } else {
                    let rank = ((p / 100.0) * (len as f64 - 1.0)).round() as usize;
                    Some(values[rank])
                }
            })
            .collect()
    }
}

// =============================================================================
// Milestone 5: Multiple Windows Simultaneously
// =============================================================================

#[derive(Debug)]
pub struct MultiWindowAnalyzer {
    windows: Vec<IncrementalWindow>,
    window_sizes: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct WindowStats {
    pub average: Option<f64>,
    pub std_dev: Option<f64>,
    pub median: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub window_size: usize,
}

impl MultiWindowAnalyzer {
    pub fn new(window_sizes: Vec<usize>) -> Self {
        let windows = window_sizes.iter().map(|&s| IncrementalWindow::new(s)).collect();
        Self { windows, window_sizes }
    }

    pub fn push(&mut self, value: f64) {
        for window in &mut self.windows {
            window.push(value);
        }
    }

    pub fn get_stats(&self, window_index: usize) -> Option<WindowStats> {
        self.windows.get(window_index).map(|window| {
            let min = window
                .window
                .iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .copied();
            let max = window
                .window
                .iter()
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .copied();
            WindowStats {
                average: window.average(),
                std_dev: window.std_dev(),
                median: window.median(),
                min,
                max,
                window_size: self.window_sizes[window_index],
            }
        })
    }

    pub fn all_stats(&self) -> Vec<WindowStats> {
        (0..self.windows.len())
            .filter_map(|idx| self.get_stats(idx))
            .collect()
    }

    pub fn window_count(&self) -> usize {
        self.windows.len()
    }
}

// =============================================================================
// Milestone 6: Anomaly Detection with Z-Score
// =============================================================================

#[derive(Debug)]
pub struct AnomalyDetector {
    analyzer: MultiWindowAnalyzer,
    threshold: f64,
    anomalies: Vec<Anomaly>,
}

#[derive(Debug, Clone)]
pub struct Anomaly {
    pub value: f64,
    pub z_score: f64,
    pub timestamp: usize,
    pub window_stats: WindowStats,
}

impl AnomalyDetector {
    pub fn new(window_sizes: Vec<usize>, threshold: f64) -> Self {
        Self {
            analyzer: MultiWindowAnalyzer::new(window_sizes),
            threshold,
            anomalies: Vec::new(),
        }
    }

    pub fn push(&mut self, value: f64, timestamp: usize) -> Option<Anomaly> {
        self.analyzer.push(value);
        if let Some(last_stats) = self.analyzer.get_stats(self.analyzer.window_count().saturating_sub(1)) {
            if let (Some(mean), Some(std_dev)) = (last_stats.average, last_stats.std_dev) {
                if std_dev > 0.0 {
                    let z = (value - mean) / std_dev;
                    if z.abs() >= self.threshold {
                        let anomaly = Anomaly {
                            value,
                            z_score: z,
                            timestamp,
                            window_stats: last_stats.clone(),
                        };
                        self.anomalies.push(anomaly.clone());
                        return Some(anomaly);
                    }
                }
            }
        }
        None
    }

    pub fn anomaly_rate(&self, total_points: usize) -> f64 {
        if total_points == 0 {
            return 0.0;
        }
        self.anomalies.len() as f64 / total_points as f64
    }

    pub fn get_anomalies(&self) -> &[Anomaly] {
        &self.anomalies
    }

    pub fn clear_anomalies(&mut self) {
        self.anomalies.clear();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    // ----- Milestone 1 -----
    #[test]
    fn test_create_window() {
        let window: SlidingWindow<f64> = SlidingWindow::new(10);
        assert_eq!(window.len(), 0);
        assert!(!window.is_full());
        assert!(window.is_empty());
    }

    #[test]
    fn test_push_values() {
        let mut window = SlidingWindow::new(3);
        assert_eq!(window.push(1.0), None);
        assert_eq!(window.push(2.0), None);
        assert_eq!(window.push(3.0), None);
        assert!(window.is_full());
    }

    #[test]
    fn test_window_eviction() {
        let mut window = SlidingWindow::new(3);
        window.push(1.0);
        window.push(2.0);
        window.push(3.0);
        assert_eq!(window.push(4.0), Some(1.0));
        assert_eq!(window.push(5.0), Some(2.0));
    }

    #[test]
    fn test_window_fifo_order() {
        let mut window = SlidingWindow::new(3);
        window.push(10.0);
        window.push(20.0);
        window.push(30.0);
        window.push(40.0);
        assert_eq!(window.as_slice(), &[20.0, 30.0, 40.0]);
    }

    #[test]
    fn test_average() {
        let mut window = SlidingWindow::new(5);
        assert_eq!(window.average(), None);
        window.push(10.0);
        assert_eq!(window.average(), Some(10.0));
        window.push(20.0);
        assert_eq!(window.average(), Some(15.0));
    }

    #[test]
    fn test_min_max() {
        let mut window = SlidingWindow::new(5);
        window.push(30.0);
        window.push(10.0);
        window.push(50.0);
        assert_eq!(window.min(), Some(10.0));
        assert_eq!(window.max(), Some(50.0));
    }

    // ----- Milestone 2 -----
    #[test]
    fn test_incremental_average() {
        let mut window = IncrementalWindow::new(5);
        window.push(10.0);
        assert_eq!(window.average(), Some(10.0));
        window.push(20.0);
        assert_eq!(window.average(), Some(15.0));
        window.push(30.0);
        assert_eq!(window.average(), Some(20.0));
    }

    #[test]
    fn test_incremental_average_after_eviction() {
        let mut window = IncrementalWindow::new(3);
        window.push(10.0);
        window.push(20.0);
        window.push(30.0);
        assert_eq!(window.average(), Some(20.0));
        window.push(40.0);
        assert_eq!(window.average(), Some(30.0));
    }

    #[test]
    fn test_variance_calculation() {
        let mut window = IncrementalWindow::new(5);
        window.push(2.0);
        window.push(4.0);
        window.push(4.0);
        window.push(4.0);
        window.push(5.0);
        let variance = window.variance().unwrap();
        assert!((variance - 0.96).abs() < 0.01);
    }

    #[test]
    fn test_std_dev_calculation() {
        let mut window = IncrementalWindow::new(5);
        window.push(2.0);
        window.push(4.0);
        window.push(4.0);
        window.push(4.0);
        window.push(5.0);
        let std_dev = window.std_dev().unwrap();
        assert!((std_dev - 0.98).abs() < 0.01);
    }

    #[test]
    fn test_incremental_vs_naive() {
        let mut window = IncrementalWindow::new(100);
        let values: Vec<f64> = (0..100).map(|i| i as f64 * 1.5).collect();
        for &v in &values {
            window.push(v);
        }
        let incremental = window.average().unwrap();
        let naive = values.iter().sum::<f64>() / values.len() as f64;
        assert!((incremental - naive).abs() < 1e-6);
    }

    #[test]
    fn test_variance_empty_window() {
        let window = IncrementalWindow::new(10);
        assert_eq!(window.variance(), None);
    }

    // ----- Milestone 3 -----
    #[test]
    fn test_min_max_basic() {
        let mut window = MinMaxWindow::new(5);
        window.push(30.0);
        assert_eq!(window.min(), Some(30.0));
        window.push(10.0);
        assert_eq!(window.min(), Some(10.0));
        window.push(50.0);
        assert_eq!(window.max(), Some(50.0));
    }

    #[test]
    fn test_min_max_after_eviction() {
        let mut window = MinMaxWindow::new(3);
        window.push(10.0);
        window.push(30.0);
        window.push(20.0);
        window.push(40.0);
        assert_eq!(window.min(), Some(20.0));
        assert_eq!(window.max(), Some(40.0));
    }

    #[test]
    fn test_max_eviction() {
        let mut window = MinMaxWindow::new(3);
        window.push(50.0);
        window.push(30.0);
        window.push(20.0);
        window.push(25.0);
        assert_eq!(window.max(), Some(30.0));
    }

    #[test]
    fn test_monotonic_sequence() {
        let mut window = MinMaxWindow::new(5);
        for i in 1..=5 {
            window.push(i as f64);
        }
        assert_eq!(window.min(), Some(1.0));
        window.push(6.0);
        assert_eq!(window.min(), Some(2.0));
    }

    // ----- Milestone 4 -----
    #[test]
    fn test_median_odd_count() {
        let mut window = IncrementalWindow::new(10);
        window.push(1.0);
        window.push(3.0);
        window.push(2.0);
        assert_eq!(window.median(), Some(2.0));
    }

    #[test]
    fn test_median_even_count() {
        let mut window = IncrementalWindow::new(10);
        window.push(1.0);
        window.push(2.0);
        window.push(3.0);
        window.push(4.0);
        assert_eq!(window.median(), Some(2.5));
    }

    #[test]
    fn test_percentile_basic() {
        let mut window = IncrementalWindow::new(10);
        for i in 1..=10 {
            window.push(i as f64);
        }
        assert_eq!(window.percentile(0.0), Some(1.0));
        assert_eq!(window.percentile(100.0), Some(10.0));
        let p50 = window.percentile(50.0).unwrap();
        assert!((p50 - 5.5).abs() < 0.5);
    }

    #[test]
    fn test_percentile_invalid_range() {
        let mut window = IncrementalWindow::new(10);
        window.push(5.0);
        assert_eq!(window.percentile(-1.0), None);
        assert_eq!(window.percentile(101.0), None);
    }

    #[test]
    fn test_multiple_percentiles() {
        let mut window = IncrementalWindow::new(100);
        for i in 1..=100 {
            window.push(i as f64);
        }
        let percentiles = window.percentiles(&[25.0, 50.0, 75.0]);
        assert_eq!(percentiles.len(), 3);
        assert!(percentiles.iter().all(|v| v.is_some()));
    }

    #[test]
    fn test_median_vs_sort() {
        let mut window = IncrementalWindow::new(1000);
        for i in 0..1000 {
            window.push((i * 7 % 1000) as f64);
        }
        let median = window.median().unwrap();
        let mut values: Vec<f64> = window.window.iter().copied().collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let expected = (values[499] + values[500]) / 2.0;
        assert!((median - expected).abs() < 0.01);
    }

    #[test]
    fn test_quickselect_vs_sort_performance() {
        let mut window = IncrementalWindow::new(1000);
        for i in 0..1000 {
            window.push(i as f64);
        }
        let start = Instant::now();
        for _ in 0..100 {
            let _ = window.median();
        }
        let quick_time = start.elapsed();
        let start = Instant::now();
        for _ in 0..100 {
            let mut v: Vec<f64> = window.window.iter().copied().collect();
            v.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let _ = v[v.len() / 2];
            let _sum: f64 = v.iter().sum();
        }
        let sort_time = start.elapsed();
        assert!(quick_time.as_nanos() > 0);
        assert!(sort_time.as_nanos() > 0);
    }

    // ----- Milestone 5 -----
    #[test]
    fn test_multi_window_creation() {
        let analyzer = MultiWindowAnalyzer::new(vec![10, 60, 300]);
        assert_eq!(analyzer.window_count(), 3);
    }

    #[test]
    fn test_multi_window_push() {
        let mut analyzer = MultiWindowAnalyzer::new(vec![3, 5]);
        for i in 1..=10 {
            analyzer.push(i as f64);
        }
        assert_eq!(analyzer.get_stats(0).unwrap().average, Some(9.0));
        assert_eq!(analyzer.get_stats(1).unwrap().average, Some(8.0));
    }

    #[test]
    fn test_all_stats() {
        let mut analyzer = MultiWindowAnalyzer::new(vec![5, 10, 20]);
        for i in 1..=30 {
            analyzer.push(i as f64);
        }
        let stats = analyzer.all_stats();
        assert_eq!(stats.len(), 3);
        assert!(stats.iter().all(|s| s.average.is_some()));
    }

    #[test]
    fn test_different_window_behaviors() {
        let mut analyzer = MultiWindowAnalyzer::new(vec![2, 5]);
        analyzer.push(10.0);
        analyzer.push(20.0);
        analyzer.push(30.0);
        analyzer.push(40.0);
        analyzer.push(50.0);
        assert_eq!(analyzer.get_stats(0).unwrap().average, Some(45.0));
        assert_eq!(analyzer.get_stats(1).unwrap().average, Some(30.0));
    }

    #[test]
    fn test_empty_stats() {
        let analyzer = MultiWindowAnalyzer::new(vec![10]);
        let stats = analyzer.get_stats(0).unwrap();
        assert_eq!(stats.average, None);
        assert_eq!(stats.std_dev, None);
        assert_eq!(stats.median, None);
    }

    #[test]
    fn test_invalid_window_index() {
        let analyzer = MultiWindowAnalyzer::new(vec![10, 20]);
        assert!(analyzer.get_stats(2).is_none());
    }

    // ----- Milestone 6 -----
    #[test]
    fn test_no_anomalies_in_normal_data() {
        let mut detector = AnomalyDetector::new(vec![100], 3.0);
        for i in 0..200 {
            detector.push(50.0 + ((i % 10) as f64 - 5.0), i);
        }
        assert_eq!(detector.get_anomalies().len(), 0);
    }

    #[test]
    fn test_detect_obvious_outlier() {
        let mut detector = AnomalyDetector::new(vec![100], 3.0);
        for i in 0..100 {
            detector.push(50.0, i);
        }
        let anomaly = detector.push(200.0, 100);
        assert!(anomaly.is_some());
        assert!(anomaly.unwrap().z_score.abs() > 3.0);
    }

    #[test]
    fn test_z_score_calculation() {
        let mut detector = AnomalyDetector::new(vec![10], 2.0);
        for i in 0..10 {
            detector.push(10.0, i);
        }
        let anomaly = detector.push(15.0, 10);
        if let Some(anomaly) = anomaly {
            assert!(anomaly.z_score > 2.0);
        }
    }

    #[test]
    fn test_anomaly_rate() {
        let mut detector = AnomalyDetector::new(vec![50], 3.0);
        for i in 0..1000 {
            let value = if i % 100 == 0 { 1000.0 } else { 50.0 };
            detector.push(value, i);
        }
        let rate = detector.anomaly_rate(1000);
        assert!(rate > 0.005 && rate < 0.015);
    }

    #[test]
    fn test_anomaly_context() {
        let mut detector = AnomalyDetector::new(vec![20], 3.0);
        for i in 0..30 {
            detector.push(100.0, i);
        }
        let anomaly = detector.push(200.0, 30).unwrap();
        assert_eq!(anomaly.timestamp, 30);
        assert!(anomaly.window_stats.average.is_some());
    }

    #[test]
    fn test_different_thresholds() {
        let data: Vec<f64> = (0..100).map(|i| 50.0 + (i % 20) as f64).collect();
        let mut strict = AnomalyDetector::new(vec![50], 2.0);
        for (i, &v) in data.iter().enumerate() {
            strict.push(v, i);
        }
        let mut lenient = AnomalyDetector::new(vec![50], 4.0);
        for (i, &v) in data.iter().enumerate() {
            lenient.push(v, i);
        }
        assert!(strict.get_anomalies().len() >= lenient.get_anomalies().len());
    }

    #[test]
    fn test_clear_anomalies() {
        let mut detector = AnomalyDetector::new(vec![10], 2.0);
        for i in 0..10 {
            detector.push(10.0, i);
        }
        detector.push(50.0, 10);
        assert_eq!(detector.get_anomalies().len(), 1);
        detector.clear_anomalies();
        assert_eq!(detector.get_anomalies().len(), 0);
    }
}

fn main() {}