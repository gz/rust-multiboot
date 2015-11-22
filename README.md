# Multiboot [![Build Status](https://travis-ci.org/gz/rust-multiboot.svg)](https://travis-ci.org/gz/rust-multiboot) [![Crates.io](https://img.shields.io/crates/v/multiboot.svg)](https://crates.io/crates/multiboot)

This is a multiboot (v1) library written entirely in rust. The code depends only on libcore.

## How-to use
```rust
/// Translate a physical memory address and size into a slice
pub unsafe fn paddr_to_slice<'a>(p: PAddr, sz: usize) -> Option<&'a [u8]> {
    let ptr = mem::transmute(p + KERNEL_BASE);
    Some(slice::from_raw_parts(ptr, sz)
}

/// mboot_ptr is the initial pointer to the multiboot structure
/// provided in %ebx on start-up.
pub fn use_multiboot(mboot_ptr: PAddr) {
    Multiboot::new(mboot_ptr,  memory::paddr_to_kernel_vaddr).map(|mb| {
        mb.memory_regions().map(|regions| {
            for region in regions {
                println!("Found {:?}", region);
            }
        });

        mb.modules().map(|modules| {
            for module in modules {
                log!("Found {:?}", module);
            }
            });
    });
}
```

Functionality is still not complete and patches are welcome!

## Documentation
* [API Documentation](http://gz.github.io/rust-multiboot/multiboot/)
