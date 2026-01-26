// Pattern 1: Layered HAL Drivers
// Demonstrates HAL trait abstractions for portable embedded drivers.
// This file uses mock implementations to test on desktop.

use std::cell::RefCell;
use std::convert::Infallible;

// ============================================================================
// Mock HAL Traits (simplified embedded-hal style)
// ============================================================================

/// OutputPin trait for controlling digital outputs
pub trait OutputPin {
    type Error;
    fn set_low(&mut self) -> Result<(), Self::Error>;
    fn set_high(&mut self) -> Result<(), Self::Error>;
}

/// CountDown timer trait
pub trait CountDown {
    type Time;
    fn start<T>(&mut self, count: T) where T: Into<Self::Time>;
    fn wait(&mut self) -> nb::Result<(), void::Void>;
}

/// SPI Transfer trait
pub trait Transfer<W> {
    type Error;
    fn transfer<'a>(&mut self, words: &'a mut [W]) -> Result<&'a [W], Self::Error>;
}

/// Delay trait
pub trait DelayUs<T> {
    fn delay_us(&mut self, us: T);
}

// nb crate simulation for non-blocking operations
pub mod nb {
    pub type Result<T, E> = core::result::Result<T, Error<E>>;

    pub enum Error<E> {
        WouldBlock,
        Other(E),
    }

    #[macro_export]
    macro_rules! block {
        ($e:expr) => {
            loop {
                match $e {
                    Ok(x) => break Ok(x),
                    Err($crate::nb::Error::WouldBlock) => continue,
                    Err($crate::nb::Error::Other(e)) => break Err(e),
                }
            }
        };
    }
}

pub mod void {
    #[derive(Debug)]
    pub enum Void {}
}

// ============================================================================
// Example: Mock LED Implementation
// ============================================================================

pub struct MockLed {
    state: RefCell<bool>,
    name: &'static str,
}

impl MockLed {
    pub fn new(name: &'static str) -> Self {
        Self {
            state: RefCell::new(false),
            name,
        }
    }

    pub fn is_on(&self) -> bool {
        *self.state.borrow()
    }
}

impl OutputPin for MockLed {
    type Error = Infallible;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        *self.state.borrow_mut() = false;
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        *self.state.borrow_mut() = true;
        Ok(())
    }
}

// ============================================================================
// Example: Mock Timer Implementation
// ============================================================================

pub struct MockTimer {
    ticks_remaining: u32,
    period: u32,
}

impl MockTimer {
    pub fn new() -> Self {
        Self {
            ticks_remaining: 0,
            period: 0,
        }
    }

    /// Simulate time passing
    pub fn tick(&mut self) {
        if self.ticks_remaining > 0 {
            self.ticks_remaining -= 1;
        }
    }
}

impl CountDown for MockTimer {
    type Time = u32;

    fn start<T>(&mut self, count: T) where T: Into<Self::Time> {
        self.period = count.into();
        self.ticks_remaining = self.period;
    }

