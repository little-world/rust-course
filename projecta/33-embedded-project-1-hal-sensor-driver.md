# HAL-Based Sensor Driver System

### Problem Statement

Build a portable sensor driver system that works across multiple embedded platforms (STM32, Raspberry Pi, or any hardware supporting `embedded-hal` traits). Your system should abstract hardware-specific details through HAL traits, support multiple sensor types (temperature, accelerometer, pressure), and provide a unified API for reading sensor data.

Your driver system should support:
- Hardware abstraction using `embedded-hal` traits
- Multiple communication protocols (I2C, SPI)
- Sensor discovery and initialization
- Data reading with error handling
- Mock implementations for host testing
- Configuration management per sensor type

## Why Hardware Abstraction Matters

### The Portability Problem

**The Problem**: Traditional embedded code is tightly coupled to specific hardware registers and vendor SDKs. Porting to a different microcontroller family means rewriting large portions of your driver code, even when the high-level logic is identical.

**Real-world example**:
```rust
// STM32-specific: Direct register access
unsafe {
    (*I2C1::ptr()).cr1.modify(|_, w| w.start().set_bit());
    while (*I2C1::ptr()).sr1.read().sb().bit_is_clear() {}
    // ... 50 lines of register manipulation
}

// Nordic nRF-specific: Different API entirely
let mut twi = Twim::new(dp.TWIM0, pins, twi::Frequency::K400);
twi.enable();
twi.write(addr, &[reg])?;
// Completely different approach!
```

Both do the same thing (I2C communication), but require separate implementations.

### HAL Traits: Write Once, Run Anywhere

**The Solution**: The `embedded-hal` traits provide a common interface that works across all platforms:

```rust
// Works on STM32, nRF, ESP32, Raspberry Pi, or ANY platform!
pub fn read_sensor<I2C>(i2c: &mut I2C, addr: u8, reg: u8) -> Result<u8, I2C::Error>
where
    I2C: embedded_hal::i2c::I2c,
{
    let mut buf = [0u8; 1];
    i2c.write_read(addr, &[reg], &mut buf)?;
    Ok(buf[0])
}
```

### Why It Matters

**Development Velocity**:
```
Traditional approach:
├─ Write driver for STM32    →  3 days
├─ Port to nRF52              →  2 days (rewrite)
├─ Port to ESP32              →  2 days (rewrite)
└─ Total: 7 days

HAL-based approach:
├─ Write HAL-agnostic driver  →  3 days
├─ Port to nRF52              →  30 minutes (configure BSP)
├─ Port to ESP32              →  30 minutes (configure BSP)
└─ Total: 4 days (43% faster!)
```

**Testing on Host**: HAL traits can be mocked, so you can unit test sensor logic on your development machine:
```rust
// Test on Linux/Mac/Windows without hardware!
#[test]
fn test_sensor_reads_temperature() {
    let mut mock_i2c = MockI2c::new();
    mock_i2c.expect_write_read(/* ... */);

    let mut sensor = TempSensor::new(mock_i2c);
    assert_eq!(sensor.read_celsius()?, 23.5);
}
```

**Production Benefits**:
- **Supplier flexibility**: Switch MCU vendors without driver rewrites
- **Prototyping speed**: Develop on Raspberry Pi, deploy to microcontroller
- **Maintenance**: Fix bugs once, benefits all platforms
- **Team collaboration**: Different engineers can work on different platforms simultaneously

## Use Cases

### 1. IoT Sensor Networks
- **Multi-vendor deployment**: Same sensor code runs on STM32 gateways and ESP32 nodes
- **Field upgrades**: Swap hardware without software changes
- **Rapid prototyping**: Test algorithms on Pi before PCB arrives

### 2. Industrial Monitoring Systems
- **Legacy migration**: Gradually replace old platforms while keeping application logic
- **Redundant systems**: Different MCU families with identical firmware
- **Compliance**: Single codebase simplifies certification (IEC 61508, ISO 26262)

### 3. Product Family Development
- **Cost optimization**: Support premium (high-end MCU) and budget (low-end MCU) variants
- **Feature scaling**: Same driver stack for basic and advanced models
- **Time-to-market**: Launch product before custom PCB ready using off-the-shelf dev boards

### 4. Research and Education
- **Platform-independent experiments**: Code transfers between lab equipment
- **Teaching**: Students learn portable practices, not vendor-specific quirks
- **Open source**: Drivers can be shared across communities

---

## Building the Project

### Milestone 1: HAL Trait Foundation

**Goal**: Define the core traits and data structures that abstract sensor operations, independent of any specific hardware platform.

**Why we start here**: Before writing drivers, we need a contract (trait) that defines what a sensor can do. This milestone teaches trait-based abstraction—the foundation of portable embedded code.

#### Architecture

**Traits:**
- `SensorDriver` - Core trait all sensors implement
  - **Method**: `fn init(&mut self) -> Result<(), SensorError>` - Initialize sensor hardware
  - **Method**: `fn read_raw(&mut self) -> Result<RawData, SensorError>` - Read raw sensor data
  - **Method**: `fn sensor_id(&self) -> &str` - Get sensor identifier

**Structs:**
- `SensorError` - Error types for sensor operations
  - **Variant**: `CommunicationError` - I2C/SPI communication failed
  - **Variant**: `InitializationError` - Sensor initialization failed
  - **Variant**: `DataError` - Invalid data received

- `RawData` - Raw sensor readings
  - **Field**: `values: [i16; 3]` - Raw sensor values (e.g., x/y/z for accel)
  - **Field**: `timestamp_ms: u64` - When data was captured

**Functions:**
- `impl Display for SensorError` - Human-readable error messages

**Starter Code**:

```rust
use core::fmt;

/// Errors that can occur during sensor operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorError {
    CommunicationError,
    InitializationError,
    DataError,
}

impl fmt::Display for SensorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Implement human-readable error messages
        todo!("Implement Display for SensorError")
    }
}

/// Raw sensor data reading
#[derive(Debug, Clone, Copy)]
pub struct RawData {
    pub values: [i16; 3],
    pub timestamp_ms: u64,
}

/// Core trait that all sensor drivers must implement
pub trait SensorDriver {
    /// Initialize the sensor hardware
    fn init(&mut self) -> Result<(), SensorError>;

    /// Read raw data from the sensor
    fn read_raw(&mut self) -> Result<RawData, SensorError>;

    /// Get a unique identifier for this sensor
    fn sensor_id(&self) -> &str;
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Mock sensor for testing trait implementation
    struct MockSensor {
        initialized: bool,
        id: String,
    }

    impl MockSensor {
        fn new(id: &str) -> Self {
            Self {
                initialized: false,
                id: id.to_string(),
            }
        }
    }

    impl SensorDriver for MockSensor {
        fn init(&mut self) -> Result<(), SensorError> {
            self.initialized = true;
            Ok(())
        }

        fn read_raw(&mut self) -> Result<RawData, SensorError> {
            if !self.initialized {
                return Err(SensorError::InitializationError);
            }
            Ok(RawData {
                values: [100, 200, 300],
                timestamp_ms: 1000,
            })
        }

        fn sensor_id(&self) -> &str {
            &self.id
        }
    }

    #[test]
    fn test_sensor_trait_basic() {
        let mut sensor = MockSensor::new("test-sensor");
        assert_eq!(sensor.sensor_id(), "test-sensor");

        // Should succeed after init
        sensor.init().unwrap();
        let data = sensor.read_raw().unwrap();
        assert_eq!(data.values, [100, 200, 300]);
    }

    #[test]
    fn test_read_before_init_fails() {
        let mut sensor = MockSensor::new("test");
        // Reading before init should fail
        assert_eq!(sensor.read_raw().unwrap_err(), SensorError::InitializationError);
    }

    #[test]
    fn test_error_display() {
        let err = SensorError::CommunicationError;
        let msg = format!("{}", err);
        assert!(!msg.is_empty());
    }
}
```

