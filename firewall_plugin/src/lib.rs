use std::slice;
use std::str;

#[no_mangle] // ✅ correct way to prevent name mangling
pub unsafe extern "C" fn should_allow(method_ptr: *const u8, method_len: usize) -> i32 {
    let data = slice::from_raw_parts(method_ptr, method_len);

    if let Ok(method_str) = str::from_utf8(data) {
        if method_str == "eth_sendTransaction" {
            return 0; // BLOCK
        }
    }

    1 // ALLOW
}
