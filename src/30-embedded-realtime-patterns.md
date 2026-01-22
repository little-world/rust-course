#  Embedded & Real-Time Patterns

Rust on microcontrollers, bare metal SoCs, and hard real-time workloads demands strict control over memory, timing, and side effects. Without an OS, you must replace the standard library with `#![no_std]`, carefully manage interrupts, and keep predictable execution. This chapter assembles patterns that combine HAL abstractions, interrupt coordination, and deterministic scheduling so experienced Rustaceans can bring the language's safety guarantees to embedded constraints.

Modern embedded stacks typically follow a split architecture:
1. Board Support Package (BSP) initializes clocks, peripherals, and pin mappings.
2. Hardware Abstraction Layer (HAL) provides portable traits (`embedded-hal`, `embedded-io`).
3. Application logic plugs drivers together using zero-allocation data structures, lightweight schedulers, and strict error handling.

## Development Setups: Raspberry Pi vs. STM32

### Working Directly on a Raspberry Pi
Linux-based SBCs like Raspberry Pi let you run `cargo` natively, which is great for rapid iteration before jumping to bare metal.
- Install Rust with `rustup` on the Pi (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`), then add needed targets such as `armv7-unknown-linux-gnueabihf`.
- Use crates like `rppal` or `linux-embedded-hal` to access GPIO, SPI, and I2C without needing `no_std`.
- For deterministic services, pin tasks to cores with `taskset` and use `systemd` units to manage startup; for realtime kernels enable `PREEMPT_RT`.
- Deploy by copying binaries or using `cargo run --release --target armv7-unknown-linux-gnueabihf`, then supervise with `systemd`, `tmux`, or container runtimes.
- You can mock out HAL traits on the Pi while your final firmware targets a microcontroller—this chapter’s HAL patterns show how to keep code portable.

### Cross-Compiling for STM32 Boards
Bare-metal STM32 development needs a `no_std` build, cross toolchain, and a flashing/debug story.
- Install the `thumbv7em-none-eabihf` (or appropriate) target with `rustup target add thumbv7em-none-eabihf`.
- Use `probe-rs` (`cargo install probe-run`) or `openocd` + `gdb` for flashing/debug; `cargo embed` automates logging via RTT/defmt.
- HAL/BSP crates (e.g., `stm32f4xx-hal`, `stm32h7xx-hal`) provide clock setup and driver scaffolding—mirror your board layout there.
- Configure `.cargo/config.toml` with runner `probe-run --chip STM32F401RETx` for seamless `cargo run --release`.
- For CI, leverage `cargo xtask` scripts or `just` recipes to build both host-mock tests and firmware artifacts, ensuring determinism with `--locked --target`.

## Pattern 1: Layered HAL Drivers

*   **Problem**: Directly touching vendor registers makes code brittle and untestable. Porting across MCUs or even board revisions forces a rewrite.
*   **Solution**: Build drivers against `embedded-hal`-style traits and keep board-specific code isolated in a BSP. This separates volatile register fiddling from reusable business logic.
*   **Why It Matters**: HAL traits allow unit testing on the host, replaceable mocks, and reuse across Cortex-M, RISC-V, or even Linux-based targets.
*   **Use Cases**: Sensor drivers, communication stacks, PWM motor control, portable display drivers.

### Examples

#### Example: Board Support Layer

Board Support Package initializing hardware peripherals. The BSP owns vendor-specific code, configures clocks and GPIO, and exposes components through HAL traits. Application code receives ready-to-use peripherals without touching registers directly.

```rust
#![no_std]
#![no_main]

use stm32f4xx_hal::{pac, prelude::*, timer::CounterHz};
use embedded_hal::digital::v2::OutputPin;

pub struct Board {
    pub led: impl OutputPin<Error = core::convert::Infallible>,
    pub timer: CounterHz<'static, pac::TIM2>,
}

pub fn init() -> Board {
    let dp = pac::Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(84.MHz()).freeze();
    let gpioa = dp.GPIOA.split();

    let mut led = gpioa.pa5.into_push_pull_output();
    led.set_low().ok();

    let mut timer = dp.TIM2.counter_hz(&clocks);
    timer.start(1.Hz()).unwrap();

    Board { led, timer }
}
```

#### Example: Driver Consuming HAL Traits
 A heartbeat driver using only embedded-hal traits. By depending on OutputPin and CountDown abstractions rather than concrete types, the same driver works on any platform. Replace with mocks for desktop unit testing.

```rust
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::timer::CountDown;
use nb::block;

pub struct Heartbeat<P, T> {
    led: P,
    timer: T,
}

impl<P, T> Heartbeat<P, T>
where
    P: OutputPin<Error = core::convert::Infallible>,
    T: CountDown,
{
    pub fn new(led: P, timer: T) -> Self {
        Self { led, timer }
    }

    pub fn spin(mut self) -> ! {
        loop {
            self.led.set_high().ok();
            block!(self.timer.wait()).ok();
            self.led.set_low().ok();
            block!(self.timer.wait()).ok();
        }
    }
}
```

#### Example: Sensor Driver Abstracted Over SPI + Delay

 Portable IMU driver using SPI, GPIO, and delay traits. The driver manages chip select, transfers bytes, and handles timing delays. Identical code compiles for STM32, Nordic chips, or desktop mocks with fake SPI implementations.

```rust
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::blocking::spi::Transfer;

pub struct ImuDriver<SPI, CS, DELAY> {
    spi: SPI,
    cs: CS,
    delay: DELAY,
}

impl<SPI, CS, DELAY> ImuDriver<SPI, CS, DELAY>
where
    SPI: Transfer<u8>,
    CS: OutputPin<Error = core::convert::Infallible>,
    DELAY: DelayUs<u16>,
{
    pub fn new(spi: SPI, cs: CS, delay: DELAY) -> Self {
        Self { spi, cs, delay }
    }

    pub fn read_whoami(&mut self) -> Result<u8, SPI::Error> {
        let mut buf = [0x75, 0];
        self.cs.set_low().ok();
        self.spi.transfer(&mut buf)?;
        self.cs.set_high().ok();
        Ok(buf[1])
    }

    pub fn configure(&mut self) -> Result<(), SPI::Error> {
        self.write_reg(0x6B, 0x00)?;
        self.delay.delay_us(50);
        self.write_reg(0x1C, 0x10)?;
        Ok(())
    }

    fn write_reg(&mut self, reg: u8, value: u8) -> Result<(), SPI::Error> {
        let mut buf = [reg & 0x7F, value];
        self.cs.set_low().ok();
        self.spi.transfer(&mut buf)?;
        self.cs.set_high().ok();
        Ok(())
    }
}
```

#### Example: Raspberry Pi HAL Wrapper


 `rppal` GPIO as an embedded-hal OutputPin. Linux-based boards like Raspberry Pi can implement the same traits as bare-metal targets. The Heartbeat driver accepts this wrapper, enabling desktop prototyping before deploying to microcontrollers.

```rust
use embedded_hal::digital::v2::OutputPin;
use rppal::gpio::{Gpio, OutputPin as PiPin};

pub struct PiLed {
    pin: PiPin,
}

impl PiLed {
    pub fn new(pin_id: u8) -> Self {
        let pin = Gpio::new().unwrap().get(pin_id).unwrap().into_output();
        Self { pin }
    }
}

impl OutputPin for PiLed {
    type Error = core::convert::Infallible;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.pin.set_low();
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.pin.set_high();
        Ok(())
    }
}
```

You can now feed `PiLed` into the `Heartbeat` example and run the exact same logic on a Raspberry Pi for desktop prototyping.

**Testing tip**: replace `P` and `T` with fake implementations using `std` timers to coverage-test logic on the desktop.

## Pattern 2: Static Allocation & Zero-Copy Buffers

*   **Problem**: Dynamic allocation (`Vec`, `Box`) is often unavailable or banned in hard real-time systems. Yet peripherals require queues for DMA, networking, or logging.
*   **Solution**: Use `heapless`, `arrayvec`, or custom `static mut` buffers guarded by safe wrappers. Favor compile-time capacity, placement in specific memory sections, and DMA-friendly alignment.
*   **Why It Matters**: Static buffers make timing predictable and avoid allocator fragmentation. They also ease certification (MISRA, DO-178C) where dynamic memory is disallowed.
*   **Use Cases**: UART ring buffers, telemetry queues, sensor fusion windows, DMA descriptors.

### Examples

#### Example: Lock-Free Telemetry Queue

Heapless SPSC queue for interrupt-safe communication without heap allocation. The producer enqueues packets with atomic IDs, while the consumer dequeues without locks. Splitting into producer/consumer halves enables safe concurrent access from ISR and main loop.

```rust
use heapless::spsc::Queue;
use core::sync::atomic::{AtomicU32, Ordering};