    fn wait(&mut self) -> nb::Result<(), void::Void> {
        if self.ticks_remaining == 0 {
            self.ticks_remaining = self.period;
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

// ============================================================================
// Example: Heartbeat Driver (Generic over HAL traits)
// ============================================================================

pub struct Heartbeat<P, T> {
    pub led: P,
    pub timer: T,
    cycles: u32,
    state: HeartbeatState,
}

#[derive(Clone, Copy, PartialEq)]
enum HeartbeatState {
    LedOff,
    LedOnWaiting,
    LedOffWaiting,
}

impl<P, T> Heartbeat<P, T>
where
    P: OutputPin<Error = Infallible>,
    T: CountDown,
{
    pub fn new(led: P, timer: T) -> Self {
        Self {
            led,
            timer,
            cycles: 0,
            state: HeartbeatState::LedOff,
        }
    }

    /// Poll-based step function (non-blocking, for testing)
    pub fn poll(&mut self) -> bool {
        match self.state {
            HeartbeatState::LedOff => {
                self.led.set_high().ok();
                self.state = HeartbeatState::LedOnWaiting;
                false
            }
            HeartbeatState::LedOnWaiting => {
                if self.timer.wait().is_ok() {
                    self.led.set_low().ok();
                    self.state = HeartbeatState::LedOffWaiting;
                }
                false
            }
            HeartbeatState::LedOffWaiting => {
                if self.timer.wait().is_ok() {
                    self.cycles += 1;
                    self.state = HeartbeatState::LedOff;
                    true // Cycle completed
                } else {
                    false
                }
            }
        }
    }

    pub fn cycles(&self) -> u32 {
        self.cycles
    }
}

// ============================================================================
// Example: Mock SPI Implementation
// ============================================================================

pub struct MockSpi {
    response_data: Vec<u8>,
    last_transfer: Vec<u8>,
}

impl MockSpi {
    pub fn new(response: &[u8]) -> Self {
        Self {
            response_data: response.to_vec(),
            last_transfer: Vec::new(),
        }
    }

    pub fn last_transfer(&self) -> &[u8] {
        &self.last_transfer
    }
}

impl Transfer<u8> for MockSpi {
    type Error = Infallible;

    fn transfer<'a>(&mut self, words: &'a mut [u8]) -> Result<&'a [u8], Self::Error> {
        self.last_transfer = words.to_vec();
        for (i, byte) in words.iter_mut().enumerate() {
            if i < self.response_data.len() {
                *byte = self.response_data[i];
            }
        }
        Ok(words)
    }
}

// ============================================================================
// Example: Mock Delay Implementation
// ============================================================================

pub struct MockDelay {
    total_us: u64,
}

impl MockDelay {
    pub fn new() -> Self {
        Self { total_us: 0 }
    }