**Check Your Understanding**:
- Why use a trait instead of a concrete struct?
- What's the benefit of separating `init()` from `read_raw()`?
- Why does `RawData` use `i16` instead of `f32` for values?

---

#### Why Milestone 1 Isn't Enough

**Limitation**: We have a trait contract, but no actual hardware communication. A sensor driver needs to talk to physical devices via I2C or SPI buses.

**What we're adding**: Integration with `embedded-hal` I2C traits to enable real hardware communication while maintaining portability.

**Improvement**:
- **Capability**: Can now communicate with real I2C sensors
- **Portability**: Works on any platform with `embedded-hal` support
- **Testability**: Can mock the I2C bus for unit testing

---

### Milestone 2: I2C Temperature Sensor Driver

**Goal**: Implement a concrete sensor driver for an I2C temperature sensor (e.g., TMP102) that uses `embedded-hal` traits for communication.

**Why this milestone**: Moving from abstract traits to concrete implementation teaches how to wrap hardware protocols with portable abstractions.

#### Architecture

**Structs:**
- `TempSensor<I2C>` - Temperature sensor driver
  - **Field**: `i2c: I2C` - I2C bus handle (generic over embedded-hal trait)
  - **Field**: `address: u8` - I2C device address
  - **Field**: `initialized: bool` - Initialization state

**Functions:**
- `new(i2c: I2C, address: u8) -> Self` - Create sensor driver
- `read_celsius(&mut self) -> Result<f32, SensorError>` - Read temperature in Celsius
- `read_fahrenheit(&mut self) -> Result<f32, SensorError>` - Read temperature in Fahrenheit
- `write_register(&mut self, reg: u8, value: u8) -> Result<(), SensorError>` - Write to sensor register
- `read_register(&mut self, reg: u8) -> Result<u8, SensorError>` - Read from sensor register

**Constants:**
- `TEMP_REGISTER: u8 = 0x00` - Temperature data register
- `CONFIG_REGISTER: u8 = 0x01` - Configuration register
- `DEFAULT_ADDRESS: u8 = 0x48` - Default I2C address

**Starter Code**:

```rust
use embedded_hal::i2c::I2c;

const TEMP_REGISTER: u8 = 0x00;
const CONFIG_REGISTER: u8 = 0x01;
pub const DEFAULT_ADDRESS: u8 = 0x48;

pub struct TempSensor<I2C> {
    i2c: I2C,
    address: u8,
    initialized: bool,
}

impl<I2C> TempSensor<I2C>
where
    I2C: I2c,
{
    pub fn new(i2c: I2C, address: u8) -> Self {
        // TODO: Initialize struct fields
        todo!("Implement TempSensor::new")
    }

    /// Read temperature in Celsius
    pub fn read_celsius(&mut self) -> Result<f32, SensorError> {
        // TODO: Read 16-bit temperature register
        // TODO: Convert raw value to Celsius (TMP102 uses 12-bit with 0.0625°C resolution)
        // Format: [MSB][LSB] where temp = (value >> 4) * 0.0625
        todo!("Implement read_celsius")
    }

    /// Read temperature in Fahrenheit
    pub fn read_fahrenheit(&mut self) -> Result<f32, SensorError> {
        // TODO: Read Celsius and convert: F = C * 1.8 + 32
        todo!("Implement read_fahrenheit")
    }

    /// Write to a sensor register
    fn write_register(&mut self, reg: u8, value: u8) -> Result<(), SensorError> {
        // TODO: Use I2C write to send [register, value]
        todo!("Implement write_register")
    }

    /// Read from a sensor register
    fn read_register(&mut self, reg: u8) -> Result<u8, SensorError> {
        // TODO: Use I2C write_read to send register address and read response
        todo!("Implement read_register")
    }
}

impl<I2C> SensorDriver for TempSensor<I2C>
where
    I2C: I2c,
{
    fn init(&mut self) -> Result<(), SensorError> {
        // TODO: Write configuration register to set 12-bit resolution
        // TODO: Verify we can read from device (check who-am-i or temp register)
        // TODO: Set initialized flag
        todo!("Implement init")
    }

    fn read_raw(&mut self) -> Result<RawData, SensorError> {
        // TODO: Read temperature as raw i16 value
        // TODO: Store in RawData (use values[0] for temp, others can be 0)
        todo!("Implement read_raw")
    }

    fn sensor_id(&self) -> &str {
        "TMP102"
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction};

    #[test]
    fn test_temp_sensor_init() {
        let expectations = vec![
            Transaction::write(DEFAULT_ADDRESS, vec![CONFIG_REGISTER, 0x60]),
            Transaction::write_read(DEFAULT_ADDRESS, vec![TEMP_REGISTER], vec![0x19, 0x00]),
        ];
        let i2c = I2cMock::new(&expectations);

        let mut sensor = TempSensor::new(i2c, DEFAULT_ADDRESS);
        sensor.init().unwrap();
    }

    #[test]
    fn test_read_celsius() {
        let expectations = vec![
            // Init sequence
            Transaction::write(DEFAULT_ADDRESS, vec![CONFIG_REGISTER, 0x60]),
            Transaction::write_read(DEFAULT_ADDRESS, vec![TEMP_REGISTER], vec![0x19, 0x00]),
            // Read temperature: 0x1900 >> 4 = 0x190 = 400 decimal
            // 400 * 0.0625 = 25°C
            Transaction::write_read(DEFAULT_ADDRESS, vec![TEMP_REGISTER], vec![0x19, 0x00]),
        ];
        let i2c = I2cMock::new(&expectations);

        let mut sensor = TempSensor::new(i2c, DEFAULT_ADDRESS);
        sensor.init().unwrap();

        let temp = sensor.read_celsius().unwrap();
        assert!((temp - 25.0).abs() < 0.1);
    }

    #[test]
    fn test_read_fahrenheit() {
        let expectations = vec![
            Transaction::write(DEFAULT_ADDRESS, vec![CONFIG_REGISTER, 0x60]),
            Transaction::write_read(DEFAULT_ADDRESS, vec![TEMP_REGISTER], vec![0x19, 0x00]),
            Transaction::write_read(DEFAULT_ADDRESS, vec![TEMP_REGISTER], vec![0x19, 0x00]),
        ];
        let i2c = I2cMock::new(&expectations);

        let mut sensor = TempSensor::new(i2c, DEFAULT_ADDRESS);
        sensor.init().unwrap();

        let temp = sensor.read_fahrenheit().unwrap();
        // 25°C = 77°F
        assert!((temp - 77.0).abs() < 0.5);
    }

    #[test]
    fn test_sensor_trait_implementation() {
        let expectations = vec![
            Transaction::write(DEFAULT_ADDRESS, vec![CONFIG_REGISTER, 0x60]),
            Transaction::write_read(DEFAULT_ADDRESS, vec![TEMP_REGISTER], vec![0x19, 0x00]),
            Transaction::write_read(DEFAULT_ADDRESS, vec![TEMP_REGISTER], vec![0x19, 0x00]),
        ];
        let i2c = I2cMock::new(&expectations);

        let mut sensor: Box<dyn SensorDriver> = Box::new(TempSensor::new(i2c, DEFAULT_ADDRESS));
        sensor.init().unwrap();

        let data = sensor.read_raw().unwrap();
        assert_eq!(sensor.sensor_id(), "TMP102");
    }
}
```