static mut Q: Queue<[u8; 32], 8> = Queue::new();
static NEXT_ID: AtomicU32 = AtomicU32::new(0);

fn producer_task() {
    // Safety: only called before RTOS start, so we get a unique splitter.
    let (mut prod, _) = unsafe { Q.split() };
    let mut packet = [0u8; 32];
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    packet[..4].copy_from_slice(&id.to_le_bytes());
    prod.enqueue(packet).ok();
}

fn consumer_task() {
    let (_, mut cons) = unsafe { Q.split() };
    while let Some(pkt) = cons.dequeue() {
        process_packet(&pkt);
    }
}
```

#### Example: DMA Buffer Placement


DMA buffer in a specific memory section using link_section attribute. DMA controllers require buffers in accessible SRAM regions. Static placement ensures correct alignment and memory domain without modifying linker scripts, while the buffer persists for transfer lifetime.

```rust
#[link_section = ".dma_data"]
static mut ADC_BUFFER: [u16; 128] = [0; 128];

fn start_dma(adc: &mut AdcDma<'static>) {
    // Safety: DMA exclusively owns the buffer until transfer completes.
    unsafe { adc.start_dma(&mut ADC_BUFFER) }.unwrap();
}
```

#### Example: Fixed-Capacity Command Log

 Command history using heapless Vec with compile-time capacity of 32 entries. Critical sections protect concurrent access. When full, new commands are silently dropped. This pattern suits logging, telemetry buffers, and audit trails without dynamic allocation.

```rust
use core::cell::RefCell;
use critical_section::Mutex;
use heapless::Vec;