    pub fn total_delay_us(&self) -> u64 {
        self.total_us
    }
}

impl DelayUs<u16> for MockDelay {
    fn delay_us(&mut self, us: u16) {
        self.total_us += us as u64;
    }
}

// ============================================================================
// Example: IMU Driver (Generic over SPI, GPIO, Delay)
// ============================================================================

pub struct ImuDriver<SPI, CS, DELAY> {
    spi: SPI,
    cs: CS,
    delay: DELAY,
}

impl<SPI, CS, DELAY> ImuDriver<SPI, CS, DELAY>
where
    SPI: Transfer<u8, Error = Infallible>,
    CS: OutputPin<Error = Infallible>,
    DELAY: DelayUs<u16>,
{
    pub fn new(spi: SPI, cs: CS, delay: DELAY) -> Self {
        Self { spi, cs, delay }
    }

    pub fn read_whoami(&mut self) -> Result<u8, Infallible> {
        let mut buf = [0x75 | 0x80, 0]; // Read bit set
        self.cs.set_low().ok();
        self.spi.transfer(&mut buf)?;
        self.cs.set_high().ok();
        Ok(buf[1])
    }

    pub fn configure(&mut self) -> Result<(), Infallible> {
        self.write_reg(0x6B, 0x00)?; // Wake up
        self.delay.delay_us(50);
        self.write_reg(0x1C, 0x10)?; // Configure accelerometer
        Ok(())
    }

    fn write_reg(&mut self, reg: u8, value: u8) -> Result<(), Infallible> {
        let mut buf = [reg & 0x7F, value];
        self.cs.set_low().ok();
        self.spi.transfer(&mut buf)?;
        self.cs.set_high().ok();
        Ok(())
    }
}

// ============================================================================
// Example: Board Support Package Pattern
// ============================================================================

pub struct Board<LED, TIMER> {
    pub led: LED,
    pub timer: TIMER,
}

impl Board<MockLed, MockTimer> {
    pub fn init_mock() -> Self {
        Board {
            led: MockLed::new("LED1"),
            timer: MockTimer::new(),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_led() {
        let mut led = MockLed::new("test");
        assert!(!led.is_on());

        led.set_high().unwrap();
        assert!(led.is_on());

        led.set_low().unwrap();
        assert!(!led.is_on());
    }

    #[test]
    fn test_mock_timer() {
        let mut timer = MockTimer::new();
        timer.start(3u32);

        // Should block until ticks complete
        assert!(matches!(timer.wait(), Err(nb::Error::WouldBlock)));
        timer.tick();
        assert!(matches!(timer.wait(), Err(nb::Error::WouldBlock)));
        timer.tick();
        assert!(matches!(timer.wait(), Err(nb::Error::WouldBlock)));
        timer.tick();
        assert!(timer.wait().is_ok());
    }

    #[test]
    fn test_heartbeat_driver() {
        let led = MockLed::new("heartbeat");
        let mut timer = MockTimer::new();
        timer.start(2u32);

        let mut heartbeat = Heartbeat::new(led, timer);
        assert_eq!(heartbeat.cycles(), 0);

        // Poll-based testing: LED off -> LED on
        heartbeat.poll();
        assert!(heartbeat.led.is_on());

        // Wait for first timer (LED on period)
        heartbeat.timer.tick();
        heartbeat.timer.tick();
        heartbeat.poll();
        assert!(!heartbeat.led.is_on());

        // Wait for second timer (LED off period)
        heartbeat.timer.tick();
        heartbeat.timer.tick();
        let completed = heartbeat.poll();

        assert!(completed);
        assert_eq!(heartbeat.cycles(), 1);
    }

    #[test]
    fn test_mock_spi() {
        let mut spi = MockSpi::new(&[0x00, 0x71]); // WHO_AM_I response
        let mut buf = [0x75, 0x00];

        spi.transfer(&mut buf).unwrap();

        assert_eq!(buf[1], 0x71);
        assert_eq!(spi.last_transfer(), &[0x75, 0x00]);
    }

    #[test]
    fn test_imu_driver() {
        let spi = MockSpi::new(&[0x00, 0x71]);
        let cs = MockLed::new("CS");
        let delay = MockDelay::new();

        let mut imu = ImuDriver::new(spi, cs, delay);

        let whoami = imu.read_whoami().unwrap();
        assert_eq!(whoami, 0x71);
    }

    #[test]
    fn test_imu_configure() {
        let spi = MockSpi::new(&[0x00, 0x00]);
        let cs = MockLed::new("CS");
        let delay = MockDelay::new();

        let mut imu = ImuDriver::new(spi, cs, delay);
        imu.configure().unwrap();

        assert_eq!(imu.delay.total_delay_us(), 50);
    }

    #[test]
    fn test_board_init() {
        let board = Board::init_mock();
        assert!(!board.led.is_on());
    }
}

fn main() {
    println!("Pattern 1: Layered HAL Drivers");
    println!("==============================\n");

    // Initialize mock board
    let mut board = Board::init_mock();
    board.timer.start(2u32);

    println!("Mock board initialized");
    println!("  LED state: {}", if board.led.is_on() { "ON" } else { "OFF" });

    // Create heartbeat driver
    let mut heartbeat = Heartbeat::new(board.led, board.timer);

    // Simulate a few cycles using poll-based approach
    for i in 0..3 {
        // Start cycle (LED on)
        heartbeat.poll();

        // Wait for LED on period
        heartbeat.timer.tick();
        heartbeat.timer.tick();
        heartbeat.poll();

        // Wait for LED off period
        heartbeat.timer.tick();
        heartbeat.timer.tick();
        heartbeat.poll();

        println!("  Heartbeat cycle {} completed", i + 1);
    }

    // Demonstrate IMU driver
    println!("\nIMU Driver Demo:");
    let spi = MockSpi::new(&[0x00, 0x71]);
    let cs = MockLed::new("IMU_CS");
    let delay = MockDelay::new();

    let mut imu = ImuDriver::new(spi, cs, delay);
    let whoami = imu.read_whoami().unwrap();
    println!("  WHO_AM_I register: 0x{:02X}", whoami);

    imu.configure().unwrap();
    println!("  IMU configured (delay: {}us)", imu.delay.total_delay_us());

    println!("\nHAL traits enable:");
    println!("  - Testing on desktop with mocks");
    println!("  - Same driver code works on STM32, Nordic, Raspberry Pi");
    println!("  - Swap implementations without changing business logic");
}
