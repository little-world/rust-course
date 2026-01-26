#include "mylib.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>

// ===========================================
// Pattern 1: C ABI Compatibility
// ===========================================

int32_t process_struct(const CStruct* s) {
    if (s == NULL) return -1;
    return (int32_t)(s->a + s->b + s->c);
}

int32_t c_add(int32_t a, int32_t b) {
    return a + b;
}

double c_sqrt(double x) {
    return sqrt(x);
}

int32_t c_abs(int32_t n) {
    return n < 0 ? -n : n;
}

// ===========================================
// Pattern 2: String Handling
// ===========================================

size_t c_string_length(const char* s) {
    if (s == NULL) return 0;
    return strlen(s);
}

char* c_string_concat(const char* a, const char* b) {
    if (a == NULL || b == NULL) return NULL;

    size_t len_a = strlen(a);
    size_t len_b = strlen(b);
    char* result = (char*)malloc(len_a + len_b + 1);

    if (result == NULL) return NULL;

    strcpy(result, a);
    strcat(result, b);
    return result;
}

void c_string_free(char* s) {
    free(s);
}

void c_print_message(const char* msg) {
    if (msg != NULL) {
        printf("[C] %s\n", msg);
    }
}

// ===========================================
// Pattern 3: Callbacks
// ===========================================

#define MAX_CALLBACKS 16

// Simple callbacks storage
static simple_callback_t simple_callbacks[MAX_CALLBACKS];
static int simple_callback_count = 0;

void register_simple_callback(simple_callback_t callback) {
    if (simple_callback_count < MAX_CALLBACKS && callback != NULL) {
        simple_callbacks[simple_callback_count++] = callback;
    }
}

void trigger_simple_callbacks(int32_t value) {
    for (int i = 0; i < simple_callback_count; i++) {
        simple_callbacks[i](value);
    }
}

void clear_simple_callbacks(void) {
    simple_callback_count = 0;
}

// Callbacks with user data
typedef struct {
    callback_with_data_t callback;
    void* user_data;
} callback_entry_t;

static callback_entry_t callbacks_with_data[MAX_CALLBACKS];
static int callback_with_data_count = 0;

void register_callback_with_data(callback_with_data_t callback, void* user_data) {
    if (callback_with_data_count < MAX_CALLBACKS && callback != NULL) {
        callbacks_with_data[callback_with_data_count].callback = callback;
        callbacks_with_data[callback_with_data_count].user_data = user_data;
        callback_with_data_count++;
    }
}

void trigger_callbacks_with_data(int32_t value) {
    for (int i = 0; i < callback_with_data_count; i++) {
        callbacks_with_data[i].callback(callbacks_with_data[i].user_data, value);
    }
}

void clear_callbacks_with_data(void) {
    callback_with_data_count = 0;
}

// Managed callbacks
typedef struct {
    int32_t handle;
    callback_with_data_t callback;
    void* user_data;
    int active;
} managed_callback_t;

static managed_callback_t managed_callbacks[MAX_CALLBACKS];
static int32_t next_handle = 1;

int32_t register_managed_callback(callback_with_data_t callback, void* user_data) {
    for (int i = 0; i < MAX_CALLBACKS; i++) {
        if (!managed_callbacks[i].active) {
            managed_callbacks[i].handle = next_handle++;
            managed_callbacks[i].callback = callback;
            managed_callbacks[i].user_data = user_data;
            managed_callbacks[i].active = 1;
            return managed_callbacks[i].handle;
        }
    }
    return -1; // No space
}

void unregister_managed_callback(int32_t handle) {
    for (int i = 0; i < MAX_CALLBACKS; i++) {
        if (managed_callbacks[i].active && managed_callbacks[i].handle == handle) {
            managed_callbacks[i].active = 0;
            return;
        }
    }
}

void trigger_managed_callbacks(int32_t value) {
    for (int i = 0; i < MAX_CALLBACKS; i++) {
        if (managed_callbacks[i].active) {
            managed_callbacks[i].callback(managed_callbacks[i].user_data, value);
        }
    }
}

// ===========================================
// Pattern 4: Error Handling
// ===========================================

int32_t c_divide(int32_t a, int32_t b, int32_t* result) {
    if (result == NULL) {
        return ERROR_NULL_POINTER;
    }
    if (b == 0) {
        return ERROR_INVALID_INPUT;
    }
    *result = a / b;
    return SUCCESS;
}

const char* c_error_message(int32_t error_code) {
    switch (error_code) {
        case SUCCESS: return "Success";
        case ERROR_NULL_POINTER: return "Null pointer provided";
        case ERROR_INVALID_INPUT: return "Invalid input";
        case ERROR_COMPUTATION_FAILED: return "Computation failed";
        default: return "Unknown error";
    }
}

// ===========================================
// Pattern 5: Opaque Types
// ===========================================

struct db_connection {
    char* path;
    int connected;
    char last_error[256];
    int query_count;
};

db_connection_t* db_open(const char* path) {
    if (path == NULL) return NULL;

    db_connection_t* conn = (db_connection_t*)malloc(sizeof(db_connection_t));
    if (conn == NULL) return NULL;

    conn->path = strdup(path);
    conn->connected = 1;
    conn->last_error[0] = '\0';
    conn->query_count = 0;

    printf("[C] Database opened: %s\n", path);
    return conn;
}

int32_t db_execute(db_connection_t* conn, const char* sql) {
    if (conn == NULL || sql == NULL) {
        return ERROR_NULL_POINTER;
    }
    if (!conn->connected) {
        strcpy(conn->last_error, "Not connected");
        return ERROR_INVALID_INPUT;
    }

    conn->query_count++;
    printf("[C] Executed query #%d: %s\n", conn->query_count, sql);

    // Simulate error for specific query
    if (strstr(sql, "ERROR") != NULL) {
        strcpy(conn->last_error, "Simulated query error");
        return ERROR_COMPUTATION_FAILED;
    }

    conn->last_error[0] = '\0';
    return SUCCESS;
}

const char* db_get_last_error(db_connection_t* conn) {
    if (conn == NULL) return "Null connection";
    return conn->last_error;
}

void db_close(db_connection_t* conn) {
    if (conn != NULL) {
        printf("[C] Database closed: %s (queries: %d)\n", conn->path, conn->query_count);
        free(conn->path);
        free(conn);
    }
}