#[derive(Clone, Copy)]
pub struct Command {
    opcode: u8,
    payload: [u8; 4],
}

static COMMAND_LOG: Mutex<RefCell<Vec<Command, 32>>> =
    Mutex::new(RefCell::new(Vec::new()));

pub fn append_command(cmd: Command) {
    critical_section::with(|cs| {
        let mut log = COMMAND_LOG.borrow(cs).borrow_mut();
        log.push(cmd).ok(); // drop oldest silently when full
    });
}

pub fn latest() -> Option<Command> {
    critical_section::with(|cs| COMMAND_LOG.borrow(cs).borrow().last().copied())
}
```

#### Example: STM32 DMA Double Buffer

Configure a double-buffered DMA for continuous audio streaming. Two pre-allocated buffers alternate: while DMA fills one, the CPU processes the other. This ping-pong technique eliminates gaps in high-rate data streams like audio or software-defined radio applications.

```rust
#[link_section = ".sram_d2"]
static mut AUDIO_BUFFERS: [[i16; 256]; 2] = [[0; 256]; 2];

fn start_audio_dma(dma: &mut stm32h7xx_hal::dma::StreamX<DMA1>) {
    let (buf_a, buf_b) = unsafe {
        (
            &mut AUDIO_BUFFERS[0] as *mut _,
            &mut AUDIO_BUFFERS[1] as *mut _,
        )
    };
    unsafe {
        dma.set_memory0(buf_a as *mut _);
        dma.set_memory1(buf_b as *mut _);
    }
    dma.enable_double_buffer();
    dma.start();
}
```

ISR handlers can then refill whichever half just completed without races or heap usage.

## Pattern 3: Interrupt-Safe Shared State

*   **Problem**: ISRs need to communicate with foreground tasks without data races. `static mut` variables are unsafe, and `RefCell` panics in interrupts.
*   **Solution**: Use synchronization primitives tailored to bare metal: `cortex_m::interrupt::Mutex`, `critical_section::Mutex`, atomics, or lock-free queues. Disable interrupts only around the minimum critical section.
*   **Why It Matters**: Predictable interrupt latency, no priority inversion, and analyzable execution times.
*   **Use Cases**: Button debouncing, timer capture/compare, sensor event batching, cross-core mailboxes.

### Examples

#### Example: Critical Section with `Mutex`

Count button presses using cortex-m Mutex with RefCell. The free() function disables interrupts briefly while accessing shared state. Both ISR and main code use identical access patterns, preventing data races while keeping critical sections minimal for low latency.

```rust
use core::cell::RefCell;
use cortex_m::interrupt::{free, Mutex};