**Check Your Understanding**:
- Why is `TempSensor` generic over `I2C` instead of taking `embedded_hal::i2c::I2c` directly?
- How does the mock I2C enable testing without hardware?
- Why separate `read_celsius()` from the trait's `read_raw()`?

---

#### Why Milestone 2 Isn't Enough

**Limitation**: We only support one sensor type (temperature) and one protocol (I2C). Real systems need multiple sensor types and SPI support.

**What we're adding**: An SPI-based accelerometer driver to demonstrate protocol flexibility and multi-sensor systems.

**Improvement**:
- **Protocol diversity**: Support both I2C and SPI sensors
- **Complexity**: Handle multi-axis data (accelerometer has 3 axes)
- **Architecture**: Pattern for managing heterogeneous sensor types

---

### Milestone 3: SPI Accelerometer Driver

**Goal**: Add an SPI-based 3-axis accelerometer driver (e.g., ADXL345) to demonstrate protocol flexibility and multi-channel data.

**Why this milestone**: Different sensors use different protocols. SPI requires chip select management and different data formats, teaching protocol-independent abstraction.

#### Architecture

**Structs:**
- `AccelSensor<SPI, CS>` - Accelerometer driver
  - **Field**: `spi: SPI` - SPI bus handle
  - **Field**: `cs: CS` - Chip select pin (generic over OutputPin trait)
  - **Field**: `scale: AccelScale` - Measurement range (±2g, ±4g, ±8g, ±16g)
  - **Field**: `initialized: bool` - Initialization state

- `AccelScale` - Measurement range configuration
  - **Variant**: `Range2G` - ±2g range, high resolution
  - **Variant**: `Range4G` - ±4g range
  - **Variant**: `Range8G` - ±8g range
  - **Variant**: `Range16G` - ±16g range, low resolution

- `AccelData` - Processed acceleration data
  - **Field**: `x: f32` - X-axis in g
  - **Field**: `y: f32` - Y-axis in g
  - **Field**: `z: f32` - Z-axis in g

**Functions:**
- `new(spi: SPI, cs: CS, scale: AccelScale) -> Self` - Create driver
- `read_accel(&mut self) -> Result<AccelData, SensorError>` - Read acceleration
- `read_xyz_raw(&mut self) -> Result<[i16; 3], SensorError>` - Read raw 16-bit values
- `set_scale(&mut self, scale: AccelScale) -> Result<(), SensorError>` - Change measurement range
- `spi_read(&mut self, reg: u8) -> Result<u8, SensorError>` - Read single register
- `spi_write(&mut self, reg: u8, value: u8) -> Result<(), SensorError>` - Write single register

**Constants:**
- `DEVID_REGISTER: u8 = 0x00` - Device ID register (should read 0xE5)
- `DATA_X0: u8 = 0x32` - X-axis data register (LSB)
- `POWER_CTL: u8 = 0x2D` - Power control register
- `DATA_FORMAT: u8 = 0x31` - Data format/scale register

**Starter Code**:

```rust
use embedded_hal::spi::SpiDevice;
use embedded_hal::digital::OutputPin;

const DEVID_REGISTER: u8 = 0x00;
const POWER_CTL: u8 = 0x2D;
const DATA_FORMAT: u8 = 0x31;
const DATA_X0: u8 = 0x32;

#[derive(Debug, Clone, Copy)]
pub enum AccelScale {
    Range2G = 0,
    Range4G = 1,
    Range8G = 2,
    Range16G = 3,
}

impl AccelScale {
    fn sensitivity(&self) -> f32 {
        // TODO: Return mg/LSB for each scale
        // 2g: 3.9 mg/LSB, 4g: 7.8 mg/LSB, 8g: 15.6 mg/LSB, 16g: 31.2 mg/LSB
        todo!("Implement sensitivity conversion")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AccelData {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub struct AccelSensor<SPI, CS> {
    spi: SPI,
    cs: CS,
    scale: AccelScale,
    initialized: bool,
}

impl<SPI, CS> AccelSensor<SPI, CS>
where
    SPI: SpiDevice,
    CS: OutputPin,
{
    pub fn new(spi: SPI, cs: CS, scale: AccelScale) -> Self {
        // TODO: Initialize struct
        todo!("Implement AccelSensor::new")
    }

    pub fn read_accel(&mut self) -> Result<AccelData, SensorError> {
        // TODO: Read raw XYZ values
        // TODO: Convert to g using scale sensitivity
        // TODO: Return AccelData
        todo!("Implement read_accel")
    }

    pub fn read_xyz_raw(&mut self) -> Result<[i16; 3], SensorError> {
        // TODO: Read 6 bytes starting from DATA_X0
        // TODO: Combine LSB/MSB pairs into i16 values
        // Format: [X_LSB, X_MSB, Y_LSB, Y_MSB, Z_LSB, Z_MSB]
        todo!("Implement read_xyz_raw")
    }

    pub fn set_scale(&mut self, scale: AccelScale) -> Result<(), SensorError> {
        // TODO: Write DATA_FORMAT register with new scale
        // TODO: Update internal scale field
        todo!("Implement set_scale")
    }

    fn spi_read(&mut self, reg: u8) -> Result<u8, SensorError> {
        // TODO: Set CS low
        // TODO: SPI transfer with read bit set (reg | 0x80)
        // TODO: Set CS high
        // TODO: Return read value
        todo!("Implement spi_read")
    }

    fn spi_write(&mut self, reg: u8, value: u8) -> Result<(), SensorError> {
        // TODO: Set CS low
        // TODO: SPI transfer with write bit (reg & 0x7F)
        // TODO: Set CS high
        todo!("Implement spi_write")
    }
}

impl<SPI, CS> SensorDriver for AccelSensor<SPI, CS>
where
    SPI: SpiDevice,
    CS: OutputPin,
{
    fn init(&mut self) -> Result<(), SensorError> {
        // TODO: Check device ID (should be 0xE5)
        // TODO: Write POWER_CTL to enable measurement mode (0x08)
        // TODO: Write DATA_FORMAT with scale
        // TODO: Set initialized flag
        todo!("Implement init")
    }

    fn read_raw(&mut self) -> Result<RawData, SensorError> {
        // TODO: Read XYZ as raw i16 values
        // TODO: Package into RawData
        todo!("Implement read_raw")
    }

    fn sensor_id(&self) -> &str {
        "ADXL345"
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal_mock::spi::{Mock as SpiMock, Transaction as SpiTransaction};
    use embedded_hal_mock::pin::{Mock as PinMock, State, Transaction as PinTransaction};

    #[test]
    fn test_accel_init() {
        let spi_expectations = vec![
            // Read device ID
            SpiTransaction::transfer(vec![0x80], vec![0x00, 0xE5]),
            // Write power control
            SpiTransaction::write(vec![POWER_CTL, 0x08]),
            // Write data format
            SpiTransaction::write(vec![DATA_FORMAT, 0x00]),
        ];

        let cs_expectations = vec![
            PinTransaction::set(State::Low),
            PinTransaction::set(State::High),
            PinTransaction::set(State::Low),
            PinTransaction::set(State::High),
            PinTransaction::set(State::Low),
            PinTransaction::set(State::High),
        ];

        let spi = SpiMock::new(&spi_expectations);
        let cs = PinMock::new(&cs_expectations);

        let mut sensor = AccelSensor::new(spi, cs, AccelScale::Range2G);
        sensor.init().unwrap();
    }

    #[test]
    fn test_read_accel() {
        let spi_expectations = vec![
            // Init
            SpiTransaction::transfer(vec![0x80], vec![0x00, 0xE5]),
            SpiTransaction::write(vec![POWER_CTL, 0x08]),
            SpiTransaction::write(vec![DATA_FORMAT, 0x00]),
            // Read 6 bytes: X=256 (1g), Y=0, Z=0
            SpiTransaction::transfer(
                vec![0xB2, 0, 0, 0, 0, 0, 0],
                vec![0, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00],
            ),
        ];

        let cs_expectations = vec![
            PinTransaction::set(State::Low),
            PinTransaction::set(State::High),
            PinTransaction::set(State::Low),
            PinTransaction::set(State::High),
            PinTransaction::set(State::Low),
            PinTransaction::set(State::High),
            PinTransaction::set(State::Low),
            PinTransaction::set(State::High),
        ];

        let spi = SpiMock::new(&spi_expectations);
        let cs = PinMock::new(&cs_expectations);

        let mut sensor = AccelSensor::new(spi, cs, AccelScale::Range2G);
        sensor.init().unwrap();

        let accel = sensor.read_accel().unwrap();
        assert!((accel.x - 1.0).abs() < 0.1); // ~1g on X axis
    }

    #[test]
    fn test_scale_conversion() {
        assert!((AccelScale::Range2G.sensitivity() - 3.9).abs() < 0.1);
        assert!((AccelScale::Range4G.sensitivity() - 7.8).abs() < 0.1);
    }
}
```

