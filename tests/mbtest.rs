extern crate core;
extern crate multiboot;

use core::mem;
use core::slice;
use multiboot::{Multiboot, PAddr};

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

fn main() {
    use_multiboot(0x0);
}
