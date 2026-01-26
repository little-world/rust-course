// Pattern 2: Structural Patterns - Adapter, Decorator, Facade, Newtype
// Demonstrates patterns for organizing relationships between entities.

use std::fmt;

// ============================================================================
// Example: Adapter Pattern with Trait Objects
// ============================================================================

// Target interface your code expects
trait MediaPlayer {
    fn play(&self, filename: &str);
}

// Existing third-party library with different interface
struct VlcPlayer;
impl VlcPlayer {
    fn play_vlc(&self, file_path: &str) {
        println!("Playing VLC: {}", file_path);
    }
}

struct Mp3Player;
impl Mp3Player {
    fn play_mp3(&self, file_name: &str) {
        println!("Playing MP3: {}", file_name);
    }
}

// Adapters to make them compatible
struct VlcAdapter {
    player: VlcPlayer,
}

impl MediaPlayer for VlcAdapter {
    fn play(&self, filename: &str) {
        self.player.play_vlc(filename);
    }
}

struct Mp3Adapter {
    player: Mp3Player,
}

impl MediaPlayer for Mp3Adapter {
    fn play(&self, filename: &str) {
        self.player.play_mp3(filename);
    }
}

fn play_media(player: &dyn MediaPlayer, file: &str) {
    player.play(file);
}

fn adapter_trait_object_example() {
    let vlc = VlcAdapter { player: VlcPlayer };
    let mp3 = Mp3Adapter { player: Mp3Player };
    play_media(&vlc, "video.mp4");
    play_media(&mp3, "song.mp3");
}

// ============================================================================
// Example: Zero-cost Adapter with Generics
// ============================================================================

trait PlayVlc {
    fn play_vlc(&self, path: &str);
}

impl PlayVlc for VlcPlayer {
    fn play_vlc(&self, path: &str) {
        println!("Generic VLC playing: {}", path);
    }
}

struct GenericAdapter<T> {
    inner: T,
}

impl<T: PlayVlc> MediaPlayer for GenericAdapter<T> {
    fn play(&self, filename: &str) {
        self.inner.play_vlc(filename);
    }
}

fn adapter_generic_example() {
    // No trait object overhead, monomorphized at compile-time
    let adapter = GenericAdapter { inner: VlcPlayer };
    adapter.play("video.mkv");
}

// ============================================================================
// Example: Decorator Pattern with Trait Objects
// ============================================================================

trait DataSource {
    fn read(&self) -> String;
    fn write(&mut self, data: &str);
}

// Concrete component
struct FileDataSource {
    #[allow(dead_code)]
    filename: String,
    contents: String,
}

impl DataSource for FileDataSource {
    fn read(&self) -> String {
        self.contents.clone()
    }

    fn write(&mut self, data: &str) {
        self.contents = data.to_string();
    }
}

// Decorator: Encryption
struct EncryptionDecorator {
    wrapped: Box<dyn DataSource>,
}

impl DataSource for EncryptionDecorator {
    fn read(&self) -> String {
        let encrypted = self.wrapped.read();
        decrypt(&encrypted)
    }

    fn write(&mut self, data: &str) {
        let encrypted = encrypt(data);
        self.wrapped.write(&encrypted);
    }
}

// Decorator: Compression
struct CompressionDecorator {
    wrapped: Box<dyn DataSource>,
}

impl DataSource for CompressionDecorator {
    fn read(&self) -> String {
        let compressed = self.wrapped.read();
        decompress(&compressed)
    }

    fn write(&mut self, data: &str) {
        let compressed = compress(data);
        self.wrapped.write(&compressed);
    }
}

fn encrypt(data: &str) -> String {
    format!("encrypted({})", data)
}
fn decrypt(data: &str) -> String {
    data.strip_prefix("encrypted(")
        .and_then(|s| s.strip_suffix(')'))
        .unwrap_or(data)
        .to_string()
}
fn compress(data: &str) -> String {
    format!("compressed({})", data)
}
fn decompress(data: &str) -> String {
    data.strip_prefix("compressed(")
        .and_then(|s| s.strip_suffix(')'))
        .unwrap_or(data)
        .to_string()
}