**Check Your Understanding**:
- Why does SPI need a chip select pin while I2C doesn't?
- How does the scale affect measurement range vs. resolution?
- Why keep both `read_accel()` and `read_raw()` methods?

---

#### Why Milestone 3 Isn't Enough

**Limitation**: Each sensor is used independently. Real applications need to manage multiple sensors simultaneously, polling them in sequence and aggregating data.

**What we're adding**: A sensor manager that handles multiple heterogeneous sensors (I2C temp + SPI accel) through trait objects.

**Improvement**:
- **Architecture**: Dynamic dispatch via trait objects
- **Flexibility**: Add/remove sensors at runtime
- **Scalability**: Manage 10+ sensors without code duplication
- **API unification**: Single interface for all sensor types

---

### Milestone 4: Multi-Sensor Manager

**Goal**: Create a sensor manager that coordinates multiple sensors of different types, providing unified polling, error handling, and data aggregation.

**Why this milestone**: Real systems have many sensors. This milestone teaches dynamic dispatch, trait objects, and system-level architecture.

#### Architecture

**Structs:**
- `SensorManager` - Manages collection of sensors
  - **Field**: `sensors: Vec<Box<dyn SensorDriver>>` - Heterogeneous sensor collection
  - **Field**: `poll_interval_ms: u64` - How often to poll sensors

- `SensorReading` - Single sensor reading with metadata
  - **Field**: `sensor_id: String` - Which sensor
  - **Field**: `data: RawData` - Sensor data
  - **Field**: `timestamp: u64` - When captured

- `SystemSnapshot` - Complete system state
  - **Field**: `readings: Vec<SensorReading>` - All sensor readings
  - **Field**: `errors: Vec<(String, SensorError)>` - Failed sensors

**Functions:**
- `new(poll_interval_ms: u64) -> Self` - Create manager
- `add_sensor(&mut self, sensor: Box<dyn SensorDriver>)` - Register sensor
- `init_all(&mut self) -> Result<(), Vec<SensorError>>` - Initialize all sensors
- `poll_all(&mut self) -> SystemSnapshot` - Read all sensors
- `get_sensor(&mut self, id: &str) -> Option<&mut Box<dyn SensorDriver>>` - Access specific sensor
- `sensor_count(&self) -> usize` - Number of registered sensors

**Starter Code**:

