# Multiboot

[![Build Status](https://travis-ci.org/gz/rust-multiboot.svg)](https://travis-ci.org/gz/rust-multiboot)

This is a multiboot (v1) library written in rust to be used in kernel level code. The code depends only on libcore. 

## How-to use
```rust
// Create a new instance of the Multiboot struct
let multiboot = Multiboot::new(mboot_ptr, paddr_to_kernel_vaddr);

// Find all available memory regions:
let cb = | base, size, mtype | { 
    println!("Found new memory region: {:x} -- {:x}, base, base+size); 
};
multiboot.find_memory(cb);

// Find all multiboot provided modules:
let mod_cb = | name, start, end | {
    log!("Found module {}: {:x} - {:x}", name, start, end);
}
multiboot.find_modules(mod_cb);
```

Functionality is still not complete and Patches are welcome!
