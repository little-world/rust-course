// Pattern 2: Common Trait Implementations
// Demonstrates Debug, Clone, PartialEq, Eq, Hash, Ord, Default, Display, Error, Iterator.

use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

// ============================================================================
// Example: The Derivable Core
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct User {
    id: u64,
    username: String,
    email: String,
}

// ============================================================================
// Example: Skipping Clone for large, move-only types
// ============================================================================

#[derive(Debug)]
struct LargeBuffer {
    data: Vec<u8>,
}

// ============================================================================
// Example: Custom PartialEq for case-insensitive comparison
// ============================================================================

#[derive(Debug, Clone)]
struct CaseInsensitiveString(String);

impl PartialEq for CaseInsensitiveString {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_lowercase() == other.0.to_lowercase()
    }
}

// ============================================================================
// Example: Ordering Traits - PartialOrd and Ord
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Priority {
    level: u8,
    timestamp: u64,
}

// Custom Ord for reverse ordering
#[derive(Debug, Clone, PartialEq, Eq)]
struct ReverseScore(u32);

impl Ord for ReverseScore {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.cmp(&self.0)
    }
}

impl PartialOrd for ReverseScore {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// ============================================================================
// Example: PartialOrd without Ord (for floats)
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
struct FloatWrapper(f64);

impl PartialOrd for FloatWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

// ============================================================================
// Example: Default for zero-cost initialization
// ============================================================================

#[derive(Debug, Default)]
struct Config {
    timeout_ms: u64,
    retries: u32,
    verbose: bool,
}

// Custom Default for better defaults
#[derive(Debug)]
struct Connection {
    host: String,
    port: u16,
    timeout_ms: u64,
}

impl Default for Connection {
    fn default() -> Self {
        Connection {
            host: "localhost".to_string(),
            port: 8080,
            timeout_ms: 5000,
        }
    }
}

fn get_config(maybe_config: Option<Config>) -> Config {
    maybe_config.unwrap_or_default()
}

// ============================================================================
// Example: Display and Debug
// ============================================================================

#[derive(Debug)]
struct Timestamp {
    unix_seconds: i64,
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Timestamp({})", self.unix_seconds)
    }
}

// ============================================================================
// Example: Error trait
// ============================================================================

#[derive(Debug)]
enum ApiError {
    NetworkFailure(String),
    InvalidResponse,
    Unauthorized,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ApiError::NetworkFailure(msg) => write!(f, "Network error: {}", msg),
            ApiError::InvalidResponse => write!(f, "Invalid response from server"),
            ApiError::Unauthorized => write!(f, "Authentication required"),
        }
    }
}

impl Error for ApiError {}

fn fetch_data() -> Result<String, ApiError> {
    Err(ApiError::NetworkFailure("Connection timeout".to_string()))
}

fn process() -> Result<(), Box<dyn Error>> {
    let _data = fetch_data()?;
    Ok(())
}

// ============================================================================
// Example: Using thiserror for ergonomic error types
// ============================================================================

use thiserror::Error;

#[derive(Error, Debug)]
enum DataError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error at line {line}: {msg}")]
    Parse { line: usize, msg: String },

    #[error("Invalid format")]
    InvalidFormat,
}

// ============================================================================
// Example: Iterator trait
// ============================================================================

struct CountDown {
    count: u32,
}

impl Iterator for CountDown {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count > 0 {
            let current = self.count;
            self.count -= 1;
            Some(current)
        } else {
            None
        }
    }
}

// ============================================================================
// Example: IntoIterator for ergonomic iteration
// ============================================================================

struct Playlist {
    songs: Vec<String>,
}

impl IntoIterator for Playlist {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.songs.into_iter()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_derive() {
        let user1 = User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
        };
        let user2 = user1.clone();

