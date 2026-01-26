#ifndef MYLIB_H
#define MYLIB_H

#include <stddef.h>
#include <stdint.h>

// ===========================================
// Pattern 1: C ABI Compatibility
// ===========================================

// C-compatible struct for demonstrating layout
typedef struct {
    uint8_t a;      // 1 byte + 3 padding
    uint32_t b;     // 4 bytes
    uint16_t c;     // 2 bytes + 2 padding
} CStruct;

// Function to work with the struct
int32_t process_struct(const CStruct* s);

// Simple math functions
int32_t c_add(int32_t a, int32_t b);
double c_sqrt(double x);
int32_t c_abs(int32_t n);

// ===========================================
// Pattern 2: String Handling
// ===========================================

// Takes a C string and returns its length
size_t c_string_length(const char* s);

// Concatenates two strings (caller must free result)
char* c_string_concat(const char* a, const char* b);

// Frees a string allocated by this library
void c_string_free(char* s);

// Prints a message
void c_print_message(const char* msg);

// ===========================================
// Pattern 3: Callbacks
// ===========================================

// Simple callback type
typedef void (*simple_callback_t)(int32_t value);

// Callback with user data
typedef void (*callback_with_data_t)(void* user_data, int32_t value);

// Register and trigger callbacks
void register_simple_callback(simple_callback_t callback);
void trigger_simple_callbacks(int32_t value);
void clear_simple_callbacks(void);

// Callbacks with user data
void register_callback_with_data(callback_with_data_t callback, void* user_data);
void trigger_callbacks_with_data(int32_t value);
void clear_callbacks_with_data(void);

// Managed callbacks (returns handle for unregistration)
int32_t register_managed_callback(callback_with_data_t callback, void* user_data);
void unregister_managed_callback(int32_t handle);
void trigger_managed_callbacks(int32_t value);

// ===========================================
// Pattern 4: Error Handling
// ===========================================

// Error codes
#define SUCCESS 0
#define ERROR_NULL_POINTER -1
#define ERROR_INVALID_INPUT -2
#define ERROR_COMPUTATION_FAILED -3

// Function that can fail
int32_t c_divide(int32_t a, int32_t b, int32_t* result);

// Get error message for code
const char* c_error_message(int32_t error_code);

// ===========================================
// Pattern 5: Opaque Types (like database handles)
// ===========================================

// Forward declaration - implementation hidden
typedef struct db_connection db_connection_t;

// Database operations
db_connection_t* db_open(const char* path);
int32_t db_execute(db_connection_t* conn, const char* sql);
const char* db_get_last_error(db_connection_t* conn);
void db_close(db_connection_t* conn);

#endif // MYLIB_H
