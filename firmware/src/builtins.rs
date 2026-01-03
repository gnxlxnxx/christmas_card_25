// Our own memcopy: Small, but works (and saves ~300 bytes)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        unsafe { *dest.add(i) = *src.add(i) };
        i += 1;
    }
    dest
}

// And memset: Saves  ~100 bytes
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        unsafe { *s.add(i) = c as u8 };
        i += 1;
    }
    s
}
