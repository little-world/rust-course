// Pattern 2: Basic C Function Binding
extern "C" {
    fn abs(input: i32) -> i32;
    fn strlen(s: *const std::os::raw::c_char) -> usize;
    fn malloc(size: usize) -> *mut std::os::raw::c_void;
    fn free(ptr: *mut std::os::raw::c_void);
}

fn use_c_functions() {
    unsafe {
        let result = abs(-42);
        println!("abs(-42) = {}", result);

        let c_str = b"Hello\0";
        let len = strlen(c_str.as_ptr() as *const std::os::raw::c_char);
        println!("String length: {}", len);

        let ptr = malloc(100);
        if !ptr.is_null() {
            println!("Allocated 100 bytes at {:?}", ptr);
            free(ptr);
            println!("Freed memory");
        }
    }
}

fn main() {
    use_c_functions();
    println!("C functions example completed");
}
