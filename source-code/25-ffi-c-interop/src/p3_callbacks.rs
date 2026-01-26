//! Pattern 3: Callback Patterns
//!
//! Demonstrates function pointer callbacks, stateful callbacks with user data,
//! and RAII-style callback management with automatic cleanup.

use std::os::raw::{c_int, c_void};

extern "C" {
    // Simple callbacks
    fn register_simple_callback(callback: extern "C" fn(c_int));
    fn trigger_simple_callbacks(value: c_int);
    fn clear_simple_callbacks();

    // Callbacks with user data
    fn register_callback_with_data(
        callback: extern "C" fn(*mut c_void, c_int),
        user_data: *mut c_void,
    );
    fn trigger_callbacks_with_data(value: c_int);
    fn clear_callbacks_with_data();

    // Managed callbacks
    fn register_managed_callback(
        callback: extern "C" fn(*mut c_void, c_int),
        user_data: *mut c_void,
    ) -> c_int;
    fn unregister_managed_callback(handle: c_int);
    fn trigger_managed_callbacks(value: c_int);
}

// ===========================================
// Simple stateless callback
// ===========================================

extern "C" fn simple_callback(value: c_int) {
    println!("  [Rust] Simple callback received: {}", value);
}

extern "C" fn another_callback(value: c_int) {
    println!("  [Rust] Another callback received: {} (doubled: {})", value, value * 2);
}

// ===========================================
// Stateful callback with user data
// ===========================================

struct CallbackState {
    name: String,
    count: i32,
    values: Vec<i32>,
}

extern "C" fn stateful_callback(user_data: *mut c_void, value: c_int) {
    unsafe {
        let state = &mut *(user_data as *mut CallbackState);
        state.count += 1;
        state.values.push(value);
        println!("  [{}] Callback #{}: received {}", state.name, state.count, value);
    }
}

// ===========================================
// RAII-managed callback
// ===========================================

struct ManagedCallback {
    handle: c_int,
    // Box ensures the state stays at a fixed memory address
    _state: Box<CallbackState>,
}

impl ManagedCallback {
    fn new(name: &str) -> Self {
        let mut state = Box::new(CallbackState {
            name: name.to_string(),
            count: 0,
            values: Vec::new(),
        });

        let handle = unsafe {
            register_managed_callback(
                stateful_callback,
                &mut *state as *mut CallbackState as *mut c_void,
            )
        };

        println!("  Registered managed callback '{}' with handle {}", name, handle);

        ManagedCallback {
            handle,
            _state: state,
        }
    }
}

impl Drop for ManagedCallback {
    fn drop(&mut self) {
        unsafe {
            unregister_managed_callback(self.handle);
        }
        println!("  Unregistered managed callback with handle {}", self.handle);
    }
}

fn main() {
    println!("=== Pattern 3: Callback Patterns ===\n");

    // --- Simple Stateless Callbacks ---
    println!("--- Simple Stateless Callbacks ---");

    unsafe {
        clear_simple_callbacks();

        register_simple_callback(simple_callback);
        register_simple_callback(another_callback);

        println!("Triggering with value 42:");
        trigger_simple_callbacks(42);

        println!("Triggering with value 100:");
        trigger_simple_callbacks(100);

        clear_simple_callbacks();
    }

    // --- Callbacks with User Data (Stateful) ---
    println!("\n--- Stateful Callbacks (User Data Pattern) ---");

    let mut state1 = CallbackState {
        name: "Counter1".to_string(),
        count: 0,
        values: Vec::new(),
    };

    let mut state2 = CallbackState {
        name: "Counter2".to_string(),
        count: 0,
        values: Vec::new(),
    };

    unsafe {
        clear_callbacks_with_data();

        register_callback_with_data(
            stateful_callback,
            &mut state1 as *mut _ as *mut c_void,
        );
        register_callback_with_data(
            stateful_callback,
            &mut state2 as *mut _ as *mut c_void,
        );

        println!("Triggering callbacks with values 10, 20, 30:");
        trigger_callbacks_with_data(10);
        trigger_callbacks_with_data(20);
        trigger_callbacks_with_data(30);

        clear_callbacks_with_data();
    }

    println!("\nState after callbacks:");
    println!("  {}: {} calls, values: {:?}", state1.name, state1.count, state1.values);
    println!("  {}: {} calls, values: {:?}", state2.name, state2.count, state2.values);

    // --- RAII-Managed Callbacks ---
    println!("\n--- RAII-Managed Callbacks ---");

    {
        println!("Creating managed callbacks in inner scope:");
        let _cb1 = ManagedCallback::new("Managed1");
        let _cb2 = ManagedCallback::new("Managed2");

        unsafe {
            println!("\nTriggering managed callbacks:");
            trigger_managed_callbacks(100);
            trigger_managed_callbacks(200);
        }

        println!("\nLeaving inner scope (callbacks will be auto-unregistered):");
    } // _cb1 and _cb2 are dropped here, unregistering callbacks

    unsafe {
        println!("\nTriggering after scope exit (should be no output from callbacks):");
        trigger_managed_callbacks(999);
    }

    // --- Non-Capturing Closure as Callback ---
    println!("\n--- Non-Capturing Closure as Callback ---");

    // Non-capturing closures can be coerced to function pointers
    let closure_callback: extern "C" fn(c_int) = {
        extern "C" fn wrapper(v: c_int) {
            println!("  [Closure] Value squared: {}", v * v);
        }
        wrapper
    };

    unsafe {
        clear_simple_callbacks();
        register_simple_callback(closure_callback);

        println!("Triggering closure callback with 7:");
        trigger_simple_callbacks(7);

        clear_simple_callbacks();
    }

    println!("\nAll callback examples completed successfully!");
}
