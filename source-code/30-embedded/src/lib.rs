// Embedded & Real-Time Patterns Library
// This module provides documentation for the example patterns.

pub mod examples {
    //! # Embedded & Real-Time Patterns
    //!
    //! This crate demonstrates embedded programming patterns in Rust:
    //!
    //! ## Pattern 1: Layered HAL Drivers
    //! - Board Support Package (BSP) initialization
    //! - HAL trait abstractions (OutputPin, SPI, Delay)
    //! - Portable drivers across MCU families
    //! - Mock implementations for desktop testing
    //!
    //! ## Pattern 2: Static Allocation & Zero-Copy Buffers
    //! - heapless SPSC queues for ISR communication
    //! - Fixed-capacity vectors and ring buffers
    //! - DMA buffer placement and alignment
    //! - Double buffering for streaming
    //!
    //! ## Pattern 3: Interrupt-Safe Shared State
    //! - Atomic counters and flags
    //! - Critical section protected state
    //! - Event queues for ISR-to-main communication
    //! - Debouncing and shared bus access
    //!
    //! ## Pattern 4: Deterministic Scheduling
    //! - RTIC-style priority-based scheduling
    //! - Embassy-style async/await patterns
    //! - Channels for task isolation
    //! - Periodic task management
    //!
    //! Note: This crate uses desktop-compatible implementations.
    //! For actual embedded targets, use:
    //! - stm32f4xx-hal, cortex-m, cortex-m-rt for STM32
    //! - embassy-stm32, embassy-executor for Embassy
    //! - rtic crate for RTIC framework
}
