# Multiboot [![Build Status](https://travis-ci.org/gz/rust-multiboot.svg)](https://travis-ci.org/gz/rust-multiboot) [![Crates.io](https://img.shields.io/crates/v/multiboot.svg)](https://crates.io/crates/multiboot)

This is a multiboot (v1) library written entirely in rust. The code depends only on libcore.

## How-to use
```rust
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
```

Functionality is still not complete and patches are welcome!

## Documentation
* [API Documentation](http://gz.github.io/rust-multiboot/multiboot/)
