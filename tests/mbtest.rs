extern crate core;
extern crate multiboot;

use core::mem;
use core::slice;
use multiboot::information::{MemoryManagement, Multiboot, PAddr};

struct Mem;

impl MemoryManagement for Mem {
    unsafe fn paddr_to_slice(&self, p: PAddr, sz: usize) -> Option<&'static [u8]> {
        let ptr = mem::transmute(p);
        Some(slice::from_raw_parts(ptr, sz))
    }
    
    unsafe fn allocate(&mut self, length: usize) -> Option<(PAddr, &mut [u8])> {
        None
    }
    
    unsafe fn deallocate(&mut self, addr: PAddr) {
        if addr != 0 {
            unimplemented!()
        }
    }
}

/// mboot_ptr is the initial pointer to the multiboot structure
/// provided in %ebx on start-up.
pub fn use_multiboot(mboot_ptr: PAddr) {
    unsafe {
        Multiboot::from_ptr(mboot_ptr, &mut Mem {});
    }
}

fn main() {
    use_multiboot(0x0);
}