```rust
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::string::String;

#[derive(Debug, Clone)]
pub struct SensorReading {
    pub sensor_id: String,
    pub data: RawData,
    pub timestamp: u64,
}

#[derive(Debug)]
pub struct SystemSnapshot {
    pub readings: Vec<SensorReading>,
    pub errors: Vec<(String, SensorError)>,
}

pub struct SensorManager {
    sensors: Vec<Box<dyn SensorDriver>>,
    poll_interval_ms: u64,
}

impl SensorManager {
    pub fn new(poll_interval_ms: u64) -> Self {
        // TODO: Initialize empty sensor list
        todo!("Implement SensorManager::new")
    }

    pub fn add_sensor(&mut self, sensor: Box<dyn SensorDriver>) {
        // TODO: Add sensor to collection
        todo!("Implement add_sensor")
    }

    pub fn init_all(&mut self) -> Result<(), Vec<SensorError>> {
        // TODO: Iterate sensors and call init on each
        // TODO: Collect errors from failed initializations
        // TODO: Return Ok if all succeed, Err with error list if any fail
        todo!("Implement init_all")
    }

    pub fn poll_all(&mut self) -> SystemSnapshot {
        // TODO: Iterate sensors
        // TODO: Call read_raw on each, capturing result
        // TODO: Build reading for successes, error entry for failures
        // TODO: Return SystemSnapshot with both readings and errors
        todo!("Implement poll_all")
    }

    pub fn get_sensor(&mut self, id: &str) -> Option<&mut Box<dyn SensorDriver>> {
        // TODO: Find sensor with matching ID
        // HINT: Use find() with sensor_id() check
        todo!("Implement get_sensor")
    }

    pub fn sensor_count(&self) -> usize {
        // TODO: Return number of sensors
        todo!("Implement sensor_count")
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    struct AlwaysSucceedSensor {
        id: String,
    }

    impl SensorDriver for AlwaysSucceedSensor {
        fn init(&mut self) -> Result<(), SensorError> {
            Ok(())
        }

        fn read_raw(&mut self) -> Result<RawData, SensorError> {
            Ok(RawData {
                values: [42, 43, 44],
                timestamp_ms: 1000,
            })
        }

        fn sensor_id(&self) -> &str {
            &self.id
        }
    }

    struct AlwaysFailSensor {
        id: String,
    }

    impl SensorDriver for AlwaysFailSensor {
        fn init(&mut self) -> Result<(), SensorError> {
            Err(SensorError::InitializationError)
        }

        fn read_raw(&mut self) -> Result<RawData, SensorError> {
            Err(SensorError::DataError)
        }

        fn sensor_id(&self) -> &str {
            &self.id
        }
    }

    #[test]
    fn test_manager_add_sensors() {
        let mut manager = SensorManager::new(1000);
        assert_eq!(manager.sensor_count(), 0);

        manager.add_sensor(Box::new(AlwaysSucceedSensor {
            id: "temp1".to_string(),
        }));
        assert_eq!(manager.sensor_count(), 1);

        manager.add_sensor(Box::new(AlwaysSucceedSensor {
            id: "accel1".to_string(),
        }));
        assert_eq!(manager.sensor_count(), 2);
    }

    #[test]
    fn test_init_all_success() {
        let mut manager = SensorManager::new(1000);
        manager.add_sensor(Box::new(AlwaysSucceedSensor {
            id: "sensor1".to_string(),
        }));
        manager.add_sensor(Box::new(AlwaysSucceedSensor {
            id: "sensor2".to_string(),
        }));

        assert!(manager.init_all().is_ok());
    }

    #[test]
    fn test_init_all_with_failures() {
        let mut manager = SensorManager::new(1000);
        manager.add_sensor(Box::new(AlwaysSucceedSensor {
            id: "good".to_string(),
        }));
        manager.add_sensor(Box::new(AlwaysFailSensor {
            id: "bad".to_string(),
        }));

        let result = manager.init_all();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().len(), 1);
    }

    #[test]
    fn test_poll_all() {
        let mut manager = SensorManager::new(1000);
        manager.add_sensor(Box::new(AlwaysSucceedSensor {
            id: "temp".to_string(),
        }));
        manager.add_sensor(Box::new(AlwaysSucceedSensor {
            id: "accel".to_string(),
        }));

        manager.init_all().unwrap();
        let snapshot = manager.poll_all();

        assert_eq!(snapshot.readings.len(), 2);
        assert_eq!(snapshot.errors.len(), 0);
        assert_eq!(snapshot.readings[0].data.values[0], 42);
    }

    #[test]
    fn test_poll_with_errors() {
        let mut manager = SensorManager::new(1000);
        manager.add_sensor(Box::new(AlwaysSucceedSensor {
            id: "good".to_string(),
        }));

        let mut fail_sensor = AlwaysFailSensor {
            id: "bad".to_string(),
        };
        // Manually init to bypass init_all check
        manager.sensors.push(Box::new(fail_sensor));

        let snapshot = manager.poll_all();
        assert_eq!(snapshot.readings.len(), 1); // Only good sensor
        assert_eq!(snapshot.errors.len(), 1);   // Bad sensor error
    }

    #[test]
    fn test_get_sensor() {
        let mut manager = SensorManager::new(1000);
        manager.add_sensor(Box::new(AlwaysSucceedSensor {
            id: "test-sensor".to_string(),
        }));

        let sensor = manager.get_sensor("test-sensor");
        assert!(sensor.is_some());
        assert_eq!(sensor.unwrap().sensor_id(), "test-sensor");

        assert!(manager.get_sensor("nonexistent").is_none());
    }
}
```

**Check Your Understanding**:
- Why use `Box<dyn SensorDriver>` instead of generics?
- What's the trade-off between Vec and a fixed-size array for sensors?
- How does `SystemSnapshot` enable error handling without panicking?

---

#### Why Milestone 4 Isn't Enough

**Limitation**: The manager works but requires `alloc` (heap allocation). Many embedded systems are `no_std` without allocators, needing static storage.

**What we're adding**: A `no_std`-compatible manager using `heapless::Vec` for static allocation, making it suitable for bare-metal microcontrollers.

**Improvement**:
- **Portability**: Works on microcontrollers without heap
- **Determinism**: Predictable memory usage (no allocator)
- **Safety**: Compile-time capacity checking
- **Performance**: Eliminates allocation overhead

---

### Milestone 5: No-Std Static Manager

**Goal**: Refactor the sensor manager to work in `no_std` environments using compile-time-sized collections, enabling deployment on bare-metal microcontrollers.

**Why this milestone**: Real embedded systems often don't have heap allocators. This milestone teaches static allocation patterns and `no_std` constraints.

#### Architecture

**Key Changes:**
- Replace `Vec` with `heapless::Vec` (fixed capacity)
- Replace `String` with `heapless::String` or `&'static str`
- Use const generics for maximum sensor count
- Remove `alloc` dependency

**Structs:**
- `SensorManager<const N: usize>` - Manager with compile-time capacity
  - **Field**: `sensors: heapless::Vec<Box<dyn SensorDriver>, N>` - Fixed-capacity sensor list
  - **Field**: `poll_interval_ms: u64` - Poll interval

- `SensorReading<'a>` - Reading with borrowed sensor ID
  - **Field**: `sensor_id: &'a str` - Borrowed sensor name
  - **Field**: `data: RawData` - Sensor data
  - **Field**: `timestamp: u64` - Timestamp

**Functions:**
- Same as Milestone 4, but with capacity-aware error handling

**Starter Code**:

```rust
#![no_std]

use heapless::Vec;
use core::fmt;

// Maximum sensor ID length
const MAX_ID_LEN: usize = 16;

#[derive(Debug, Clone)]
pub struct SensorReading<'a> {
    pub sensor_id: &'a str,
    pub data: RawData,
    pub timestamp: u64,
}

pub struct SystemSnapshot<'a, const N: usize> {
    pub readings: Vec<SensorReading<'a>, N>,
    pub errors: Vec<(&'a str, SensorError), N>,
}

pub struct SensorManager<const N: usize> {
    sensors: Vec<&'static mut dyn SensorDriver, N>,
    poll_interval_ms: u64,
}

impl<const N: usize> SensorManager<N> {
    pub fn new(poll_interval_ms: u64) -> Self {
        // TODO: Initialize with heapless::Vec::new()
        todo!("Implement SensorManager::new")
    }

    pub fn add_sensor(&mut self, sensor: &'static mut dyn SensorDriver) -> Result<(), ()> {
        // TODO: Try to push sensor
        // TODO: Return Err if capacity exceeded
        todo!("Implement add_sensor")
    }

    pub fn init_all(&mut self) -> Result<(), Vec<SensorError, N>> {
        // TODO: Initialize all sensors
        // TODO: Collect errors in heapless::Vec
        todo!("Implement init_all")
    }

    pub fn poll_all(&mut self) -> SystemSnapshot<N> {
        // TODO: Poll all sensors
        // TODO: Build snapshot with heapless::Vec
        // TODO: Handle capacity limits gracefully
        todo!("Implement poll_all")
    }

    pub fn get_sensor(&mut self, id: &str) -> Option<&mut &'static mut dyn SensorDriver> {
        // TODO: Find sensor by ID
        todo!("Implement get_sensor")
    }

    pub fn sensor_count(&self) -> usize {
        self.sensors.len()
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Note: In real embedded code, sensors would be static mut
    // Here we simulate with local statics for testing

    static mut TEST_SENSOR1: Option<TestSensor> = None;
    static mut TEST_SENSOR2: Option<TestSensor> = None;

    struct TestSensor {
        id: &'static str,
        fail: bool,
    }

    impl SensorDriver for TestSensor {
        fn init(&mut self) -> Result<(), SensorError> {
            if self.fail {
                Err(SensorError::InitializationError)
            } else {
                Ok(())
            }
        }

        fn read_raw(&mut self) -> Result<RawData, SensorError> {
            if self.fail {
                Err(SensorError::DataError)
            } else {
                Ok(RawData {
                    values: [1, 2, 3],
                    timestamp_ms: 1000,
                })
            }
        }

        fn sensor_id(&self) -> &str {
            self.id
        }
    }

    #[test]
    fn test_nostd_manager_capacity() {
        const CAPACITY: usize = 2;
        let mut manager: SensorManager<CAPACITY> = SensorManager::new(1000);

        unsafe {
            TEST_SENSOR1 = Some(TestSensor {
                id: "sensor1",
                fail: false,
            });
            TEST_SENSOR2 = Some(TestSensor {
                id: "sensor2",
                fail: false,
            });

            // Should succeed - within capacity
            assert!(manager.add_sensor(TEST_SENSOR1.as_mut().unwrap()).is_ok());
            assert!(manager.add_sensor(TEST_SENSOR2.as_mut().unwrap()).is_ok());

            // Should fail - exceeds capacity
            let mut extra = TestSensor {
                id: "extra",
                fail: false,
            };
            assert!(manager.add_sensor(&mut extra).is_err());
        }
    }

    #[test]
    fn test_nostd_poll() {
        const CAPACITY: usize = 4;
        let mut manager: SensorManager<CAPACITY> = SensorManager::new(100);

        unsafe {
            TEST_SENSOR1 = Some(TestSensor {
                id: "temp",
                fail: false,
            });

            manager.add_sensor(TEST_SENSOR1.as_mut().unwrap()).unwrap();
            manager.init_all().unwrap();

            let snapshot = manager.poll_all();
            assert_eq!(snapshot.readings.len(), 1);
            assert_eq!(snapshot.errors.len(), 0);
        }
    }

    #[test]
    fn test_nostd_mixed_results() {
        const CAPACITY: usize = 4;
        let mut manager: SensorManager<CAPACITY> = SensorManager::new(100);

        unsafe {
            TEST_SENSOR1 = Some(TestSensor {
                id: "good",
                fail: false,
            });
            TEST_SENSOR2 = Some(TestSensor {
                id: "bad",
                fail: true,
            });

            manager.add_sensor(TEST_SENSOR1.as_mut().unwrap()).unwrap();
            manager.add_sensor(TEST_SENSOR2.as_mut().unwrap()).unwrap();

            // Init will fail for "bad"
            assert!(manager.init_all().is_err());

            // Poll will show mixed results
            let snapshot = manager.poll_all();
            assert_eq!(snapshot.readings.len(), 1);  // Only "good"
            assert_eq!(snapshot.errors.len(), 1);    // "bad" fails
        }
    }
}
```