fn decorator_trait_object_example() {
    let file = FileDataSource {
        filename: "data.txt".to_string(),
        contents: "sensitive data".to_string(),
    };

    let mut source: Box<dyn DataSource> = Box::new(file);
    source = Box::new(EncryptionDecorator { wrapped: source });
    source = Box::new(CompressionDecorator { wrapped: source });

    source.write("secret");
    println!("Decorated write: compressed(encrypted(secret))");
    println!("Decorated read: {}", source.read());
}

// ============================================================================
// Example: Type-safe Decorator with Generics
// ============================================================================

struct Encrypted<T>(T);
struct Compressed<T>(T);

trait Read {
    fn read(&self) -> String;
}

impl Read for String {
    fn read(&self) -> String {
        self.clone()
    }
}

impl<T: Read> Read for Encrypted<T> {
    fn read(&self) -> String {
        decrypt(&self.0.read())
    }
}

impl<T: Read> Read for Compressed<T> {
    fn read(&self) -> String {
        decompress(&self.0.read())
    }
}

fn decorator_generic_example() {
    // Compile-time composition, zero-cost
    let data = String::from("compressed(encrypted(data))");
    let secure = Compressed(Encrypted(data));
    println!("Generic decorator read: {}", secure.read());
}

// ============================================================================
// Example: Facade Pattern
// ============================================================================

mod video_processing {
    pub struct VideoDecoder;
    impl VideoDecoder {
        pub fn decode(&self, _file: &str) -> Vec<u8> {
            println!("  Decoding video...");
            vec![1, 2, 3]
        }
    }

    pub struct AudioExtractor;
    impl AudioExtractor {
        pub fn extract(&self, _data: &[u8]) -> Vec<u8> {
            println!("  Extracting audio...");
            vec![4, 5, 6]
        }
    }

    pub struct CodecManager;
    impl CodecManager {
        pub fn configure(&self) {
            println!("  Configuring codecs...");
        }
    }

    pub struct FormatConverter;
    impl FormatConverter {
        pub fn convert(&self, _data: &[u8], _format: &str) -> String {
            println!("  Converting format...");
            "output.mp4".to_string()
        }
    }
}

// Facade: Simple interface
struct VideoConverter {
    decoder: video_processing::VideoDecoder,
    audio: video_processing::AudioExtractor,
    codec: video_processing::CodecManager,
    converter: video_processing::FormatConverter,
}

impl VideoConverter {
    fn new() -> Self {
        Self {
            decoder: video_processing::VideoDecoder,
            audio: video_processing::AudioExtractor,
            codec: video_processing::CodecManager,
            converter: video_processing::FormatConverter,
        }
    }

    // Simple API for common use case
    fn convert_to_mp4(&self, filename: &str) -> String {
        self.codec.configure();
        let video_data = self.decoder.decode(filename);
        let audio_data = self.audio.extract(&video_data);
        self.converter.convert(&audio_data, "mp4")
    }
}

fn facade_example() {
    // Client code: Simple and clean
    let converter = VideoConverter::new();
    let output = converter.convert_to_mp4("input.avi");
    println!("Converted: {}", output);
}

// ============================================================================
// Example: Newtype Pattern - Type Safety
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
struct Meters(f64);

#[derive(Debug, Clone, Copy, PartialEq)]
struct Feet(f64);

impl Meters {
    fn new(value: f64) -> Self {
        Meters(value)
    }

    fn to_feet(self) -> Feet {
        Feet(self.0 * 3.28084)
    }

    fn value(&self) -> f64 {
        self.0
    }
}

impl Feet {
    fn new(value: f64) -> Self {
        Feet(value)
    }

    fn to_meters(self) -> Meters {
        Meters(self.0 / 3.28084)
    }

    #[allow(dead_code)]
    fn value(&self) -> f64 {
        self.0
    }
}

fn calculate_distance_safe(m: Meters) -> Meters {
    Meters(m.0 * 2.0)
}

