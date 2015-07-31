# Multiboot [![Build Status](https://travis-ci.org/gz/rust-multiboot.svg)](https://travis-ci.org/gz/rust-multiboot) [![Crates.io](https://img.shields.io/crates/v/multiboot.svg)](https://crates.io/crates/multiboot)

This is a multiboot (v1) library written entirely in rust. The code depends only on libcore.

## How-to use
```rust
/// Translate a physical memory address into a kernel addressable location.
pub fn paddr_to_kernel_vaddr(p: PAddr) -> VAddr {
    (p + KERNEL_BASE) as VAddr
}

/// mboot_ptr is the initial pointer to the multiboot structure
/// provided in %ebx on start-up.
pub fn use_multiboot(mboot_ptr: PAddr) {
    let mb = Multiboot::new(mboot_ptr,  memory::paddr_to_kernel_vaddr);
    mb.memory_regions().map(|regions| {
        for region in regions {
            println!("Found {:?}", region);
        }
    });

    mb.modules().map(|modules| {
        for module in modules {
            log!("Found {:?}", module);
        }
    }
}
```

Functionality is still not complete and patches are welcome!

## Documentation
* [API Documentation](http://gz.github.io/rust-multiboot/multiboot/)