static BUTTON_COUNT: Mutex<RefCell<u32>> = Mutex::new(RefCell::new(0));

#[interrupt]
fn EXTI0() {
    free(|cs| {
        let mut count = BUTTON_COUNT.borrow(cs).borrow_mut();
        *count += 1;
    });
}

fn read_count() -> u32 {
    free(|cs| *BUTTON_COUNT.borrow(cs).borrow())
}
```

#### Example: Atomic Flag for Wake-Ups

AtomicBool for lock-free ISR-to-main communication. The ADC interrupt sets the flag with Release ordering, main loop checks with AcqRel swap. WFI sleeps until interrupts arrive, saving power while maintaining responsiveness without critical section overhead.

```rust
use core::sync::atomic::{AtomicBool, Ordering};

static DATA_READY: AtomicBool = AtomicBool::new(false);

#[interrupt]
fn ADC1() {
    DATA_READY.store(true, Ordering::Release);
}

fn main_loop() {
    loop {
        if DATA_READY.swap(false, Ordering::AcqRel) {
            handle_sample();
        }
        cortex_m::asm::wfi(); // sleep until next interrupt
    }
}
```

#### Example: Sharing Buses with `critical_section::Mutex`

 I2C sensor between contexts using critical_section::Mutex. The portable critical-section crate works across platforms, providing Send+Sync without unsafe blocks. Late initialization with Option allows hardware setup after boot while maintaining safe concurrent access patterns.

```rust
use core::cell::RefCell;
use critical_section::Mutex;

struct EnvSensor<I2C> {
    bus: I2C,
}

static SENSOR: Mutex<RefCell<Option<EnvSensor<I2cDriver>>>> =
    Mutex::new(RefCell::new(None));

fn init_sensor(bus: I2cDriver) {
    critical_section::with(|cs| {
        *SENSOR.borrow(cs).borrow_mut() = Some(EnvSensor { bus });
    });
}

fn read_temperature() -> Option<i16> {
    critical_section::with(|cs| {
        let mut guard = SENSOR.borrow(cs).borrow_mut();
        guard.as_mut().and_then(|sensor| sensor.bus.read_temp().ok())
    })
}
```

#### Example: Raspberry Pi GPIO Interrupt Counter

 GPIO interrupts on Raspberry Pi using rppal's async callback. AtomicU32 counts button presses lock-free, mirroring bare-metal ISR patterns. The callback triggers on falling edges, enabling responsive input handling while the main thread continues other work.

```rust
use rppal::gpio::{Gpio, Trigger};
use core::sync::atomic::{AtomicU32, Ordering};

static BUTTON_COUNT: AtomicU32 = AtomicU32::new(0);

fn init_button(pin: u8) -> Result<(), rppal::gpio::Error> {
    let gpio = Gpio::new()?;
    let mut button = gpio.get(pin)?.into_input_pulldown();
    button.set_async_interrupt(Trigger::FallingEdge, |_| {
        BUTTON_COUNT.fetch_add(1, Ordering::Relaxed);
    })?;
    Ok(())
}