fn newtype_safety_example() {
    let feet = Feet::new(10.0);
    // let result = calculate_distance_safe(feet);  // Compile error!
    let meters = feet.to_meters();
    let result = calculate_distance_safe(meters);
    println!("Newtype safety: {:?} -> doubled: {:?}", meters, result);
}

// ============================================================================
// Example: Newtype for Implementing External Traits (Orphan Rule)
// ============================================================================

// Can't implement Display for Vec<i32> directly (orphan rule)
// impl fmt::Display for Vec<i32> { }  // Error!

// Newtype enables custom implementation
struct IntList(Vec<i32>);

impl fmt::Display for IntList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        for (i, item) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", item)?;
        }
        write!(f, "]")
    }
}

fn newtype_orphan_rule_example() {
    let list = IntList(vec![1, 2, 3]);
    println!("IntList with Display: {}", list);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_vlc() {
        let adapter = VlcAdapter { player: VlcPlayer };
        // Just verify it doesn't panic
        adapter.play("test.mp4");
    }

    #[test]
    fn test_adapter_mp3() {
        let adapter = Mp3Adapter { player: Mp3Player };
        adapter.play("test.mp3");
    }

    #[test]
    fn test_generic_adapter() {
        let adapter = GenericAdapter { inner: VlcPlayer };
        adapter.play("test.mkv");
    }

    #[test]
    fn test_decorator_encryption() {
        let file = FileDataSource {
            filename: "test.txt".to_string(),
            contents: String::new(),
        };

        let mut source = EncryptionDecorator {
            wrapped: Box::new(file),
        };

        source.write("secret");
        assert_eq!(source.read(), "secret");
    }

    #[test]
    fn test_decorator_composition() {
        let file = FileDataSource {
            filename: "test.txt".to_string(),
            contents: String::new(),
        };

        let source: Box<dyn DataSource> = Box::new(file);
        let source = EncryptionDecorator { wrapped: source };
        let mut source = CompressionDecorator {
            wrapped: Box::new(source),
        };

        source.write("data");
        assert_eq!(source.read(), "data");
    }

    #[test]
    fn test_generic_decorator() {
        // Order matters: Encrypted reads first (decrypts), then Compressed (decompresses)
        // So the data must be encrypted(compressed(hello))
        let data = String::from("encrypted(compressed(hello))");
        let secure = Compressed(Encrypted(data));
        assert_eq!(secure.read(), "hello");
    }

    #[test]
    fn test_facade() {
        let converter = VideoConverter::new();
        let output = converter.convert_to_mp4("test.avi");
        assert_eq!(output, "output.mp4");
    }

    #[test]
    fn test_newtype_conversion() {
        let meters = Meters::new(1.0);
        let feet = meters.to_feet();
        let back = feet.to_meters();

        // Approximate equality due to floating point
        assert!((back.value() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_newtype_safety() {
        let meters = Meters::new(5.0);
        let result = calculate_distance_safe(meters);
        assert_eq!(result, Meters(10.0));
    }

    #[test]
    fn test_intlist_display() {
        let list = IntList(vec![1, 2, 3]);
        assert_eq!(format!("{}", list), "[1, 2, 3]");
    }

    #[test]
    fn test_intlist_empty() {
        let list = IntList(vec![]);
        assert_eq!(format!("{}", list), "[]");
    }
}

fn main() {
    println!("Pattern 2: Structural Patterns");
    println!("===============================\n");

    println!("=== Adapter Pattern (Trait Objects) ===");
    adapter_trait_object_example();
    println!();

    println!("=== Adapter Pattern (Generics) ===");
    adapter_generic_example();
    println!();

    println!("=== Decorator Pattern (Trait Objects) ===");
    decorator_trait_object_example();
    println!();

    println!("=== Decorator Pattern (Generics) ===");
    decorator_generic_example();
    println!();

    println!("=== Facade Pattern ===");
    facade_example();
    println!();

    println!("=== Newtype Pattern (Type Safety) ===");
    newtype_safety_example();
    println!();

    println!("=== Newtype Pattern (Orphan Rule) ===");
    newtype_orphan_rule_example();
}
