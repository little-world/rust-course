//! Pattern 8: Const Generics
//! Example: Const Generic Protocol Frames
//!
//! Run with: cargo run --example p8_protocol_frames

// Network frame with const generic payload size
#[derive(Debug)]
struct Frame<const SIZE: usize> {
    header: [u8; 4],
    payload: [u8; SIZE],
    checksum: u32,
}

impl<const SIZE: usize> Frame<SIZE> {
    fn new(payload: [u8; SIZE]) -> Self {
        let checksum = Self::calculate_checksum(&payload);
        Frame {
            header: [0x01, 0x02, 0x03, 0x04], // Magic bytes
            payload,
            checksum,
        }
    }

    fn calculate_checksum(payload: &[u8; SIZE]) -> u32 {
        payload.iter().map(|&b| b as u32).sum()
    }

    fn verify(&self) -> bool {
        self.checksum == Self::calculate_checksum(&self.payload)
    }

    fn total_size(&self) -> usize {
        4 + SIZE + 4 // header + payload + checksum
    }

    fn payload_size(&self) -> usize {
        SIZE
    }
}

// Type aliases for common frame sizes
type SmallFrame = Frame<64>;
type StandardFrame = Frame<256>;
type LargeFrame = Frame<1024>;
type JumboFrame = Frame<9000>;

// Function that works with any frame size
fn print_frame_info<const N: usize>(frame: &Frame<N>) {
    println!("Frame info:");
    println!("  Header: {:?}", frame.header);
    println!("  Payload size: {} bytes", frame.payload_size());
    println!("  Total size: {} bytes", frame.total_size());
    println!("  Checksum: {}", frame.checksum);
    println!("  Valid: {}", frame.verify());
}

// Helper function to demonstrate const generics
fn array_to_vec<T: Copy, const N: usize>(arr: [T; N]) -> Vec<T> {
    arr.to_vec()
}

// Function requiring minimum array size (using runtime check)
fn first_two<T: Copy, const N: usize>(arr: [T; N]) -> Option<(T, T)> {
    if N >= 2 {
        Some((arr[0], arr[1]))
    } else {
        None
    }
}

fn main() {
    println!("=== Small Frame (64 bytes) ===");
    let small_payload = [0u8; 64];
    let small = SmallFrame::new(small_payload);
    print_frame_info(&small);

    println!("\n=== Standard Frame (256 bytes) ===");
    let mut std_payload = [0u8; 256];
    std_payload[0] = 0xFF;
    std_payload[255] = 0xFF;
    let standard = StandardFrame::new(std_payload);
    print_frame_info(&standard);

    println!("\n=== Large Frame (1024 bytes) ===");
    let large_payload = [0xAB; 1024];
    let large = LargeFrame::new(large_payload);
    print_frame_info(&large);

    println!("\n=== Frame Size Comparison ===");
    println!("SmallFrame total: {} bytes", std::mem::size_of::<SmallFrame>());
    println!("StandardFrame total: {} bytes", std::mem::size_of::<StandardFrame>());
    println!("LargeFrame total: {} bytes", std::mem::size_of::<LargeFrame>());
    println!("JumboFrame total: {} bytes", std::mem::size_of::<JumboFrame>());

    println!("\n=== Array to Vec Conversion ===");
    let arr = [1, 2, 3];
    let vec = array_to_vec(arr);
    println!("array_to_vec([1, 2, 3]) = {:?}", vec);

    let arr5 = [10, 20, 30, 40, 50];
    let vec5 = array_to_vec(arr5);
    println!("array_to_vec([10,20,30,40,50]) = {:?}", vec5);

    println!("\n=== First Two Elements ===");
    let arr = [1, 2, 3, 4, 5];
    let result = first_two(arr);
    println!("first_two([1,2,3,4,5]) = {:?}", result);

    let arr1 = [1];
    let result1 = first_two(arr1);
    println!("first_two([1]) = {:?}", result1);

    println!("\n=== Type Safety ===");
    println!("Frame<64> and Frame<256> are DIFFERENT types");
    println!("You cannot accidentally mix frame sizes");
    println!("All size calculations happen at compile time");

    // This would NOT compile:
    // let small: Frame<64> = Frame::new([0u8; 64]);
    // let standard: Frame<256> = small; // ERROR: mismatched types
}