fn button_presses() -> u32 {
    BUTTON_COUNT.load(Ordering::Relaxed)
}
```

## Pattern 4: Deterministic Scheduling with RTIC/Embassy

*   **Problem**: Cooperative `loop {}` architectures make it hard to guarantee deadlines when peripherals compete for CPU time.
*   **Solution**: Use a lightweight real-time framework (RTIC, Embassy) that models tasks as interrupt handlers with explicit priorities and resource locking.
*   **Why It Matters**: Priority-based scheduling gives bounded latency, automatic critical sections, and eliminates the need for a traditional RTOS.
*   **Use Cases**: Motor control loops, sensor fusion pipelines, industrial fieldbus stacks, battery management systems.

### Examples

#### Example: RTIC Task Graph

 RTIC framework managing motor control tasks. Shared rpm state is accessed via automatic locking based on task priorities. The sample task captures encoder readings while the higher-priority control task computes PWM duty cycles, all without heap allocation.

```rust
#![no_std]
#![no_main]

#[rtic::app(device = stm32f4xx_hal::pac, peripherals = true)]
mod app {
    use super::*;

    #[shared]
    struct Shared {
        rpm: u16,
    }

    #[local]
    struct Local {
        encoder: Encoder,
        pwm: PwmDriver,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local) {
        // init hardware ...
        (Shared { rpm: 0 }, Local { encoder, pwm })
    }

    #[task(binds = TIM2, shared = [rpm], local = [encoder])]
    fn sample(mut ctx: sample::Context) {
        let rpm_measurement = ctx.local.encoder.capture();
        ctx.shared.rpm.lock(|rpm| *rpm = rpm_measurement);
    }

    #[task(priority = 2, shared = [rpm], local = [pwm])]
    fn control(mut ctx: control::Context) {
        let rpm = *ctx.shared.rpm.lock(|rpm| rpm);
        let duty = pid_step(rpm);
        ctx.local.pwm.set_duty(duty);
    }
}
```

#### Example: Embassy Async Driver

 Embassy's async/await model for embedded systems. UART writes and timer delays use cooperative scheduling without blocking. The executor automatically enters low-power WFI sleep between async operations, combining ergonomic code with energy efficiency.

```rust
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let mut uart = Uart::new(p.USART2, p.PA2, p.PA3, Irqs, p.DMA1_CH6, p.DMA1_CH7);

    spawner.spawn(sample_task()).unwrap();

    loop {
        uart.write(b"ping\r\n").await.unwrap();
        Timer::after_secs(1).await;
    }
}
```

#### Example: Embassy Channels for Task Isolation

Decouple ADC sampling from filtering using Embassy channels. The sampler task sends readings at 2kHz into a bounded channel, while the filter task processes independently. Backpressure naturally throttles producers when consumers fall behind, preserving timing guarantees.

```rust
use embassy_executor::{Spawner, task};
use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::{Duration, Timer};

static ADC_SAMPLES: Channel<NoopRawMutex, u16, 8> = Channel::new();

#[task]
async fn adc_sampler() {
    loop {
        let sample = read_adc_sample();
        ADC_SAMPLES.send(sample).await;
        Timer::after(Duration::from_micros(500)).await;
    }
}

#[task]
async fn filter_task() {
    loop {
        let sample = ADC_SAMPLES.recv().await;
        let filtered = low_pass(sample);
        publish(filtered).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    spawner.spawn(adc_sampler()).unwrap();
    spawner.spawn(filter_task()).unwrap();
    embassy_time::Timer::after_secs(1).await;
}
```

**Design tip**: Keep ISR work minimal (capture timestamp, enqueue event) and defer heavy computation to lower-priority tasks to maintain deadlines.

---

**Checklist for Embedded Rust Patterns**
- Compile with `#![no_std]` and `panic_probe`/`defmt` for meaningful crash info.
- Keep unsafe code confined to BSP crates; expose safe APIs upward.
- Measure worst-case execution times (WCET) per task and ensure they fit within interrupt budgets.
- Use hardware timers for scheduling instead of busy loops to save power.