**Check Your Understanding**:
- Why use `&'static mut dyn SensorDriver` instead of `Box<dyn>`?
- What happens if you try to add more sensors than capacity N?
- How does this design work without a heap allocator?

---

#### Why Milestone 5 Isn't Enough

**Limitation**: The manager polls sensors sequentially, which is fine for a few sensors but doesn't scale. With 10+ sensors, polling becomes slow and blocks the system.

**What we're adding**: Asynchronous sensor reading using Embassy's async/await, enabling concurrent sensor polling without blocking.

**Improvement**:
- **Concurrency**: Poll multiple sensors simultaneously
- **Efficiency**: CPU sleeps between polls instead of busy-waiting
- **Responsiveness**: Fast sensors don't wait for slow ones
- **Scalability**: Handle 20+ sensors without performance degradation

---

### Milestone 6: Async Embassy Integration

**Goal**: Integrate async sensor reading using Embassy executor, enabling concurrent sensor polling with minimal resource overhead.

**Why this milestone**: Modern embedded systems need efficient concurrency. This milestone teaches async embedded patterns and demonstrates the power of HAL abstraction—the same drivers work in both sync and async contexts.

#### Architecture

**Key Additions:**
- Embassy executor integration
- Async sensor reading tasks
- Channel-based data collection
- Periodic polling with timers

**Structs:**
- `AsyncSensorManager<const N: usize>` - Async manager
  - **Field**: `sensors: Vec<&'static mut dyn SensorDriver, N>` - Sensor collection
  - **Field**: `sender: Sender<SensorReading>` - Channel for readings

**Functions:**
- `async fn poll_sensor_task(sensor: &mut dyn SensorDriver, sender: Sender)` - Per-sensor task
- `async fn collect_readings(receiver: Receiver) -> SystemSnapshot` - Aggregator task
- `async fn run_manager(manager: AsyncSensorManager)` - Main manager loop

**Starter Code**:

```rust
use embassy_executor::Spawner;
use embassy_sync::channel::{Channel, Sender, Receiver};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::{Duration, Timer};

// Channel for sensor readings (capacity 16)
static SENSOR_CHANNEL: Channel<NoopRawMutex, SensorReading, 16> = Channel::new();

/// Async task that polls a single sensor periodically
#[embassy_executor::task]
async fn poll_sensor_task(
    mut sensor: &'static mut dyn SensorDriver,
    interval_ms: u64,
) {
    // TODO: Initialize sensor
    // TODO: Loop forever:
    //   - Read sensor data
    //   - Send to channel
    //   - Sleep for interval
    todo!("Implement poll_sensor_task")
}

/// Collect readings from channel
pub async fn collect_readings(
    receiver: Receiver<'static, NoopRawMutex, SensorReading, 16>,
    count: usize,
) -> heapless::Vec<SensorReading, 16> {
    // TODO: Receive 'count' readings from channel
    // TODO: Return collected readings
    todo!("Implement collect_readings")
}

pub struct AsyncSensorManager<const N: usize> {
    sensors: heapless::Vec<&'static mut dyn SensorDriver, N>,
    poll_interval_ms: u64,
}

impl<const N: usize> AsyncSensorManager<N> {
    pub fn new(poll_interval_ms: u64) -> Self {
        Self {
            sensors: heapless::Vec::new(),
            poll_interval_ms,
        }
    }

    pub fn add_sensor(&mut self, sensor: &'static mut dyn SensorDriver) -> Result<(), ()> {
        self.sensors.push(sensor).map_err(|_| ())
    }

    /// Spawn polling tasks for all sensors
    pub async fn spawn_all(&'static mut self, spawner: Spawner) {
        // TODO: For each sensor, spawn poll_sensor_task
        todo!("Implement spawn_all")
    }
}

/// Main application using async manager
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // TODO: Create manager
    // TODO: Add sensors
    // TODO: Spawn sensor tasks
    // TODO: Spawn collection task
    // TODO: Main loop: collect and process readings
    todo!("Implement main")
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use embassy_executor::Executor;
    use embassy_time::Instant;

    // Mock sensor that counts reads
    struct CountingSensor {
        id: &'static str,
        read_count: core::sync::atomic::AtomicU32,
    }

    impl SensorDriver for CountingSensor {
        fn init(&mut self) -> Result<(), SensorError> {
            Ok(())
        }

        fn read_raw(&mut self) -> Result<RawData, SensorError> {
            let count = self.read_count.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
            Ok(RawData {
                values: [count as i16, 0, 0],
                timestamp_ms: Instant::now().as_millis(),
            })
        }

        fn sensor_id(&self) -> &str {
            self.id
        }
    }

    #[embassy_executor::test]
    async fn test_async_single_sensor() {
        static mut SENSOR: CountingSensor = CountingSensor {
            id: "test",
            read_count: core::sync::atomic::AtomicU32::new(0),
        };

        let sender = SENSOR_CHANNEL.sender();
        let receiver = SENSOR_CHANNEL.receiver();

        // Spawn sensor task
        spawner.spawn(poll_sensor_task(unsafe { &mut SENSOR }, 10)).unwrap();

        // Wait for a few readings
        Timer::after(Duration::from_millis(50)).await;

        // Should have multiple readings
        let readings = collect_readings(receiver, 3).await;
        assert_eq!(readings.len(), 3);

        // Counts should increment
        assert!(readings[1].data.values[0] > readings[0].data.values[0]);
    }

    #[embassy_executor::test]
    async fn test_concurrent_sensors() {
        static mut SENSOR1: CountingSensor = CountingSensor {
            id: "sensor1",
            read_count: core::sync::atomic::AtomicU32::new(0),
        };
        static mut SENSOR2: CountingSensor = CountingSensor {
            id: "sensor2",
            read_count: core::sync::atomic::AtomicU32::new(100),
        };

        // Spawn both sensors
        spawner.spawn(poll_sensor_task(unsafe { &mut SENSOR1 }, 10)).unwrap();
        spawner.spawn(poll_sensor_task(unsafe { &mut SENSOR2 }, 10)).unwrap();

        Timer::after(Duration::from_millis(50)).await;

        let receiver = SENSOR_CHANNEL.receiver();
        let readings = collect_readings(receiver, 6).await;

        // Should have readings from both sensors
        let sensor1_count = readings.iter().filter(|r| r.sensor_id == "sensor1").count();
        let sensor2_count = readings.iter().filter(|r| r.sensor_id == "sensor2").count();

        assert!(sensor1_count >= 2);
        assert!(sensor2_count >= 2);
    }
}
```

