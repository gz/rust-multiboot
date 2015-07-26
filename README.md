# Multiboot [![Build Status](https://travis-ci.org/gz/rust-multiboot.svg)](https://travis-ci.org/gz/rust-multiboot) [![Crates.io](https://img.shields.io/crates/v/multiboot.svg)](https://crates.io/crates/multiboot)

This is a multiboot (v1) library written in rust to be used in kernel level code. The code depends only on libcore. 

## How-to use
```rust
/// Translate a physical memory address into a kernel addressable location.
pub fn paddr_to_kernel_vaddr(p: PAddr) -> VAddr {
    (p + KERNEL_BASE) as VAddr
}

/// mboot_ptr is the initial pointer to the multiboot structure
/// provided in %ebx on start-up.
pub fn use_multiboot(mboot_ptr: PAddr) {
    // Create a new instance of the Multiboot struct
    let multiboot = Multiboot::new(mboot_ptr, paddr_to_kernel_vaddr);

    // Find all available memory regions:
    let cb = | base, size, mtype | { 
        println!("Found new memory region: {:x} -- {:x}", base, base+size); 
    };
    multiboot.find_memory(cb);

    // Find all multiboot provided modules:
    let mod_cb = | name, start, end | {
        log!("Found module {}: {:x} - {:x}", name, start, end);
    }
    multiboot.find_modules(mod_cb);
}
```

Functionality is still not complete and Patches are welcome!

## Documentation
* [API Documentation](http://gz.github.io/rust-multiboot/multiboot/)