        assert_eq!(user1, user2);
        assert_eq!(format!("{:?}", user1), format!("{:?}", user2));
    }

    #[test]
    fn test_case_insensitive_eq() {
        let s1 = CaseInsensitiveString("Hello".to_string());
        let s2 = CaseInsensitiveString("HELLO".to_string());
        let s3 = CaseInsensitiveString("hello".to_string());

        assert_eq!(s1, s2);
        assert_eq!(s2, s3);
        assert_eq!(s1, s3);
    }

    #[test]
    fn test_priority_ordering() {
        let p1 = Priority {
            level: 1,
            timestamp: 100,
        };
        let p2 = Priority {
            level: 2,
            timestamp: 50,
        };
        let p3 = Priority {
            level: 1,
            timestamp: 200,
        };

        // Higher level = higher priority
        assert!(p2 > p1);
        // Same level, higher timestamp = higher
        assert!(p3 > p1);

        // BTreeSet maintains sorted order
        let mut tasks = BTreeSet::new();
        tasks.insert(p2.clone());
        tasks.insert(p1.clone());
        tasks.insert(p3.clone());

        let sorted: Vec<_> = tasks.into_iter().collect();
        assert_eq!(sorted[0], p1);
        assert_eq!(sorted[1], p3);
        assert_eq!(sorted[2], p2);
    }

    #[test]
    fn test_reverse_score() {
        let mut scores = vec![
            ReverseScore(10),
            ReverseScore(50),
            ReverseScore(30),
        ];
        scores.sort();

        // Should be in descending order due to reverse Ord
        assert_eq!(scores[0].0, 50);
        assert_eq!(scores[1].0, 30);
        assert_eq!(scores[2].0, 10);
    }

    #[test]
    fn test_float_wrapper_partial_ord() {
        let f1 = FloatWrapper(1.0);
        let f2 = FloatWrapper(2.0);
        let f_nan = FloatWrapper(f64::NAN);

        assert!(f1 < f2);
        // NaN comparisons return None
        assert_eq!(f_nan.partial_cmp(&f1), None);
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.timeout_ms, 0);
        assert_eq!(config.retries, 0);
        assert!(!config.verbose);
    }

    #[test]
    fn test_custom_default_connection() {
        let conn = Connection::default();
        assert_eq!(conn.host, "localhost");
        assert_eq!(conn.port, 8080);
        assert_eq!(conn.timeout_ms, 5000);
    }

    #[test]
    fn test_get_config_with_default() {
        let config = get_config(None);
        assert_eq!(config.timeout_ms, 0);

        let custom = Config {
            timeout_ms: 1000,
            ..Default::default()
        };
        let config = get_config(Some(custom));
        assert_eq!(config.timeout_ms, 1000);
    }

    #[test]
    fn test_display_vs_debug() {
        let ts = Timestamp {
            unix_seconds: 1609459200,
        };
        assert_eq!(format!("{}", ts), "Timestamp(1609459200)");
        assert_eq!(
            format!("{:?}", ts),
            "Timestamp { unix_seconds: 1609459200 }"
        );
    }

    #[test]
    fn test_api_error_display() {
        let err = ApiError::NetworkFailure("timeout".to_string());
        assert_eq!(format!("{}", err), "Network error: timeout");

        let err = ApiError::InvalidResponse;
        assert_eq!(format!("{}", err), "Invalid response from server");
    }

    #[test]
    fn test_thiserror_data_error() {
        let err = DataError::Parse {
            line: 42,
            msg: "unexpected token".to_string(),
        };
        assert_eq!(format!("{}", err), "Parse error at line 42: unexpected token");

        let err = DataError::InvalidFormat;
        assert_eq!(format!("{}", err), "Invalid format");
    }

    #[test]
    fn test_countdown_iterator() {
        let countdown = CountDown { count: 5 };
        let nums: Vec<u32> = countdown.collect();
        assert_eq!(nums, vec![5, 4, 3, 2, 1]);
    }

    #[test]
    fn test_countdown_combinators() {
        let countdown = CountDown { count: 5 };
        let sum: u32 = countdown.filter(|&n| n % 2 == 0).sum();
        assert_eq!(sum, 6); // 4 + 2
    }

    #[test]
    fn test_playlist_into_iterator() {
        let playlist = Playlist {
            songs: vec!["Song 1".to_string(), "Song 2".to_string()],
        };

        let mut count = 0;
        for _song in playlist {
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_error_propagation() {
        let result = process();
        assert!(result.is_err());
    }
}

fn main() {
    println!("Pattern 2: Common Trait Implementations");
    println!("=======================================\n");

    // Derivable core
    println!("Derivable Core Traits:");
    let user = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    println!("  Debug: {:?}", user);
    println!("  Clone: {:?}", user.clone());

    // Case insensitive comparison
    println!("\nCustom PartialEq (case-insensitive):");
    let s1 = CaseInsensitiveString("Hello".to_string());
    let s2 = CaseInsensitiveString("HELLO".to_string());
    println!("  {:?} == {:?}: {}", s1, s2, s1 == s2);

    // Ordering
    println!("\nOrdering with Ord:");
    let mut scores = vec![ReverseScore(10), ReverseScore(50), ReverseScore(30)];
    println!("  Before sort: {:?}", scores);
    scores.sort();
    println!("  After sort (reverse): {:?}", scores);

    // Default
    println!("\nDefault trait:");
    println!("  Config::default(): {:?}", Config::default());
    println!("  Connection::default(): {:?}", Connection::default());

    let conn = Connection {
        host: "api.example.com".to_string(),
        ..Default::default()
    };
    println!("  Custom host with defaults: {:?}", conn);

    // Display vs Debug
    println!("\nDisplay vs Debug:");
    let ts = Timestamp {
        unix_seconds: 1609459200,
    };
    println!("  Display: {}", ts);
    println!("  Debug: {:?}", ts);

    // Error types
    println!("\nError types:");
    let err = ApiError::NetworkFailure("connection refused".to_string());
    println!("  ApiError: {}", err);

    let err = DataError::Parse {
        line: 10,
        msg: "missing semicolon".to_string(),
    };
    println!("  DataError (thiserror): {}", err);

    // Iterator
    println!("\nCustom Iterator:");
    let countdown = CountDown { count: 5 };
    print!("  CountDown: ");
    for n in countdown {
        print!("{} ", n);
    }
    println!();

    // IntoIterator
    println!("\nIntoIterator for custom types:");
    let playlist = Playlist {
        songs: vec!["Track 1".to_string(), "Track 2".to_string(), "Track 3".to_string()],
    };
    for song in playlist {
        println!("  Playing: {}", song);
    }
}