**Check Your Understanding**:
- How does async polling improve efficiency compared to sequential polling?
- Why use channels instead of shared state?
- What's the advantage of per-sensor tasks vs. one task polling all sensors?

---

## Complete Working Example

Here's a full implementation demonstrating all milestones integrated together:

```rust
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::{Duration, Timer};
use embedded_hal::i2c::I2c;
use embedded_hal::spi::SpiDevice;
use embedded_hal::digital::OutputPin;
use panic_probe as _;
use defmt::*;

// ===== Milestone 1: Traits and Errors =====

#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum SensorError {
    CommunicationError,
    InitializationError,
    DataError,
}

#[derive(Debug, Clone, Copy, defmt::Format)]
pub struct RawData {
    pub values: [i16; 3],
    pub timestamp_ms: u64,
}

pub trait SensorDriver {
    fn init(&mut self) -> Result<(), SensorError>;
    fn read_raw(&mut self) -> Result<RawData, SensorError>;
    fn sensor_id(&self) -> &str;
}

// ===== Milestone 2: I2C Temperature Sensor =====

const TEMP_REGISTER: u8 = 0x00;
const CONFIG_REGISTER: u8 = 0x01;

pub struct TempSensor<I2C> {
    i2c: I2C,
    address: u8,
    initialized: bool,
}

impl<I2C: I2c> TempSensor<I2C> {
    pub fn new(i2c: I2C, address: u8) -> Self {
        Self {
            i2c,
            address,
            initialized: false,
        }
    }

    pub fn read_celsius(&mut self) -> Result<f32, SensorError> {
        if !self.initialized {
            return Err(SensorError::InitializationError);
        }

        let mut buf = [0u8; 2];
        self.i2c
            .write_read(self.address, &[TEMP_REGISTER], &mut buf)
            .map_err(|_| SensorError::CommunicationError)?;

        let raw = u16::from_be_bytes(buf);
        let temp = ((raw >> 4) as f32) * 0.0625;
        Ok(temp)
    }
}

impl<I2C: I2c> SensorDriver for TempSensor<I2C> {
    fn init(&mut self) -> Result<(), SensorError> {
        // Set 12-bit resolution
        self.i2c
            .write(self.address, &[CONFIG_REGISTER, 0x60])
            .map_err(|_| SensorError::InitializationError)?;

        self.initialized = true;
        Ok(())
    }

    fn read_raw(&mut self) -> Result<RawData, SensorError> {
        let celsius = self.read_celsius()?;
        Ok(RawData {
            values: [(celsius * 10.0) as i16, 0, 0],
            timestamp_ms: embassy_time::Instant::now().as_millis(),
        })
    }

    fn sensor_id(&self) -> &str {
        "TMP102"
    }
}

// ===== Milestone 3: SPI Accelerometer =====

const DEVID_REG: u8 = 0x00;
const POWER_CTL: u8 = 0x2D;
const DATA_FORMAT: u8 = 0x31;
const DATA_X0: u8 = 0x32;

#[derive(Clone, Copy)]
pub enum AccelScale {
    Range2G = 0,
    Range4G = 1,
    Range8G = 2,
    Range16G = 3,
}

impl AccelScale {
    fn sensitivity(&self) -> f32 {
        match self {
            AccelScale::Range2G => 3.9,
            AccelScale::Range4G => 7.8,
            AccelScale::Range8G => 15.6,
            AccelScale::Range16G => 31.2,
        }
    }
}

pub struct AccelSensor<SPI, CS> {
    spi: SPI,
    cs: CS,
    scale: AccelScale,
    initialized: bool,
}

impl<SPI, CS> AccelSensor<SPI, CS>
where
    SPI: SpiDevice,
    CS: OutputPin,
{
    pub fn new(spi: SPI, cs: CS, scale: AccelScale) -> Self {
        Self {
            spi,
            cs,
            scale,
            initialized: false,
        }
    }

    pub fn read_xyz_raw(&mut self) -> Result<[i16; 3], SensorError> {
        let mut buf = [0u8; 7];
        buf[0] = DATA_X0 | 0x80 | 0x40; // Read bit + multi-byte

        self.cs.set_low().ok();
        self.spi.transfer_in_place(&mut buf)
            .map_err(|_| SensorError::CommunicationError)?;
        self.cs.set_high().ok();

        let x = i16::from_le_bytes([buf[1], buf[2]]);
        let y = i16::from_le_bytes([buf[3], buf[4]]);
        let z = i16::from_le_bytes([buf[5], buf[6]]);

        Ok([x, y, z])
    }

    fn write_reg(&mut self, reg: u8, value: u8) -> Result<(), SensorError> {
        self.cs.set_low().ok();
        self.spi.write(&[reg & 0x7F, value])
            .map_err(|_| SensorError::CommunicationError)?;
        self.cs.set_high().ok();
        Ok(())
    }

    fn read_reg(&mut self, reg: u8) -> Result<u8, SensorError> {
        let mut buf = [reg | 0x80, 0];
        self.cs.set_low().ok();
        self.spi.transfer_in_place(&mut buf)
            .map_err(|_| SensorError::CommunicationError)?;
        self.cs.set_high().ok();
        Ok(buf[1])
    }
}

impl<SPI, CS> SensorDriver for AccelSensor<SPI, CS>
where
    SPI: SpiDevice,
    CS: OutputPin,
{
    fn init(&mut self) -> Result<(), SensorError> {
        let dev_id = self.read_reg(DEVID_REG)?;
        if dev_id != 0xE5 {
            return Err(SensorError::InitializationError);
        }

        self.write_reg(POWER_CTL, 0x08)?; // Measurement mode
        self.write_reg(DATA_FORMAT, self.scale as u8)?;

        self.initialized = true;
        Ok(())
    }

    fn read_raw(&mut self) -> Result<RawData, SensorError> {
        let xyz = self.read_xyz_raw()?;
        Ok(RawData {
            values: xyz,
            timestamp_ms: embassy_time::Instant::now().as_millis(),
        })
    }

    fn sensor_id(&self) -> &str {
        "ADXL345"
    }
}

// ===== Milestone 6: Async Embassy Integration =====

#[derive(Clone, defmt::Format)]
pub struct SensorReading {
    pub sensor_id: &'static str,
    pub data: RawData,
}

static SENSOR_CHANNEL: Channel<NoopRawMutex, SensorReading, 16> = Channel::new();

#[embassy_executor::task]
async fn poll_sensor(
    sensor: &'static mut dyn SensorDriver,
    interval_ms: u64,
) {
    info!("Starting sensor: {}", sensor.sensor_id());

    if let Err(e) = sensor.init() {
        error!("Init failed for {}: {:?}", sensor.sensor_id(), e);
        return;
    }

    let sender = SENSOR_CHANNEL.sender();

    loop {
        match sensor.read_raw() {
            Ok(data) => {
                let reading = SensorReading {
                    sensor_id: sensor.sensor_id(),
                    data,
                };
                sender.send(reading).await;
            }
            Err(e) => {
                warn!("Read error from {}: {:?}", sensor.sensor_id(), e);
            }
        }

        Timer::after(Duration::from_millis(interval_ms)).await;
    }
}

#[embassy_executor::task]
async fn collect_and_log() {
    let receiver = SENSOR_CHANNEL.receiver();

    loop {
        let reading = receiver.receive().await;
        info!(
            "Sensor: {} | Values: [{}, {}, {}] | Time: {}ms",
            reading.sensor_id,
            reading.data.values[0],
            reading.data.values[1],
            reading.data.values[2],
            reading.data.timestamp_ms
        );
    }
}

// ===== Main Application =====

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("HAL Sensor System starting...");

    // Initialize hardware (platform-specific)
    let p = embassy_stm32::init(Default::default());

    // Setup I2C for temperature sensor
    let i2c = embassy_stm32::i2c::I2c::new(
        p.I2C1,
        p.PB6,
        p.PB7,
        embassy_stm32::interrupt::take!(I2C1_EV),
        p.DMA1_CH6,
        p.DMA1_CH7,
        embassy_stm32::i2c::Config::default(),
    );

    // Setup SPI for accelerometer
    let spi = embassy_stm32::spi::Spi::new(
        p.SPI1,
        p.PA5,
        p.PA7,
        p.PA6,
        p.DMA2_CH3,
        p.DMA2_CH2,
        embassy_stm32::spi::Config::default(),
    );
    let cs = embassy_stm32::gpio::Output::new(p.PA4, embassy_stm32::gpio::Level::High, embassy_stm32::gpio::Speed::VeryHigh);

    // Create static sensors
    static mut TEMP_SENSOR: Option<TempSensor<_>> = None;
    static mut ACCEL_SENSOR: Option<AccelSensor<_, _>> = None;

    unsafe {
        TEMP_SENSOR = Some(TempSensor::new(i2c, 0x48));
        ACCEL_SENSOR = Some(AccelSensor::new(spi, cs, AccelScale::Range2G));
    }

    // Spawn sensor polling tasks
    spawner.spawn(poll_sensor(
        unsafe { TEMP_SENSOR.as_mut().unwrap() },
        1000, // 1 second interval
    )).unwrap();

    spawner.spawn(poll_sensor(
        unsafe { ACCEL_SENSOR.as_mut().unwrap() },
        100, // 100ms interval
    )).unwrap();

    // Spawn collector task
    spawner.spawn(collect_and_log()).unwrap();

    info!("All tasks spawned. System running.");

    // Main loop can do other work
    loop {
        Timer::after(Duration::from_secs(10)).await;
        info!("System heartbeat - 10s");
    }
}
```

