#![feature(no_std)]
#![feature(core)]
#![no_std]

#![crate_name = "multiboot"]
#![crate_type = "lib"]

extern crate core;

#[cfg(test)]
#[macro_use]
extern crate std;

use core::mem::{transmute};

/// Value that is in rax after multiboot jumps to our entry point
pub const SIGNATURE_RAX: u64 = 0x2BADB002;

#[derive(Debug)]
pub enum MemType {
    RAM = 1,
    Unusable = 2,
}


/// Multiboot struct clients mainly interact with
/// To create this use Multiboot::new()
pub struct Multiboot<'a> {
    header: &'a MultibootHeader,
    paddr_to_vaddr: fn(u64) -> u64,
}

/// Representation of Multiboot header according to specification.
#[derive(Debug)]
#[repr(packed)]
struct MultibootHeader {
    flags: u32,

    mem_lower: u32,
    mem_upper: u32,

    boot_device: u32,
    cmdline: u32,

    mods_count: u32,
    mods_addr: u32,

    elf_symbols: ElfSymbols,

    mmap_length: u32,
    mmap_addr: u32,
}

/// Multiboot format of the MMAP buffer.
/// Note that size is defined to be at -4 bytes.
#[derive(Debug)]
#[repr(packed)]
struct MemEntry {
    size: u32,
    base_addr: u64,
    length: u64,
    mtype: u32
}

#[derive(Debug)]
#[repr(packed)]
struct ElfSymbols {
    num: u32,
    size: u32,
    addr: u32,
    shndx: u32,
}

impl<'a> Multiboot<'a> {

    /// Initializes the multiboot structure.
    ///
    /// # Arguments
    ///
    ///  * `mboot_ptr` - The physical address of the multiboot header. On qemu for example
    ///                  this is typically at 0x9500.
    ///  * `paddr_to_vaddr` - Translation of the physical addresses into kernel addresses.
    ///
    ///  `paddr_to_vaddr` translates physical it into a kernel accessible address.
    ///  The simplest paddr_to_vaddr function would for example be just the identity
    ///  function. But this may vary depending on how your page table layout looks like.
    ///
    pub fn new(mboot_ptr: u64, paddr_to_vaddr: fn(paddr: u64) -> u64) -> Multiboot<'a> {
        let header = paddr_to_vaddr(mboot_ptr);
        let mb: &MultibootHeader = unsafe { transmute::<u64, &MultibootHeader>(header) };

        Multiboot { header: mb, paddr_to_vaddr: paddr_to_vaddr }
    }

    /// Discover all memory regions in the multiboot memory map.
    /// 
    /// # Arguments
    ///  
    ///  * `discovery_callback` - Function to notify your memory system about regions.
    ///
    pub fn find_memory(&'a self, discovery_callback: fn(base: u64, length: u64, MemType))
    {
        let paddr_to_vaddr = self.paddr_to_vaddr;

        let mut current = self.header.mmap_addr;
        let end = self.header.mmap_addr + self.header.mmap_length;
        while current < end
        {
            let memory_region: &MemEntry = unsafe { transmute::<u64, &MemEntry>(paddr_to_vaddr(current as u64)) };

            let mtype = match memory_region.mtype {
                1 => MemType::RAM,
                2 => MemType::Unusable,
                _ => MemType::Unusable
            };

            discovery_callback(memory_region.base_addr, memory_region.length, mtype);
            current += memory_region.size + 4;
        }
    }

}
