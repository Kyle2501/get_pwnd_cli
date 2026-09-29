// lib.rs (Rust)
// Compile with: cargo build --target wasm32-unknown-unknown --release

use std::alloc::{alloc, dealloc, Layout};
use std::slice;
use std::str;

// 1. Allocate memory in WASM so Go can write into it
#[no_mangle]
pub extern "C" fn alloc(size: usize) *mut u8 {
    let alignment = std::mem::align_of::<u8>();
    let layout = Layout::from_size_align(size, alignment).unwrap();
    unsafe { alloc(layout) }
}

// 2. Free memory after use to prevent leaks
#[no_mangle]
pub unsafe extern "C" fn dealloc(ptr: *mut u8, size: usize) {
    let alignment = std::mem::align_of::<u8>();
    let layout = Layout::from_size_align(size, alignment).unwrap();
    dealloc(ptr, layout);
}

// 3. Process the string buffer (e.g., validate or transform)
#[no_mangle]
pub unsafe extern "C" fn process_string(ptr: *mut u8, len: usize) -> *mut u8 {
    // Reconstruct the string slice from the pointer and length
    let slice = slice::from_raw_parts(ptr, len);
    let input_str = str::from_utf8_slice(slice).unwrap_or("invalid utf8");

    // Perform your logic (e.g., transforming a command or filtering text)
    let output_string = format!("Processed: {}", input_str);
    
    // Allocate new memory for the output string
    let output_bytes = output_string.into_bytes();
    let output_len = output_bytes.len();
    
    let result_ptr = alloc(output_len + 4); // Reserve 4 bytes for length prefix, or return ptr + len separately
    
    // Write length as u32 prefix, followed by bytes (or use a tuple/struct approach)
    std::ptr::copy_nonoverlapping(output_bytes.as_ptr(), result_ptr.add(4), output_len);
    std::ptr::copy_nonoverlapping(&(output_len as u32).to_le_bytes()[..], result_ptr, 4);

    result_ptr
}