### Running the Complete Example

**On STM32 (bare metal):**
```toml
# Cargo.toml
[dependencies]
embassy-stm32 = { version = "0.1", features = ["stm32f401re"] }
embassy-executor = { version = "0.5", features = ["arch-cortex-m", "executor-thread"] }
embassy-sync = "0.5"
embassy-time = "0.3"
embedded-hal = "1.0"
heapless = "0.8"
defmt = "0.3"
panic-probe = "0.3"

[profile.release]
opt-level = "z"
lto = true
```

**Build and flash:**
```bash
cargo build --release
probe-rs run --chip STM32F401RETx target/thumbv7em-none-eabihf/release/sensor-system
```

**On Raspberry Pi (Linux):**
Replace hardware initialization with `rppal`:
```rust
use rppal::i2c::I2c as RppalI2c;
use rppal::spi::{Spi, Mode, SlaveSelect};
use rppal::gpio::Gpio;

let i2c = RppalI2c::new().unwrap();
let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_000_000, Mode::Mode3).unwrap();
let cs = Gpio::new().unwrap().get(25).unwrap().into_output();
```

**Expected output:**
```
INFO  HAL Sensor System starting...
INFO  Starting sensor: TMP102
INFO  Starting sensor: ADXL345
INFO  All tasks spawned. System running.
INFO  Sensor: TMP102 | Values: [235, 0, 0] | Time: 1023ms
INFO  Sensor: ADXL345 | Values: [16, -32, 1024] | Time: 1108ms
INFO  Sensor: TMP102 | Values: [236, 0, 0] | Time: 2024ms
INFO  Sensor: ADXL345 | Values: [18, -30, 1022] | Time: 2109ms
```

---

## Testing Your Implementation

### Unit Testing Strategy

1. **Mock Hardware**: Use `embedded-hal-mock` for I2C/SPI
2. **Trait Testing**: Verify each driver implements `SensorDriver` correctly
3. **Error Paths**: Test communication failures, init failures
4. **Async Testing**: Use Embassy test harness for concurrent behavior

### Integration Testing

1. **Loopback Tests**: Connect MOSI to MISO for SPI, SDA/SCL with pullups for I2C
2. **Mock Sensors**: Build simple hardware simulators (Arduino as I2C slave)
3. **Platform Matrix**: Test on multiple platforms (STM32F4, nRF52, Raspberry Pi)

### Example Test Command
```bash
# Unit tests (host)
cargo test

# Integration tests (hardware required)
cargo test --features integration-test --target thumbv7em-none-eabihf

# CI pipeline
cargo clippy -- -D warnings
cargo fmt -- --check
cargo test --all-features
```

---

## Extensions and Challenges

1. **Add More Sensors**: Implement drivers for BME280 (humidity), LIS3DH (accelerometer), BMP280 (pressure)
2. **Power Management**: Add sleep modes, wake-on-interrupt for battery-powered systems
3. **Calibration**: Implement offset/scale calibration storage in EEPROM
4. **Filtering**: Add moving average, Kalman filtering for noisy sensors
5. **Data Logging**: Store readings to SD card or flash memory
6. **Network Integration**: Send sensor data over MQTT or CoAP
7. **Safety**: Add watchdog timer, CRC checking for sensor data
8. **Performance**: Profile async vs sync polling overhead

This project demonstrates the full power of HAL abstraction in Rust embedded systems: portable, testable, efficient, and safe.
