#![feature(lang_items, start, libc)]
#![no_std]

extern crate libc;
extern crate multiboot;

use multiboot::{Multiboot, PAddr};
use core::slice;
use core::mem;

pub fn paddr_to_slice<'a>(p: multiboot::PAddr, sz: usize) -> Option<&'a [u8]> {
    unsafe {
        let ptr = mem::transmute(p);
        Some(slice::from_raw_parts(ptr, sz))
    }
}

/// mboot_ptr is the initial pointer to the multiboot structure
/// provided in %ebx on start-up.
pub fn use_multiboot(mboot_ptr: PAddr) {
    unsafe {
        Multiboot::new(mboot_ptr, paddr_to_slice);
    }
}


#[start]
fn start(_argc: isize, _argv: *const *const u8) -> isize {
    use_multiboot(0x0);
    0
}

#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang = "panic_fmt"] fn panic_fmt() -> ! { loop {} }
