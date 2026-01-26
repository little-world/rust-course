//! Pattern 3: Chunking and Windowing
//! Example: Sliding Windows for Moving Averages
//!
//! Run with: cargo run --example p3_windows

fn main() {
    println!("=== Sliding Windows ===\n");

    // Basic windows
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
    println!("Data: {:?}", data);

    println!("\nWindows of size 3:");
    for (i, window) in data.windows(3).enumerate() {
        println!("  Window {}: {:?}", i, window);
    }

    // Moving average
    println!("\n=== Moving Average ===\n");

    fn moving_average(values: &[f64], window_size: usize) -> Vec<f64> {
        values.windows(window_size)
            .map(|window| {
                let sum: f64 = window.iter().sum();
                sum / window.len() as f64
            })
            .collect()
    }

    let prices = vec![100.0, 102.0, 101.0, 105.0, 103.0, 107.0, 106.0, 110.0];
    println!("Prices: {:?}", prices);

    let ma3 = moving_average(&prices, 3);
    println!("3-period MA: {:?}", ma3);

    let ma5 = moving_average(&prices, 5);
    println!("5-period MA: {:?}", ma5);

    // Pairwise operations (deltas)
    println!("\n=== Pairwise Operations (Deltas) ===\n");

    fn compute_deltas(values: &[i32]) -> Vec<i32> {
        values.windows(2)
            .map(|pair| pair[1] - pair[0])
            .collect()
    }

    let data = vec![1, 3, 6, 10, 15, 21];
    println!("Data: {:?}", data);

    let deltas = compute_deltas(&data);
    println!("Deltas: {:?}", deltas);

    // Growth rates
    fn compute_growth_rates(values: &[f64]) -> Vec<f64> {
        values.windows(2)
            .map(|pair| (pair[1] - pair[0]) / pair[0] * 100.0)
            .collect()
    }

    let prices = vec![100.0, 105.0, 103.0, 110.0, 108.0];
    println!("\nPrices: {:?}", prices);

    let rates = compute_growth_rates(&prices);
    println!("Growth rates (%): {:?}", rates.iter()
        .map(|r| format!("{:.1}", r))
        .collect::<Vec<_>>());

    // Pattern detection
    println!("\n=== Pattern Detection ===\n");

    fn find_consecutive_sequences(data: &[i32], target: i32) -> Vec<usize> {
        data.windows(3)
            .enumerate()
            .filter_map(|(i, window)| {
                if window.iter().all(|&x| x == target) {
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
    }

    let data = vec![1, 5, 5, 5, 2, 5, 5, 5, 5, 3];
    println!("Data: {:?}", data);

    let sequences = find_consecutive_sequences(&data, 5);
    println!("Sequences of three 5s start at indices: {:?}", sequences);

    // Detect local maxima
    fn find_local_maxima(values: &[f64]) -> Vec<usize> {
        values.windows(3)
            .enumerate()
            .filter_map(|(i, window)| {
                if window[1] > window[0] && window[1] > window[2] {
                    Some(i + 1) // Index of the middle element
                } else {
                    None
                }
            })
            .collect()
    }

    let signal = vec![1.0, 3.0, 2.0, 5.0, 4.0, 6.0, 3.0, 7.0, 2.0];
    println!("\nSignal: {:?}", signal);

    let maxima = find_local_maxima(&signal);
    println!("Local maxima at indices: {:?}", maxima);
    println!("Values: {:?}", maxima.iter().map(|&i| signal[i]).collect::<Vec<_>>());

    // Spectrogram pattern (overlapping windows)
    println!("\n=== Overlapping Windows (Spectrogram) ===\n");

    fn compute_spectrogram_indices(
        signal_len: usize,
        window_size: usize,
        hop_size: usize,
    ) -> Vec<(usize, usize)> {
        (0..signal_len)
            .step_by(hop_size)
            .filter_map(|i| {
                if i + window_size <= signal_len {
                    Some((i, i + window_size))
                } else {
                    None
                }
            })
            .collect()
    }

    let signal_len = 100;
    let window_size = 20;
    let hop_size = 10; // 50% overlap

    let windows = compute_spectrogram_indices(signal_len, window_size, hop_size);
    println!("Signal length: {}", signal_len);
    println!("Window size: {}, Hop size: {} ({}% overlap)",
        window_size, hop_size, 100 - (hop_size * 100 / window_size));
    println!("Number of windows: {}", windows.len());
    println!("First 5 windows: {:?}", &windows[..5.min(windows.len())]);

    // Sampling with step_by
    println!("\n=== Sampling Every Nth Element ===\n");

    fn sample_every_nth(data: &[f64], n: usize) -> Vec<f64> {
        data.iter()
            .step_by(n)
            .copied()
            .collect()
    }

    let data: Vec<f64> = (0..20).map(|i| i as f64).collect();
    println!("Data: {:?}", data);

    let sampled = sample_every_nth(&data, 4);
    println!("Every 4th element: {:?}", sampled);

    println!("\n=== Key Points ===");
    println!("1. windows(n) creates overlapping views (n-1 shared elements)");
    println!("2. Output length is len - window_size + 1");
    println!("3. Perfect for moving averages, derivatives, pattern detection");
    println!("4. Use step_by for custom hop sizes (spectrograms)");
    println!("5. No allocation - windows are views into original data");
}
