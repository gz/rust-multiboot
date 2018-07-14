#![feature(lang_items, core_intrinsics, panic_implementation)]
#![no_std]
#![no_main]

// Pull in the system libc library for what crt0.o likely requires.
extern crate multiboot;

use core::mem;
use core::panic::PanicInfo;
use core::slice;
use multiboot::{Multiboot, PAddr};

#[cfg(feature = "nightly")]
use core::panic::PanicInfo;

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

#[cfg(feature = "nightly")]
#[no_mangle]
#[main]
pub extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    use_multiboot(0x0);
    0
}

#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

#[lang = "eh_unwind_resume"]
#[no_mangle]
pub extern "C" fn rust_eh_unwind_resume() {}

#[panic_implementation]
#[no_mangle]
pub fn panic_impl(_info: &PanicInfo) -> ! {
    loop {}
}
